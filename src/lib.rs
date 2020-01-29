#![deny(missing_docs)]

//! A key-value store

/// several predefined configuration constant
pub mod config;
mod error;
mod kvs;
mod result;

mod bb2;

pub use crate::error::Error;
pub use crate::kvs::{KvStore, KvsCmd};
pub use crate::result::Result;
