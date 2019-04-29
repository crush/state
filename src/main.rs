#[macro_use] extern crate serde_derive;

use chrono::{DateTime, Utc};
use clap::{App, Arg, SubCommand};
use serde_json::value::Value as JsonValue;


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
                Arg::with_name("file")
                    .help("A path to a file to write states to")
                    .short("f")
                    .long("file")
                    .default_value(".state.json"))
            .subcommand(
                SubCommand::with_name("run")
                    .about("Run an application with state management")
                    .arg(
                        Arg::with_name("application")
                            .help("Application to run, such as examples/example1.py")
                            .required(true)))
            .get_matches();

    match Command::execute(args) {
        Ok(monitor) => println!("Success"),
        Err(err)    => println!("Error!"),
    }
}
