use std::io::{Seek, SeekFrom};

use crate::error::Error;
use crate::kvs::index::Index;
use crate::kvs::log::Log;
use crate::kvs::record::Record;
use crate::result::Result;

mod index;
mod log;
mod record;

/// A simple on-disk key/value store that can be modified and queried from the command line.
///
#[derive(Debug)]
pub struct KvStore {
    index: index::Index,
    log: log::Log,
}

impl KvStore {
    /// Set the value of a string key to a string.
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.log.seek(SeekFrom::End(0))?;
        self.index.set(key.clone(), pos);
        self.log.write(&Record::Set(key, value))?;
        Ok(())
    }

    /// Get the string value of a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(pos) = self.index.get(key) {
            self.log.seek(SeekFrom::Start(pos))?;
            if let Record::Set(_, v) = self.log.read()? {
                Ok(Some(v))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Remove a given key.
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        let _pos = self
            .index
            .get(key.clone())
            .and_then(|_| self.index.remove(key.clone()))
            .ok_or_else(|| Error::KeyNotFound(key.clone()))?;
        self.log.seek(SeekFrom::End(0))?;
        self.log.write(&Record::Rm(key))
    }

    /// Open the KvStore at a given path.
    ///
    /// Return the KvStore.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let kvs = KvStore {
            index: Index::open(path.clone())?,
            log: Log::open(path)?,
        };
        Ok(kvs)
    }
}
