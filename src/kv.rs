use std::cell::RefCell;
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
    active: RefCell<Segment>,
    /// index
    memtbl: MemTable,
    /// use set_count to decide whether to perform compaction
    set_count: u64,
}

const COMPACTION_THRESHOLD: u64 = 8 * 1024;
const SEGMENT_SIZE_THRESHOLD: u64 = 4 * 1024;

/// in memory representation of the index
#[derive(Debug)]
struct MemTable {
    map: HashMap<String, log::Pointer>,
}

impl KvStore {
    /// Open the KvStore at a given path. Return the KvStore.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        let segments = Self::list_segments(&dir)?;
        if segments.is_empty() {
            return Self::new(dir);
        }

        let mut memtbl = MemTable::default();
        for seg in segments {
            let active = Segment::open(seg)?;
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
            active: RefCell::new(active),
            memtbl,
            set_count: 0,
        })
    }

    fn list_segments(dir: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut segments = Vec::new();
        // scan through all the log files
        for res in fs::read_dir(&dir)? {
            let entry = res?;
            let path = entry.path();
            if path.is_file() && path.extension() == Some(&OsStr::new("log")) {
                segments.push(path);
            }
        }
        // log files are created by time order, which should be in ascending order
        segments.sort();
        Ok(segments)
    }

    fn new(dir: impl Into<PathBuf>) -> Result<Self> {
        let full_path = dir.into();
        let active = Segment::new(full_path.clone())?;
        Ok(Self {
            full_path,
            active: RefCell::new(active),
            memtbl: MemTable::default(),
            set_count: 0,
        })
    }

    /// Set the value of a string key to a string
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.set_impl(key, value)?;
        self.set_count += 1;
        if self.set_count > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn set_impl(&mut self, key: String, value: String) -> Result<()> {
        let pointer = self.active.borrow_mut().set(key.clone(), value)?;
        self.memtbl.map.insert(key, pointer);
        Ok(())
    }

    /// Get the string value of the a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.memtbl.map.get(&key) {
            None => Ok(None),
            Some(pointer) => {
                let file = pointer.path();
                if file == self.active.borrow().path() {
                    Ok(self.active.borrow_mut().get(&key)?)
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
                self.active.borrow_mut().remove(&key)?;
                self.memtbl.map.remove(&key);
                Ok(())
            }
        }
    }

    fn compact(&mut self) -> Result<()> {
        let mut segments = Self::list_segments(&self.full_path)?;

        // place a useless placeholder, see the comment below for reasons
        let old = self.active.replace(Segment::new(&self.full_path)?);
        drop(old);
        self.set_count = 0;

        let mut store = Self::new(&self.full_path)?;
        for key in self.memtbl.map.keys() {
            if let Some(value) = self.get(key.to_owned())? {
                store.set_impl(key.to_owned(), value)?;
                if store.active.borrow().size() > SEGMENT_SIZE_THRESHOLD {
                    let old = store.active.replace(Segment::new(&self.full_path)?);
                    drop(old);
                }
            }
        }

        // the log file is strictly sorted according time order
        // need to be sure that the active segment is the last created
        segments.push(self.active.borrow().path().clone());
        let old = self.active.replace(Segment::new(&self.full_path)?);
        drop(old);
        std::mem::swap(&mut self.memtbl, &mut store.memtbl);
        drop(store);

        for mut file in segments {
            file.set_extension("log");
            fs::remove_file(&file)?;
            file.set_extension("hint");
            fs::remove_file(&file)?;
        }

        Ok(())
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
