use crate::kvs::record::Record;
use crate::result::Result;

use crate::config::DB_LOG_FILE_NAME;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Log {
    path: String,
    handle: std::fs::File,
}

impl Log {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut path = path.into();
        path.push(DB_LOG_FILE_NAME);
        // unwrap would definitely success because we pushed a filename just now
        let path = path.to_str().unwrap().to_owned();
        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let log = Log { path, handle };
        Ok(log)
    }

    pub fn read(&mut self) -> Result<Record> {
        let de = bson::decode_document(&mut self.handle)?;
        let record: Record = bson::from_bson(bson::Bson::from(de))?;
        Ok(record)
    }

    pub fn write(&mut self, record: &Record) -> Result<()> {
        let ser = bson::to_bson(&record)?;
        if let bson::Bson::Document(doc) = ser {
            bson::encode_document(&mut self.handle, &doc)?;
        };
        Ok(())
    }
}

impl Seek for Log {
    fn seek(&mut self, pos: SeekFrom) -> std::result::Result<u64, std::io::Error> {
        self.handle.seek(pos)
    }
}
