#![deny(missing_docs)]
#![feature(with_options)]

//! A key-value store

pub use error::{Error, ErrorKind, Result};
pub use kv::KvStore;
pub use resp::Resp;

mod config;
mod error;
mod kv;
mod log;
mod resp;
