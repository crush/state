use std::error::Error;
use std::fs;
use std::fmt;

use chrono::Utc;
use serde_json::json;
use serde_json::value::Value as JsonValue;

use crate::state;


#[derive(Debug)]
pub enum PersistErr {
    PermissionError,
}

pub struct File {
    pub filename: String
}

impl File {
    pub fn record_state(&self, state: JsonValue) -> Result<(), PersistErr> {
        let mut file = fs::File::create(&self.filename)
            .map_err(|_| PersistErr::PermissionError)?;

        let record = state::StateRecord {
            recorded_at: Utc::now(),
            config: json!({}),
            state: state,
        };

        serde_json::to_writer_pretty(&mut file, &record)
            .map_err(|_| PersistErr::PermissionError)
    }
}

impl fmt::Display for PersistErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PersistErr::PermissionError => write!(f, "Do not have file write permissions."),
        }
    }
}

impl Error for PersistErr {
    fn description(&self) -> &str {
        match *self {
            PersistErr::PermissionError => "Do not have file write permissions.",
        }
    }
}
