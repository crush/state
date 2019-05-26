#[macro_use] extern crate serde_derive;

mod backend;
mod config;
mod commands;
mod state;

use clap::{App, Arg, SubCommand};

use crate::config::{CfgErr, Config};

const DEFAULT_CFG_FILENAME: &str = ".state.conf.json";


fn main() {
    let args =
        App::new("state")
            .version("0.1")
            .author("Crush")
            .about("Persist application state and handle restarts")
            .arg(
                Arg::with_name("config")
                    .help("A path to a file containing a configuration for state.")
                    .short("c")
                    .long("config")
                    .default_value(".state.conf.json"))
            .subcommand(
                SubCommand::with_name("run")
                    .about("Run an application with state management")
                    .arg(
                        Arg::with_name("application")
                            .help("Application to run, such as examples/example1.py")
                            .required(true)))
            .get_matches();


    let config_path = args
        .value_of("config")
        .unwrap_or(DEFAULT_CFG_FILENAME);

    let config = Config::load(config_path).unwrap_or(Config::default());

    match commands::Cmd::execute(config, &args) {
        Ok(monitor) => {
            let events = monitor.wait_for_termination().expect("Failed to run process");
            for event in events {
                println!("Event: {}", event);
            }
        },
        Err(err) => println!("Error! {}", err),
    }
}
