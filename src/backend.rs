use std::error::Error;
use std::fs;
use std::fmt;
use std::io::Error as IoError;

use chrono::Utc;
use serde_json::{
    json,
    error::Error as EncodingError,
    value::Value as JsonValue,
};

use crate::state;


#[derive(Debug)]
pub enum PersistErr {
    EncodeError(EncodingError),
    IO(IoError),
}

pub struct File {
    pub filename: String
}

impl File {
    pub fn load(&self) -> Result<state::StateFile, PersistErr> {
        let mut f = fs::File::open(&self.filename).map_err(PersistErr::IO)?;

        serde_json::from_reader(&mut f).map_err(PersistErr::EncodeError)
    }

    pub fn record(&self, s: state::State) -> Result<(), PersistErr> {
        let mut statefile = self.load()?;

        statefile.states.push(state::StateRecord {
            recorded_at: Utc::now(),
            state: s,
        });

        self.write(&statefile)
    }

    pub fn log<S>(&self, msg: S, evt: Option<state::Event>) -> Result<(), PersistErr>
        where S: Into<String>
    {
        let mut statefile = self.load()?;

        statefile.logs.push(state::LogRecord {
            recorded_at: Utc::now(),
            event: evt,
            message: msg.into(),
        });

        self.write(&statefile)
    }

    fn write(&self, statefile: &state::StateFile) -> Result<(), PersistErr> {
        let mut f = fs::File::create(&self.filename).map_err(PersistErr::IO)?;

        serde_json::to_writer_pretty(&mut f, statefile).map_err(PersistErr::EncodeError)
    }
}

impl fmt::Display for PersistErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PersistErr::EncodeError(ref e) => write!(f, "invalid encoding: {}", e),
            PersistErr::IO(ref e)          => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for PersistErr {
    fn description(&self) -> &str {
        match *self {
            PersistErr::EncodeError(_) => "invalid encoding",
            PersistErr::IO(_)          => "IO error",
        }
    }
}
