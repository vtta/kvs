use std::path::PathBuf;

use crate::{KvsEngine, Result};

pub struct SledKvsEngine {
    db: sled::Db,
}

impl KvsEngine for SledKvsEngine {
    fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let db = sled::open(dir.into())?;
        Ok(Self { db })
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let val = self.db.get(key)?;
        match val {
            None => Ok(None),
            Some(iv) => Ok(Some(String::from_utf8(iv.to_vec())?)),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.db.remove(key)?;
        Ok(())
    }
}
