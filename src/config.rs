use std::default::Default;
use std::error::Error;
use std::fmt;

use serde_json::json;

use crate::backends::{self, Backend};


pub struct Config {
    backend_configs: Vec<serde_json::value::Value>,
}

#[derive(Debug)]
pub enum CfgErr {
    PermissionDenied,
}

impl Config {
    pub fn load(file_path: &str) -> Result<Config, CfgErr> {
        Err(CfgErr::PermissionDenied)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            backend_configs: vec![
                json!({ "filename": ".state.conf.json" }),
            ],
        }
    }
}

impl fmt::Display for CfgErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CfgErr::PermissionDenied => write!(f, "Could not open configuration file."),
        }
    }
}

impl Error for CfgErr {
    fn description(&self) -> &str {
        match *self {
            CfgErr::PermissionDenied => "Could not open configuration file.",
        }
    }
}
