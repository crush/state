#[macro_use] extern crate serde_derive;

mod backends;
mod config;
mod commands;

use std::default::Default;

use chrono::{DateTime, Utc};
use clap::{App, Arg, SubCommand};
use serde_json::value::Value as JsonValue;

use crate::config::{CfgErr, Config};


#[derive(Serialize)]
#[serde(untagged)]
enum Supervisor {
    #[serde(rename = "state")]
    State,
}

#[derive(Serialize)]
struct ApplicationInput {
    pub config: JsonValue,
    pub state: JsonValue,
    pub supervisor: Supervisor,
}

#[derive(Deserialize, Serialize)]
enum Event {
    #[serde(rename = "applicationTerminated")]
    ApplicationTerminated,
    #[serde(rename = "signalReceived")]
    SignalReceived(u8),
}

#[derive(Deserialize, Serialize)]
struct StateRecord {
    pub recorded_at: DateTime<Utc>,
    pub config: JsonValue,
    pub state: JsonValue,
}

#[derive(Deserialize, Serialize)]
struct LogRecord {
    pub recorded_at: DateTime<Utc>,
    pub event: Event,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
struct StateFile {
    pub states: Vec<StateRecord>,
    pub logs: Vec<LogRecord>,
}

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
        .unwrap_or(".state.conf.json");

    let config = match Config::load(config_path) {
        Ok(config)                    => config,
        Err(CfgErr::PermissionDenied) => return (),
        Err(_)                        => Config::default(),
    };

    match commands::Cmd::execute(config, &args) {
        Ok(monitor) => {
            println!("Success");
            monitor.wait_for_termination();
        }
        Err(err)    => println!("Error! {}", err),
    }
}
