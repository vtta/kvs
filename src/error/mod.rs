//! User defined error type
//! Adopted the Error and ErrorKind pattern

use std::{error, fmt, io, result};

use serde::export::Formatter;

/// Use Error in this crate as default Error type in Result
pub type Result<T> = result::Result<T, Error>;

/// Custom Error type
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    error: Option<Box<dyn error::Error + Send + Sync>>,
}

/// possible type of error
#[derive(Debug)]
pub enum ErrorKind {
    /// originated from std::io::Error
    Io,
    /// originated while serializing or deserializing data
    Serde,
    /// corrupted hint file
    InvalidHintFile,
    /// corrupted log entry
    InvalidLogEntry,
    /// the key to remove doesn't exist
    KeyNotExist,
}

impl Error {
    /// get the underlying error type
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match self {
            ErrorKind::Io => "I/O error",
            ErrorKind::Serde => "serialization or serialization error",
            ErrorKind::InvalidHintFile => "invalid hint file",
            ErrorKind::InvalidLogEntry => "invalid log entry",
            ErrorKind::KeyNotExist => "key not exist",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref e) = self.error {
            write!(f, "{}\tCaused by: {}", self.kind.as_str(), e)
        } else {
            write!(f, "{}", self.kind.as_str())
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Some(ref e) = self.error {
            e.source()
        } else {
            None
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind, error: None }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            kind: ErrorKind::Io,
            error: Some(e.into()),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error {
            kind: ErrorKind::Serde,
            error: Some(e.into()),
        }
    }
}
