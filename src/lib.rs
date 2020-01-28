#![deny(missing_docs)]

//! A key-value store

mod error;
mod kvs;
mod result;

mod bb2;

pub use error::Error;
pub use kvs::KvStore;
pub use result::Result;
