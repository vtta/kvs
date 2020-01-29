use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use crate::config::DB_INDEX_FILE_NAME;
use crate::result::Result;

/// In memory representation of the index structure
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Index {
    /// A HashMap maps given string key to the position of corresponding value in a file
    tbl: HashMap<String, u64>,
    path: String,
}

impl Index {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut path = path.into();
        path.push(DB_INDEX_FILE_NAME);
        // unwrap would definitely success because we pushed a filename just now
        let path = path.to_str().unwrap().to_owned();
        let idx_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let reader = BufReader::new(&idx_file);
        //let mut index: Index =
        let mut index: Index = ron::de::from_reader(reader).unwrap_or_else(|_| Index {
            tbl: HashMap::new(),
            path: String::new(),
        });
        index.path = path;
        Ok(index)
    }

    pub fn set(&mut self, key: String, value: u64) -> Option<u64> {
        self.tbl.insert(key, value)
    }

    pub fn get(&mut self, key: String) -> Option<u64> {
        self.tbl.get(&key).map(|v| v.to_owned())
    }

    pub fn remove(&mut self, key: String) -> Option<u64> {
        self.tbl.remove(&key)
    }
}

impl Drop for Index {
    fn drop(&mut self) {
        // cannot deal with error in this method
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .expect("error while opening index file for write")
            .write_all(
                ron::ser::to_string(self)
                    .expect("error while serializing index file")
                    .as_bytes(),
            )
            .expect("error while writing back index file");
    }
}
