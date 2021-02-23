use super::{KvStoreError, Result};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::ffi::OsStr;

pub struct KvStore {
    path: PathBuf,
    readers: HashMap<u64, Reader<File>>,
    writer: Writer<File>,
    current: u64,
    index: BTreeMap<String, CommandPos>,
    uncompacted: u64,
}

struct Reader<T: Read + Seek> {
    buffer: BufReader<T>,
    position: u64,
}

impl<T: Read + Seek> Reader<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position: u64 = inner.seek(SeekFrom::Current(0))?;

        Ok(Reader {
            buffer: BufReader::new(inner),
            position,
        })
    }
}

impl<T: Read + Seek> Read for Reader<T> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let length: usize = self.buffer.read(buffer)?;
        self.position += length as u64;
        Ok(length)
    }
}

impl<T: Read + Seek> Seek for Reader<T> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = self.buffer.seek(position)?;
        Ok(self.position)
    }
}

struct Writer<T: Write + Seek> {
    buffer: BufWriter<T>,
    position: u64,
}

impl<T: Write + Seek> Writer<T> {
    fn new(mut inner: T) -> Result<Self> {
        let position: u64 = inner.seek(SeekFrom::Current(0))?;

        Ok(Writer {
            buffer: BufWriter::new(inner),
            position,
        })
    }
}

impl<T: Write + Seek> Write for Writer<T> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length: usize = self.buffer.write(buffer)?;

        self.position += length as u64;
        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<T: Write + Seek> Seek for Writer<T> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = self.buffer.seek(position)?;
        Ok(self.position)
    }
}

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let position = self.writer.position;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current, position..self.writer.position).into())
            {
                self.uncompacted += old_cmd.length;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        let compaction_generation: u64 = self.current + 1;

        self.current += 2;

        self.writer = new_log(&self.path, self.current, &mut self.readers)?;

        let mut compaction_writer = new_log(&self.path, compaction_generation, &mut self.readers)?;

        let mut new_position = 0;

        for cmd_position in &mut self.index.values_mut() {
            let reader = self
                .readers
                .get_mut(&cmd_position.generation)
                .expect("Cannot find log reader");

            if reader.position != cmd_position.position {
                reader.seek(SeekFrom::Start(cmd_position.position))?;
            }

            let mut entry_reader = reader.take(cmd_position.length);

            let length = io::copy(&mut entry_reader, &mut compaction_writer)?;

            *cmd_position = (compaction_generation, new_position..new_position + length).into();
            new_position += length;
        }

        compaction_writer.flush()?;

        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_generation)
            .cloned()
            .collect();

        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }

        self.uncompacted = 0;

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_position) = self.index.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_position.generation)
                .expect("Cannot find log reader");

            reader.seek(SeekFrom::Start(cmd_position.position))?;

            let cmd_reader = reader.take(cmd_position.length);

            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvStoreError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;

            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("Key not found");
                self.uncompacted += old_cmd.length;
            }

            Ok(())
        } else {
            Err(KvStoreError::KeyNotFound)
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers: HashMap<u64, Reader<File>> = HashMap::new();
        let mut index: BTreeMap<String, CommandPos> = BTreeMap::new();

        let genenerations: Vec<u64> = sort_generation(&path)?;
        let mut uncompacted: u64 = 0;

        for &generation in &genenerations {
            let log: PathBuf = log_path(&path, generation);
            let file: File = File::open(&log)?;
            let mut reader: Reader<File> = Reader::new(file)?;

            uncompacted += load(generation, &mut reader, &mut index)?;
            readers.insert(generation, reader);
        }

        let current: u64 = genenerations.last().unwrap_or(&0) + 1;
        let writer: Writer<File> = new_log(&path, current, &mut readers)?;

        Ok(KvStore {
            path,
            readers,
            writer,
            current,
            index,
            uncompacted,
        })
    }
}

fn sort_generation(path: &Path) -> Result<Vec<u64>> {
    let mut generation_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .filter_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    generation_list.sort_unstable();

    Ok(generation_list)
}

fn log_path(dir: &Path, generation: u64) -> PathBuf {
    dir.join(format!("{}.log", generation))
}

fn load(
    generation: u64,
    reader: &mut Reader<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut position: u64 = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted: u64 = 0;

    while let Some(cmd) = stream.next() {
        let new_position = stream.byte_offset() as u64;

        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) =
                    index.insert(key, (generation, position..new_position).into())
                {
                    uncompacted += old_cmd.length;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.length;
                }

                uncompacted += new_position - position;
            }
        }

        position = new_position;
    }

    Ok(uncompacted)
}

fn new_log(
    path: &Path,
    generation: u64,
    readers: &mut HashMap<u64, Reader<File>>,
) -> Result<Writer<File>> {
    let path = log_path(&path, generation);

    let writer = Writer::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;

    readers.insert(generation, Reader::new(File::open(&path)?)?);

    Ok(writer)
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

struct CommandPos {
    generation: u64,
    position: u64,
    length: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((generation, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            generation,
            position: range.start,
            length: range.end - range.start,
        }
    }
}
