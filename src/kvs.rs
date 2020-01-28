use failure::Error;
use std::path::PathBuf;

/// Result type wraps around Error type from [failure](https://crates.io/crates/failure) crate
pub type Result<T> = std::result::Result<T, Error>;

/// A simple on-disk key/value store that can be modified and queried from the command line.
///
#[derive(Debug, Default)]
pub struct KvStore {}

impl KvStore {
    /// Set the value of a string key to a string.
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }

    /// Get the string value of a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }

    /// Remove a given key.
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }

    /// Open the KvStore at a given path.
    ///
    /// Return the KvStore.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        todo!()
    }
}
