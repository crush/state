use std::fs::File;
use std::io::Error as IoError;

use chrono::{DateTime, Utc};
use serde_json::json;
use serde_json::value::Value as JsonValue;


pub type State = JsonValue;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Supervisor {
    #[serde(rename = "state")]
    State,
}

#[derive(Serialize)]
pub struct ApplicationInput {
    pub state: State,
    pub supervisor: Supervisor,
}

#[derive(Deserialize, Serialize)]
pub enum Event {
    #[serde(rename = "applicationTerminated")]
    ApplicationTerminated,
    #[serde(rename = "signalReceived")]
    SignalReceived(u8),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StateRecord {
    pub recorded_at: DateTime<Utc>,
    pub state: State,
}

#[derive(Deserialize, Serialize)]
pub struct LogRecord {
    pub recorded_at: DateTime<Utc>,
    pub event: Option<Event>,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct StateFile {
    pub states: Vec<StateRecord>,
    pub logs: Vec<LogRecord>,
}

impl StateRecord {
    pub fn empty() -> Self {
        StateRecord {
            recorded_at: Utc::now(),
            state: json!({}),
        }
    }
}

impl StateFile {
    pub fn latest_record(&mut self) -> Option<&StateRecord> {
        self.states.sort_by_key(|state| state.recorded_at);

        self.states.iter().last()
    }
}
