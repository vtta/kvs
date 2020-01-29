use crate::error::Error;
use crate::result::Result;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// A simple on-disk key/value store that can be modified and queried from the command line.
///
#[derive(Debug, Default)]
pub struct KvStore {
    m: std::collections::HashMap<String, String>,
    path: String,
    handle: Option<std::fs::File>,
}

/// Possible commands to be executed
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub enum KvsCmd {
    /// Set the value of a string key to a string
    Set {
        /// key to be set
        #[structopt(name = "KEY")]
        key: String,
        /// value to be set
        #[structopt(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        /// key part of the pair to get
        #[structopt(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        /// key part of the pair to remove
        #[structopt(name = "KEY")]
        key: String,
    },
}

impl KvStore {
    // fn new() -> Self {
    //     KvStore {
    //         map: std::collections::HashMap::new(),
    //         handle: None,
    //     }
    // }

    // fn file_err(&self) -> Error {
    //     Error::File(self.path.clone())
    // }

    fn log_cmd(&self, cmd: KvsCmd) -> Result<()> {
        let ser = bson::to_bson(&cmd)?;
        if let bson::Bson::Document(doc) = ser {
            bson::encode_document(
                &mut self
                    .handle
                    .as_ref()
                    .ok_or_else(|| Error::File(self.path.clone()))?,
                &doc,
            )?;
        };
        Ok(())
    }

    /// Set the value of a string key to a string.
    ///
    /// Return an error if the value is not written successfully.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.m.insert(key.clone(), value.clone());
        self.log_cmd(KvsCmd::Set { key, value })?;
        Ok(())
    }

    /// Get the string value of a string key.
    ///
    /// If the key does not exist, return `None`.
    /// Return an error if the value is not read successfully.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self.m.get(&key).map(|v| v.to_owned()))
    }

    /// Remove a given key.
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.m.get(&key).is_some() {
            self.m.remove(&key);
            self.log_cmd(KvsCmd::Rm { key })?;
            return Ok(());
        }
        Err(Error::InvalidCmd(KvsCmd::Rm { key }))
    }

    /// Open the KvStore at a given path.
    ///
    /// Return the KvStore.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<KvStore> {
        let mut p = path.into();
        if p.is_dir() {
            p.push(crate::config::DB_FILE_NAME);
        }
        let mut kvs = KvStore {
            m: std::collections::HashMap::new(),
            path: p.to_str().unwrap_or_default().to_owned(),
            handle: Some(
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(p)?,
            ),
        };
        let db_file = kvs.handle.as_ref().unwrap();
        let mut reader = std::io::BufReader::new(db_file);
        // reader.seek(std::io::SeekFrom::Start(0));
        while let Ok(doc) = bson::decode_document(&mut reader) {
            let cmd: KvsCmd = bson::from_bson(bson::Bson::Document(doc))?;
            // println!("{:?}", cmd);
            match cmd {
                KvsCmd::Set { key, value } => {
                    kvs.m.insert(key, value);
                }
                KvsCmd::Rm { key } => {
                    kvs.m.remove(&key);
                }
                _ => (),
            }
        }
        Ok(kvs)
    }
}
