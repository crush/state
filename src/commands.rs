use std::sync::{Arc, Mutex};

/// Command dispatcher.
pub enum Command {
    Run(String),
}

pub enum CmdErr {
    UnknownCommand,
    FailToRun,
}

pub enum Event {
    Terminated,
}

pub struct Monitor {
    event_queue: Arc<Mutex<Vec<Event>>>,
}

impl Command {
    pub fn execute<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
        let subcmd_args = (
            args.subcommand_matches("run"),
            args.subcommand_matches("log"),
        );

        match subcmd_args {
            (
                Some(run_args),
                _,
            ) => run(run_args),
            (
                _,
                Some(log_args),
            ) => log(log_args),
            _ => Err(CmdErr::UnknownCommand),
        }
    }
}

impl Monitor {
    pub fn events(&self) -> Option<Vec<Event>> {
        None
    }
}

fn run<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
    Err(CmdErr::FailToRun)
}

fn log<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
    Err(CmdErr::FailToRun)
}
