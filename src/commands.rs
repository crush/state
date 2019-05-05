use std::collections::LinkedList as List;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::process::{Child as ChildProcess, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::value::Value as JsonValue;

use crate::config::{CfgErr, Config};
use crate::backend;


/// Command dispatcher.
#[derive(Debug)]
pub struct Cmd;

#[derive(Debug)]
pub enum CmdErr {
    UnknownCommand,
    FailToRun,
    UnexpectedOutput,
    SupervisorCrashed,
    FailToRecord(backend::PersistErr),
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
                println!("Trying to get application");

                let app_path = run_args
                    .value_of("application")
                    .ok_or(CmdErr::FailToRun)?;

                println!("Running");
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

    pub fn wait_for_termination(self) -> Result<(), CmdErr> {
        loop {
            match self.to_supervisor.recv() {
                Ok(Some(Msg::Event(Event::ApplicationTerminated))) => return Ok(()),
                Err(_) => return Err(CmdErr::SupervisorCrashed),
                Ok(_)  => (),
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
    println!("Starting subprocess");

    let (send, recv) = channel();

    let last_state = "{\"state\": {\"count\": 0}}";

    let thread_handle = thread::spawn(
        move || supervise(cfg, recv, app_path, last_state.to_string()));

    let monitor = Monitor::new(thread_handle, send);

    println!("Created monitor");

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

fn supervise(cfg: Config, chan: Channel, app_path: String, last_state: String) {
    println!("In call to supervise");
    let file_backend = backend::File {
        filename: ".state.json".to_string(),
    };

    let mut stdout = Command::new(&app_path)
        .args(&["{\"state\": { \"count\": 0 } }"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();

        /*
        .map_err(|err| {
            println!("Error running app: {}", err);
            CmdErr::FailToRun
        })?;
        */

    //loop {
        let state: Result<JsonValue, ()> = serde_json::from_reader(&mut stdout).map_err(|_| ());

        match state {
            Ok(current_state) => {
                if let Err(err) = file_backend.record_state(current_state) {
                    let error = CmdErr::FailToRecord(err);
                    chan.send(Msg::Event(Event::Error(error)));
                } else {
                    println!("Recorded state!");
                }
            },
            Err(_) => println!("Didn't get a state object"),
        }

        chan.send(Msg::Event(Event::ApplicationTerminated));

    //}
    // Process output, parsing for JSON objects, writing state to backends.
    // Listen for signals
    // Watch for termination
    // Serve a monitor that can be polled
}

impl fmt::Display for CmdErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmdErr::UnknownCommand        => write!(f, "Unknown command."),
            CmdErr::FailToRun             => write!(f, "Failed to run the application specified."),
            CmdErr::UnexpectedOutput      => write!(f, "Unexpected output received from application."),
            CmdErr::SupervisorCrashed     => write!(f, "Supervisor managing application crashed."),
            CmdErr::FailToRecord(ref pe)  => write!(f, "Fail to record: {}", pe),
        }
    }
}

impl Error for CmdErr {
    fn description(&self) -> &str {
        match *self {
            CmdErr::UnknownCommand    => "Unknown command.",
            CmdErr::FailToRun         => "Failed to run the application specified.",
            CmdErr::UnexpectedOutput  => "Unexpected output received from application.",
            CmdErr::SupervisorCrashed => "Supervisor managing application crashed.",
            CmdErr::FailToRecord(_)   => "Failed to record state.",
        }
    }
}
