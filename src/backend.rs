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
        let mut statefile = self.state_file();

        statefile.states.push(state::StateRecord {
            recorded_at: Utc::now(),
            config: json!({}),
            state: state,
        });

        let mut file = fs::File::create(&self.filename)
            .map_err(|_| PersistErr::PermissionError)?;

        serde_json::to_writer_pretty(&mut file, &statefile)
            .map_err(|_| PersistErr::PermissionError)
    }

    pub fn last_state(&self) -> Option<state::StateRecord> {
        let mut statefile = self.state_file();

        statefile.states.sort_by_key(|state| state.recorded_at);

        statefile.states.iter().last().map(Clone::clone)
    }

    fn state_file(&self) -> state::StateFile {
        let default = state::StateFile::new();

        match fs::File::open(&self.filename) {
            Err(_) =>
                default,

            Ok(ref mut f) =>
                serde_json::from_reader(f).unwrap_or(default),
        }
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
