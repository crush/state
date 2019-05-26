use std::collections::LinkedList as List;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::process::{Child as ChildProcess, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::prelude::*;
use serde_json::error::{
    Error as DecodingError,
    Category as DecodingErrorCategory,
};

use crate::config::Config;
use crate::backend;
use crate::state;


/// Command dispatcher.
#[derive(Debug)]
pub struct Cmd;

#[derive(Debug)]
pub enum CmdErr {
    UnknownCommand,
    FailToRun,
    UnexpectedOutput(DecodingError),
    SupervisorCrashed,
    PersistError(backend::PersistErr),
}

#[derive(Debug)]
pub enum Event {
    ApplicationTerminated,
    ApplicationRestarted,
    StateRecorded,
    LogRecorded,
    Error(CmdErr),
}

pub struct Monitor {
    event_queue: Arc<Mutex<List<Event>>>,
    supervisor: JoinHandle<()>,
    to_supervisor: Channel,
}

enum Msg {
    Kill,
    StillActive,
    Event(Event),
}

struct Channel {
    send_msg: mpsc::Sender<Msg>,
    recv_msg: mpsc::Receiver<Msg>,
}

impl Cmd {
    pub fn execute<'main>(
        cfg: Config,
        args: &clap::ArgMatches<'main>
    ) -> Result<Monitor, CmdErr>
    {
        let subcmd_args = (
            args.subcommand_matches("run"),
            args.subcommand_matches("log"),
        );

        match subcmd_args {
            (
                Some(run_args),
                _,
            ) => {
                let app_path = run_args
                    .value_of("application")
                    .ok_or(CmdErr::FailToRun)?;
                run(cfg, app_path.to_string())
            }
            (
                _,
                Some(log_args),
            ) => log(log_args),
            _ => Err(CmdErr::UnknownCommand),
        }
    }
}

impl Monitor {
    fn new(thread_handle: JoinHandle<()>, chan: Channel) -> Self {
        Monitor {
            event_queue: Arc::new(Mutex::new(List::new())),
            supervisor: thread_handle,
            to_supervisor: chan,
        }
    }

    pub fn events(&self) -> Option<Vec<Event>> {
        None
    }

    pub fn wait_for_termination(self) -> Result<Vec<Event>, CmdErr> {
        let mut events = Vec::new();

        loop {
            match self.to_supervisor.recv() {
                // Termination conditions
                Ok(Some(Msg::Event(Event::ApplicationTerminated))) => {
                    println!("app terminated");
                    return Ok(events)
                },

                Ok(Some(Msg::Kill)) => {
                    println!("kill recvd");
                    return Ok(events)
                },
                
                Err(_) => {
                    println!("Error encountered");
                    return Err(CmdErr::SupervisorCrashed)
                },

                // Processing conditions
                Ok(Some(Msg::Event(event))) => {
                    println!("storing event: {}", event);
                    events.push(event)
                },

                Ok(Some(Msg::StillActive)) => {
                    println!("application still active");
                },

                Ok(None) => {
                    println!("no message received");
                },
            }

            thread::sleep(Duration::new(1, 0));
        }
    }
}

fn channel() -> (Channel, Channel) {
    let (s_msg, r_msg) = mpsc::channel();
    let (s_msg2, r_msg2) = mpsc::channel();

    let monitor_half = Channel {
        send_msg: s_msg,
        recv_msg: r_msg2,
    };

    let supervisor_half = Channel {
        send_msg: s_msg2,
        recv_msg: r_msg,
    };

    (monitor_half, supervisor_half)
}

fn run<'main>(
    cfg: Config,
    app_path: String,
) -> Result<Monitor, CmdErr>
{
    let (send, recv) = channel();

    let thread_handle = thread::spawn(move || supervise(cfg, recv, app_path));

    let monitor = Monitor::new(thread_handle, send);

    Ok(monitor)
}

impl Channel {
    pub fn send(&self, msg: Msg) -> Result<(), ()> {
        self.send_msg.send(msg).map_err(|_| ())
    }

    pub fn recv(&self) -> Result<Option<Msg>, ()> {
        let timeout = Duration::new(1, 0);

        match self.recv_msg.recv_timeout(timeout) {
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Ok(msg) => Ok(Some(msg)),
            Err(_)  => Err(())
        }
    }
}

fn log<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
    Err(CmdErr::FailToRun)
}

fn supervise(cfg: Config, chan: Channel, app_path: String) {
    let file_backend = backend::File {
        filename: ".state.json".to_string(),
    };

    let mut state_file = match file_backend.load() {
        Ok(statefile) => statefile,

        Err(backend::PersistErr::IO(err)) => state::StateFile::new(),

        Err(encode_err) => {
            let error = CmdErr::PersistError(backend::PersistErr::EncodeError(encode_err));
            chan.send(Msg::Event(Event::Error(error);

            return;
        }
    };

    let last_state = state_file
        .latest_record()
        .map(Clone::clone)
        .unwrap_or(state::StateRecord::empty());

    let encoded_state = serde_json::to_string(&last_state)
        .expect("Failed to deserialize last recorded state");

    let mut stdout = Command::new(&app_path)
        .args(&[ encoded_state ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();

    let mut last_write_encountered = Some(Utc::now());

    loop {
        let did_timeout = last_write_encountered
            .map(|t| Utc::now().timestamp_millis() - t.timestamp_millis() > 300)
            .unwrap_or(false);

        if did_timeout {
            break;
        }

        match serde_json::from_reader(&mut stdout) {
            Ok(current_state) => {
                last_write_encountered = last_write_encountered.map(|_| Utc::now());

                if let Err(err) = file_backend.record(current_state) {
                    let error = CmdErr::PersistError(err);
                    chan.send(Msg::Event(Event::Error(error)));
                }
            },
            Err(err) => {
                if err.classify() != DecodingErrorCategory::Eof {
                    let error = CmdErr::UnexpectedOutput(err);
                    chan.send(Msg::Event(Event::Error(error)));
                }
            },
        }

        let sleep_nano_sec = last_write_encountered
            .map(|t| (Utc::now().timestamp_millis() - t.timestamp_millis()) as u32 / 20)
            .unwrap_or(0);

        thread::sleep(Duration::new(0, sleep_nano_sec));
    }
    
    println!("Application terminated");
    chan.send(Msg::Event(Event::ApplicationTerminated));
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::ApplicationTerminated => write!(f, "application terminated"),
            Event::ApplicationRestarted  => write!(f, "application restarted"),
            Event::StateRecorded         => write!(f, "state recorded"),
            Event::LogRecorded           => write!(f, "log recorded"),
            Event::Error(ref err)        => write!(f, "error: {}", err),
        }
    }
}

impl fmt::Display for CmdErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmdErr::UnknownCommand          => write!(f, "unknown command"),
            CmdErr::FailToRun               => write!(f, "failed to run the application specified"),
            CmdErr::UnexpectedOutput(ref e) => write!(f, "unexpected output received from application: {}", e),
            CmdErr::SupervisorCrashed       => write!(f, "supervisor managing application crashed"),
            CmdErr::PersistError(ref pe)    => write!(f, "fail to record: {}", pe),
        }
    }
}

impl Error for CmdErr {
    fn description(&self) -> &str {
        match *self {
            CmdErr::UnknownCommand      => "unknown command",
            CmdErr::FailToRun           => "failed to run the application specified",
            CmdErr::UnexpectedOutput(_) => "unexpected output received from application",
            CmdErr::SupervisorCrashed   => "supervisor managing application crashed",
            CmdErr::PersistError(_)     => "failed to record state",
        }
    }
}
