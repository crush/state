use std::default::Default;

use chrono::{DateTime, Utc};
use serde_json::value::Value as JsonValue;

use crate::config::{CfgErr, Config};


#[derive(Serialize)]
#[serde(untagged)]
pub enum Supervisor {
    #[serde(rename = "state")]
    State,
}

#[derive(Serialize)]
pub struct ApplicationInput {
    pub config: JsonValue,
    pub state: JsonValue,
    pub supervisor: Supervisor,
}

#[derive(Deserialize, Serialize)]
pub enum Event {
    #[serde(rename = "applicationTerminated")]
    ApplicationTerminated,
    #[serde(rename = "signalReceived")]
    SignalReceived(u8),
}

#[derive(Deserialize, Serialize)]
pub struct StateRecord {
    pub recorded_at: DateTime<Utc>,
    pub config: JsonValue,
    pub state: JsonValue,
}

#[derive(Deserialize, Serialize)]
pub struct LogRecord {
    pub recorded_at: DateTime<Utc>,
    pub event: Event,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct StateFile {
    pub states: Vec<StateRecord>,
    pub logs: Vec<LogRecord>,
}


