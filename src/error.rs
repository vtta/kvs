use crate::kvs;
use bson;

/// custom Error type, which could be caused by std::io::Error and other things
#[derive(Debug)]
pub enum Error {
    /// error caused by I/O operations
    Io(std::io::Error),
    /// serialization error
    Ser(bson::EncoderError),
    /// deserialization error
    De(bson::DecoderError),
    /// triggered by remove a non-existing key/value pair
    InvalidCmd(kvs::KvsCmd),
    /// backend database file is missing
    File(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
impl From<bson::EncoderError> for Error {
    fn from(e: bson::EncoderError) -> Self {
        Error::Ser(e)
    }
}
impl From<bson::DecoderError> for Error {
    fn from(e: bson::DecoderError) -> Self {
        Error::De(e)
    }
}
impl From<kvs::KvsCmd> for Error {
    fn from(e: kvs::KvsCmd) -> Self {
        Error::InvalidCmd(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref e) => Some(e),
            _ => None,
        }
    }

    // fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
    //     match *self {
    //         Error::Io(ref e) => e.backtrace(),
    //         _ => None,
    //     }
    // }
}
