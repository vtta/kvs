use serde::{Deserialize, Serialize};

/// Logical representation of a log record
#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum Record {
    Rm(String),
    Set(String, String),
}
