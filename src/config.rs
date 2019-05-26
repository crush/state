use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Error as IoError;

use serde_json::{
    json,
    error::Error as EncodingError,
    value::Value as JsonValue,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub state_file: String,
}

#[derive(Debug)]
pub enum CfgErr {
    FailToLoad(IoError),
    Invalid(EncodingError),
}

impl Config {
    pub fn load(filename: &str) -> Result<Self, CfgErr> {
        let mut f = File::open(filename).map_err(CfgErr::FailToLoad)?;

        serde_json::from_reader(&mut f).map_err(CfgErr::Invalid)
    }

    pub fn save(&self, filename: &str) -> Result<(), CfgErr> {
        let mut f = File::create(filename).map_err(CfgErr::FailToLoad)?;

        serde_json::to_writer(&mut f, &self).map_err(CfgErr::Invalid)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config{
            state_file: ".state.json".to_string(),
        }
    }
}

impl fmt::Display for CfgErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CfgErr::FailToLoad(ref e) => write!(f, "failed to load configuration: {}", e),
            CfgErr::Invalid(ref e)    => write!(f, "configuration format is invalid: {}", e),
        }
    }
}

impl Error for CfgErr {
    fn description(&self) -> &str {
        match *self {
            CfgErr::FailToLoad(_) => "failed to load configuration",
            CfgErr::Invalid(_)    => "configuration format is invalid",
        }
    }
}
