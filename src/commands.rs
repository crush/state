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
use crate::backends::{Backend, File};


/// Command dispatcher.
#[derive(Debug)]
pub struct Cmd;

#[derive(Debug)]
pub enum CmdErr {
    UnknownCommand,
    FailToRun,
    UnexpectedOutput,
}

#[derive(Debug)]
pub enum Event {
    ApplicationTerminated,
    ApplicationRestarted,
    StateRecorded,
    LogRecorded,
}

pub struct Monitor {
    event_queue: Arc<Mutex<Vec<Event>>>,
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
            event_queue: Arc::new(Mutex::new(vec![])),
            supervisor: thread_handle,
            to_supervisor: chan,
        }
    }

    pub fn events(&self) -> Option<Vec<Event>> {
        None
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

    let subprocess = Command::new(&app_path)
        .args(&["{\"state\": { \"count\": 0 } }"])
        .stdin(Stdio::inherit())
        //.stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| {
            println!("Error running app: {}", err);
            CmdErr::FailToRun
        })?;

    let (send, recv) = channel();

    let thread_handle = thread::spawn(move || supervise(subprocess, recv));

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

fn supervise(proc: ChildProcess, chan: Channel) {
    let mut stdout = proc.stdout.unwrap();

    loop {
        let state: Result<JsonValue, ()> = serde_json::from_reader(&mut stdout).map_err(|_| ());

        match state {
            Ok(current_state) => println!("Need to persist"),
            Err(_)            => println!("Didn't get a state object"),
        }
    }
    // Process output, parsing for JSON objects, writing state to backends.
    // Listen for signals
    // Watch for termination
    // Serve a monitor that can be polled
}

impl fmt::Display for CmdErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmdErr::UnknownCommand   => write!(f, "Unknown command."),
            CmdErr::FailToRun        => write!(f, "Failed to run the application specified."),
            CmdErr::UnexpectedOutput => write!(f, "Unexpected output received from application."),
        }
    }
}

impl Error for CmdErr {
    fn description(&self) -> &str {
        match *self {
            CmdErr::UnknownCommand   => "Unknown command.",
            CmdErr::FailToRun        => "Failed to run the application specified.",
            CmdErr::UnexpectedOutput => "Unexpected output received from application.",
        }
    }
}
