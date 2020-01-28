/// custom Error type, which could be caused by std::io::Error and other things
#[derive(Debug)]
pub enum Error {
    /// error caused by I/O operations
    Io(std::io::Error),
    /// placeholder
    Wtf,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
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
