#![deny(missing_docs)]
// #![feature(backtrace)]

//! A key-value store

mod error;
mod kv;

pub use error::{Error, Result};
pub use kv::KvStore;
