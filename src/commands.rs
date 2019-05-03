use std::error::Error;
use std::fmt;
use std::process::{Child as ChildProcess, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{self, JoinHandle};

use crate::config::{CfgErr, Config};
use crate::backends::{Backend, File};


/// Command dispatcher.
#[derive(Debug)]
pub struct Cmd;

#[derive(Debug)]
pub enum CmdErr {
    UnknownCommand,
    FailToRun,
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
    sender: Sender<Msg>,
}

enum Msg {
    Kill,
    StillActive,
    Event(Event),
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
                let app_path = args
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
    fn new(thread_handle: JoinHandle<()>, send: Sender<Msg>) -> Self {
        Monitor {
            event_queue: Arc::new(Mutex::new(vec![])),
            supervisor: thread_handle,
            sender: send,
        }
    }

    pub fn events(&self) -> Option<Vec<Event>> {
        None
    }
}

fn run<'main>(
    cfg: Config,
    app_path: String,
) -> Result<Monitor, CmdErr>
{
    let subprocess = Command::new(&app_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| CmdErr::FailToRun)?;

    let (send, recv) = channel();

    let thread_handle = thread::spawn(move || supervise(subprocess, recv));

    let monitor = Monitor::new(thread_handle, send);

    Ok(monitor)
}

fn log<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
    Err(CmdErr::FailToRun)
}

fn supervise(proc: ChildProcess, recv: Receiver<Msg>) {
    // Process output, parsing for JSON objects, writing state to backends.
    // Listen for signals
    // Watch for termination
    // Serve a monitor that can be polled
}

impl fmt::Display for CmdErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmdErr::UnknownCommand => write!(f, "Unknown command."),
            CmdErr::FailToRun      => write!(f, "Failed to run the application specified."),
        }
    }
}

impl Error for CmdErr {
    fn description(&self) -> &str {
        match *self {
            CmdErr::UnknownCommand => "Unknown command.",
            CmdErr::FailToRun      => "Failed to run the application specified.",
        }
    }
}
