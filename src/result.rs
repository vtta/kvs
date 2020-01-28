use crate::error::Error;

/// use Error defined in this crate as default
pub type Result<T> = std::result::Result<T, Error>;
