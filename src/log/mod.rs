use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::{Error, ErrorKind, Result};

#[cfg(test)]
mod tests;

/// a single log entry
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Entry {
    Set(String, String),
    Rm(String),
}

/// full path and file offset of a log entry
#[derive(Debug)]
pub(crate) struct Pointer {
    filename: PathBuf,
    offset: u64,
}

/// index for a log file
/// the on disk hint file contains `offset` and `count` back to back
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Hint {
    full_path: PathBuf,
    offset: HashMap<String, u64>,
    count: HashMap<String, u64>,
}

/// a log file
#[derive(Debug)]
pub(crate) struct Segment {
    full_path: PathBuf,
    hint: Hint,
    reader: BufReader<fs::File>,
    writer: BufWriter<fs::File>,
    write_offset: u64,
}

impl Segment {
    pub fn hint(&self) -> &Hint {
        &self.hint
    }
    pub fn path(&self) -> &PathBuf {
        &self.full_path
    }

    pub fn gen_name() -> String {
        format!("{}", chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S-%f"))
    }

    pub fn open(file: impl Into<PathBuf>) -> Result<Self> {
        let mut full_path = file.into();
        full_path.set_extension(LOG_FILE_EXT);
        let hint = match Hint::open(&full_path) {
            Ok(hint) => hint,
            Err(e) => match e.kind() {
                // TODO: rebuild hint file
                ErrorKind::InvalidHintFile => Hint::new(&full_path),
                _ => return Err(e),
            },
        };
        // create must be used with write/append
        let mut writer = BufWriter::new(
            fs::File::with_options()
                .append(true)
                .create(true)
                .open(&full_path)?,
        );
        let write_offset = writer.seek(SeekFrom::End(0))?;
        let reader = BufReader::new(fs::File::with_options().read(true).open(&full_path)?);

        Ok(Self {
            full_path,
            hint,
            reader,
            writer,
            write_offset,
        })
    }

    pub fn new(dir: impl Into<PathBuf>) -> Result<Self> {
        let mut full_path = dir.into();
        full_path.push(Self::gen_name());
        full_path.set_extension(LOG_FILE_EXT);
        let hint = Hint::new(&full_path);
        // create must be used with write/append
        let writer = BufWriter::new(
            fs::File::with_options()
                .write(true)
                .create_new(true)
                .open(&full_path)?,
        );
        let write_offset = 0;
        let reader = BufReader::new(fs::File::with_options().read(true).open(&full_path)?);

        Ok(Self {
            full_path,
            hint,
            reader,
            writer,
            write_offset,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<Pointer> {
        let pointer = Pointer::new(&self.full_path, self.write_offset);
        let entry = Entry::Set(key.clone(), value);
        let buf = bincode::serialize(&entry)?;
        self.writer.write_all(&buf)?;
        self.write_offset += buf.len() as u64;
        self.hint.set(key, pointer.offset);

        Ok(pointer)
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        // let pointer = Pointer::new(&self.full_path, self.write_offset);
        let entry = Entry::Rm(key.into());
        let buf = bincode::serialize(&entry)?;
        self.writer.write_all(&buf)?;
        self.write_offset += buf.len() as u64;
        self.hint.remove(key);

        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        if let Some(offset) = self.hint.get(key) {
            self.writer.flush()?;
            self.reader.seek(SeekFrom::Start(offset))?;
            let value: Entry = bincode::deserialize_from(&mut self.reader)?;
            if let Entry::Set(_, v) = value {
                Ok(Some(v))
            } else {
                Err(Error::from(ErrorKind::InvalidLogEntry))
            }
        } else {
            Ok(None)
        }
    }

    pub fn size(&self) -> u64 {
        self.write_offset
    }

    pub fn flush(&self) -> Result<()> {
        self.hint.flush()
    }
}

impl Hint {
    // TODO: ignore hint file when reading or deserialization fails
    pub fn open(file: impl Into<PathBuf>) -> Result<Self> {
        let mut full_path = file.into();
        full_path.set_extension(HINT_FILE_EXT);
        let file = fs::File::with_options()
            .read(true)
            .open(full_path.clone())
            .map_err(|_| Error::from(ErrorKind::InvalidHintFile))?;
        let mut hint: Hint =
            bincode::deserialize_from(file).map_err(|_| Error::from(ErrorKind::InvalidHintFile))?;
        hint.full_path = full_path;
        Ok(hint)
    }

    /// create an empty hint file in memory
    pub fn new(file: impl Into<PathBuf>) -> Self {
        let mut full_path = file.into();
        full_path.set_extension(HINT_FILE_EXT);
        Self {
            full_path,
            offset: HashMap::new(),
            count: HashMap::new(),
        }
    }

    /// change the offset corresponding to given key
    pub fn set(&mut self, key: String, offset: u64) {
        self.offset
            .entry(key.clone())
            .and_modify(|v| *v = offset)
            .or_insert(offset);
        self.count.entry(key).and_modify(|v| *v += 1).or_insert(1);
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        self.offset.get(key).copied()
    }

    /// remove the given key in hint file
    pub fn remove(&mut self, key: &str) {
        self.offset.remove(key);
        self.count
            .entry(key.into())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    // pub fn path(&self) -> &PathBuf {
    //     &self.full_path
    // }

    pub fn offset(&self) -> &HashMap<String, u64> {
        &self.offset
    }

    pub fn count(&self) -> &HashMap<String, u64> {
        &self.count
    }

    /// flush hint file to disk
    /// every flush would override previous result
    pub fn flush(&self) -> Result<()> {
        let _ = fs::File::with_options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.full_path)
            .map_err(Error::from)
            .and_then(|file| {
                bincode::serialize_into(&file, &self)
                    .map(|_| file)
                    .map_err(Error::from)
            })?;
        Ok(())
    }
}

impl Drop for Hint {
    fn drop(&mut self) {
        self.flush()
            .unwrap_or_else(|_| panic!("error while writing back hint file: {:?}", self.full_path));
    }
}

impl Pointer {
    pub fn new(filename: impl Into<PathBuf>, offset: u64) -> Self {
        Self {
            filename: filename.into(),
            offset,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.filename
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}
