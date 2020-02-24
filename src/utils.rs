use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use simplelog::*;

use crate::Result;

/// helper to init the logger
pub fn logger(file: impl AsRef<Path>) -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Stderr).unwrap(),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            fs::File::create(file)?,
        ),
    ])?;
    Ok(())
}

/// command sent between client and server
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// get key
    Get(String),
    /// update key value pair
    Set(String, String),
    /// remove key value pair
    Rm(String),
}

/// respond from server
#[derive(Debug, Serialize, Deserialize)]
pub enum Respond {
    /// failed
    Err(String),
    /// the data is retrived
    Ok(Option<String>),
}
