use std::default::Default;
use std::error::Error;
use std::fmt;

use serde_json::json;


pub struct Config {
}

#[derive(Debug)]
pub enum CfgErr {
    PermissionDenied,
    FileNotFound,
}

impl Config {
    pub fn load(file_path: &str) -> Result<Config, CfgErr> {
        Err(CfgErr::FileNotFound)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
        }
    }
}

impl fmt::Display for CfgErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CfgErr::PermissionDenied => write!(f, "Could not open configuration file."),
            CfgErr::FileNotFound     => write!(f, "Configuration file not found."),
        }
    }
}

impl Error for CfgErr {
    fn description(&self) -> &str {
        match *self {
            CfgErr::PermissionDenied => "Could not open configuration file.",
            CfgErr::FileNotFound     => "Configuration file not found.",
        }
    }
}
