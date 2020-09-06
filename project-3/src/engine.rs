use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use serde_json::Deserializer;

pub struct Store {
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    index: BTreeMap<String, CommandPosition>,
    uncompacted: u64,
    current_gen: u64,
}

const COPMACTION_THRESHOLD: u64 = 1024 * 1024;

impl Store {
    pub fn open(dir: impl Into<PathBuf>) -> Result<Store> {
        let path = dir.into();
        std::fs::create_dir_all(&path)?;

        let gen_list = sorted_gen_list(&path)?;

        let mut readers = HashMap::<u64, BufReaderWithPos<File>>::new();
        let mut index = BTreeMap::<String, CommandPosition>::new();

        let mut uncompacted: u64 = 0;

        for &gen in &gen_list {
            let file_path = log_path(&path, gen);
            let log_file = File::open(file_path)?;
            let mut log_reader = BufReaderWithPos::new(log_file)?;

            let uncompacted_log = load(gen, &mut log_reader, &mut index)?;

            uncompacted += uncompacted_log;
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;

        let writer = new_log_file(&path, current_gen, &mut readers)?;

        Ok(Store {
            path,
            readers,
            writer,
            index,
            uncompacted,
            current_gen,
        })
    }

    fn compact(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;

        self.writer = self.new_log_file(self.current_gen)?;
        let mut writer = self.new_log_file(compaction_gen)?;

        let mut new_compact_position: u64 = 0;
        for command in self.index.values_mut() {
            let reader = self.readers.get_mut(&command.pos).expect("msg");

            if command.pos != reader.pos {
                reader.seek(SeekFrom::Start(command.pos))?;
            }

            let mut reader = reader.take(command.len);

            let len: u64 = std::io::copy(&mut reader, &mut writer)?;

            let new_position_range = (
                compaction_gen,
                new_compact_position..new_compact_position + len,
            );

            let new_position: CommandPosition = new_position_range.into();

            *command = new_position;
            new_compact_position += len;
        }

        writer.flush()?;

        {
            let stales: Vec<u64> = self
                .readers
                .keys()
                .filter(|&&gen| gen > compaction_gen)
                .cloned()
                .collect();

            for stale in stales {
                self.readers.remove(&stale);
                let stale_log_path: PathBuf = log_path(&self.path, stale);
                std::fs::remove_dir(stale_log_path)?;
            }
        }

        self.uncompacted = 0;
        Ok(())
    }

    fn new_log_file(&mut self, gen: u64) -> Result<BufWriterWithPos<File>> {
        new_log_file(&self.path, gen, &mut self.readers)
    }
}

pub trait Engine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}

impl Engine for Store {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos: u64 = self.writer.pos;

        {
            serde_json::to_writer(
                &mut self.writer,
                &Command::Set {
                    key: key.clone(),
                    value,
                },
            )?;
            self.writer.flush()?;
        }

        let to_insert_value: CommandPosition = (self.current_gen, (pos..self.writer.pos)).into();
        if let Some(inserted) = self.index.insert(key, to_insert_value) {
            self.uncompacted += inserted.len;
        }

        if self.uncompacted > COPMACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd) = self.index.get(&key) {
            let reader = self.readers.get_mut(&cmd.gen).expect("log reade not found");
            let seek_position = SeekFrom::Start(cmd.pos);
            let _start_from = reader.seek(seek_position)?;

            let reader_handler = reader.take(cmd.pos);

            if let Command::Set { value, .. } = serde_json::from_reader(reader_handler)? {
                Ok(Some(value))
            } else {
                Err(Error::UnexpectedCommand)
            }
        } else {
            Ok(None)
        }
    }
    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            {
                let cmd = Command::Remove { key: key.clone() };
                serde_json::to_writer(&mut self.writer, &cmd)?;
            }
            self.writer.flush()?;

            let removed = self
                .index
                .remove(&key)
                .expect("key not found after index.contains_key");
            self.uncompacted += removed.len;
            Ok(())
        } else {
            Err(Error::KeyNotFound)
        }
    }
}

struct BufWriterWithPos<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> BufWriterWithPos<T> {
    fn new(mut inner: T) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;

        Ok(BufWriterWithPos {
            pos,
            writer: BufWriter::new(inner),
        })
    }
}

struct BufReaderWithPos<T: Read + Seek> {
    reader: BufReader<T>,
    pos: u64,
}

impl<T: Read + Seek> BufReaderWithPos<T> {
    fn new(mut inner: T) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;

        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<T: Write + Seek> Write for BufWriterWithPos<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;

        self.pos += len as u64;
        Ok(len)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<T: Write + Seek> Seek for BufWriterWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
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
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

struct CommandPosition {
    len: u64,
    gen: u64,
    pos: u64,
}

impl From<(u64, Range<u64>)> for CommandPosition {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPosition {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

#[derive(Deserialize, Serialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

fn sorted_gen_list<'a>(path: &'a Path) -> Result<Vec<u64>> {
    let dir: std::fs::ReadDir = std::fs::read_dir(path)?;

    let mut list: Vec<u64> = dir
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    list.sort_unstable();
    Ok(list)
}

fn new_log_file(
    path: &Path,
    gen: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let p = log_path(path, gen);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&p)?;
    let file_to_read = File::open(&p)?;
    let writer: BufWriterWithPos<File> = BufWriterWithPos::new(file)?;
    readers.insert(gen, BufReaderWithPos::new(file_to_read)?);
    Ok(writer)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn load(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPosition>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Current(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted: u64 = 0;

    while let Some(cmd) = stream.next() {
        let new_pos: u64 = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                let value: CommandPosition = (gen, pos..new_pos).into();
                if let Some(old) = index.insert(key, value) {
                    uncompacted += old.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old) = index.remove(&key) {
                    uncompacted += old.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}
