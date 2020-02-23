use crate::Result;
use simplelog::*;
use std::fs;
use std::path::Path;

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
