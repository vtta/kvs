use std::collections::HashMap;

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
    /// Default ctor, init the underlying HashMap
    pub fn new() -> Self {
        KvStore { m: HashMap::new() }
    }

    /// Set the value of a string key to a fstring
    ///
    /// If a value is already associated with the given key, the value would be overridden.
    pub fn set(&mut self, key: String, value: String) {
        self.m.insert(key, value);
    }

    /// Get the string value of the a string key.
    ///
    /// If the key does not exist, a `None` would be returned.
    pub fn get(&mut self, key: String) -> Option<String> {
        self.m.get(&key).cloned()
    }

    /// Remove a given key.
    ///
    /// If nothing is there, would simply fall through.
    pub fn remove(&mut self, key: String) {
        self.m.remove(&key);
    }
}
