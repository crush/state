use std::error::Error;
use std::fmt;
use std::process::Command;
use std::sync::{Arc, Mutex};

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
    Terminated,
}

pub struct Monitor {
    event_queue: Arc<Mutex<Vec<Event>>>,
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
    pub fn events(&self) -> Option<Vec<Event>> {
        None
    }
}

fn run<'main>(
    cfg: Config,
    app_path: String,
) -> Result<Monitor, CmdErr>
{
    Err(CmdErr::FailToRun)
}

fn log<'main>(args: &clap::ArgMatches<'main>) -> Result<Monitor, CmdErr> {
    Err(CmdErr::FailToRun)
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;


    #[test]
    fn example1_counts_up() {

    }
}
