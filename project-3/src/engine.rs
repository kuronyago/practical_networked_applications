use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

// pub use self::kvs::KvStore;

pub struct Store {
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    index: BTreeMap<String, CommandPosition>,
    uncompacted: u64,
    current_gen: u64,
}

impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<Store> {
        unimplemented!()
    }
}

pub trait Engine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}

impl Engine for Store {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd: Command = Command::Set { key, value };

        let pos: u64 = self.writer.pos;

        serde_json::to_writer(&mut self.writer, &cmd)?;

        self.writer.flush()?;

        let to_insert_value: CommandPosition = (self.current_gen, (pos..self.writer.pos)).into();

        if let Command::Set { key, .. } = cmd {
            if let Some(inserted) = self.index.insert(key, to_insert_value) {
                self.uncompacted += inserted.len;
            }
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd) = self.index.get(&key) {
            let reader = self.readers.get_mut(&cmd.gen).expect("some message");

            let pos = SeekFrom::Start(cmd.pos);

            let _start_from = reader.seek(pos)?;

            let handler = reader.take(cmd.pos);

            if let Command::Set { value, .. } = serde_json::from_reader(handler)? {
                Ok(Some(value))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}

struct BufWriterWithPos<T: Write + Seek> {
    writer: T,
    pos: u64,
}

struct BufReaderWithPos<T: Read + Seek> {
    reader: BufReader<T>,
    pos: u64,
}

impl<T: Write + Seek> Write for BufWriterWithPos<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }
    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl<T: Write + Seek> Seek for BufWriterWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

impl<T: Read + Seek> Seek for BufReaderWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

impl<T: Read + Seek> Read for BufReaderWithPos<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

struct CommandPosition {
    len: u64,
    gen: u64,
    pos: u64,
}

impl From<(u64, Range<u64>)> for CommandPosition {
    fn from(_: (u64, Range<u64>)) -> Self {
        todo!()
    }
}

#[derive(Deserialize, Serialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}
