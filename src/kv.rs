use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

use crate::error::{Error, ErrorKind, Result};
use crate::log;
use crate::log::Segment;

/// A simple key-value store implementation which wraps around std `HashMap`
///
/// Key-value pairs are stored in a `HashMap` which means it's not durable and persistent
///
/// Example:
///
/// ```rust
/// ```
#[derive(Debug)]
pub struct KvStore {
    /// the directory that contains database files
    full_path: PathBuf,
    /// active database segment
    active: Segment,
    /// index
    memtbl: MemTable,
}

/// in memory representation of the index
#[derive(Debug)]
struct MemTable {
    map: HashMap<String, log::Pointer>,
}

impl KvStore {
    /// Open the KvStore at a given path. Return the KvStore.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        let mut entries = Vec::new();
        // scan through all the log files
        for res in fs::read_dir(&dir)? {
            let entry = res?;
            let path = entry.path();
            if path.is_file() && path.extension() == Some(&OsStr::new("log")) {
                entries.push(path);
            }
        }
        if entries.is_empty() {
            return Self::new(dir);
        }

        // log files are created by time order, which should be in ascending order
        entries.sort();
        let mut memtbl = MemTable::default();
        for entry in entries {
            let active = Segment::open(entry)?;
            for key in active.hint().count().keys() {
                if let Some(offset) = active.hint().offset().get(key) {
                    let pointer = log::Pointer::new(active.path(), *offset);
                    memtbl.map.insert(key.clone(), pointer);
                } else {
                    memtbl.map.remove(key);
                }
            }
        }
        let active = Segment::new(dir.clone())?;
        Ok(Self {
            full_path: dir,
            active,
            memtbl,
        })
    }

    fn new(dir: impl Into<PathBuf>) -> Result<Self> {
        let full_path = dir.into();
        let active = Segment::new(full_path.clone())?;
        Ok(Self {
            full_path,
            active,
            memtbl: MemTable::default(),
        })
    }

    /// Set the value of a string key to a string
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pointer = self.active.set(key.clone(), value)?;
        self.memtbl.map.insert(key, pointer);

        Ok(())
    }

    /// Get the string value of the a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.memtbl.map.get(&key) {
            None => Ok(None),
            Some(pointer) => {
                let file = pointer.path();
                if file == self.active.path() {
                    Ok(self.active.get(&key)?)
                } else {
                    let mut file = fs::File::with_options().read(true).open(file)?;
                    file.seek(SeekFrom::Start(pointer.offset()))?;
                    let entry: log::Entry = bincode::deserialize_from(file)?;
                    if let log::Entry::Set(_, v) = entry {
                        Ok(Some(v))
                    } else {
                        Err(Error::from(ErrorKind::InvalidLogEntry))
                    }
                }
            }
        }
    }

    /// Remove a given key.
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.memtbl.map.get(&key) {
            None => Err(Error::from(ErrorKind::KeyNotExist)),
            Some(_) => {
                self.active.remove(&key)?;
                self.memtbl.map.remove(&key);
                Ok(())
            }
        }
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
