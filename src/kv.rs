use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// A simple key-value store implementation which wraps around std `HashMap`
///
/// Key-value pairs are stored in a `HashMap` which means it's not durable and persistent
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), Some("value1".to_owned()));
/// store.set("key2".to_owned(), "value2".to_owned());
/// assert_eq!(store.get("key2".to_owned()), Some("value2".to_owned()));
/// store.set("key1".to_owned(), "value2".to_owned());
/// assert_eq!(store.get("key1".to_owned()), Some("value2".to_owned()));
/// store.remove("key1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), None);
/// store.remove("key2".to_owned());
/// assert_eq!(store.get("key2".to_owned()), None);
/// store.remove("key2".to_owned());
/// assert_eq!(store.get("key2".to_owned()), None);
/// ```
#[derive(Debug, Default)]
pub struct KvStore {
    m: HashMap<String, String>,
}

impl KvStore {
    /// Open the KvStore at a given path. Return the KvStore.
    pub fn open(_path: impl Into<PathBuf>) -> Result<Self> {
        todo!()
    }

    /// Set the value of a string key to a fstring
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, _key: String, _value: String) -> Result<()> {
        todo!()
    }

    /// Get the string value of the a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, _key: String) -> Result<Option<String>> {
        todo!()
    }

    /// Remove a given key.
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, _key: String) -> Result<()> {
        todo!()
    }
}
