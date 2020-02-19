use std::{error, io, result};

use thiserror::Error;

/// Custom Error sum type
#[derive(Error, Debug)]
pub enum Error {
    /// I/O errors
    #[error(transparent)]
    Io(#[from] io::Error),

    ///Ser/De errors
    #[error(transparent)]
    Serde(#[from] bincode::Error),

    /// any other errors
    #[error(transparent)]
    Other(#[from] Box<dyn error::Error + Send + Sync>),
}

/// Use Error in this crate as default Error type in Result
pub type Result<T> = result::Result<T, Error>;
