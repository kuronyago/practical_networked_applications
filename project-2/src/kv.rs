use super::Result;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

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

struct Writer<T: Write + Seek> {
    buffer: BufWriter<T>,
    position: u64,
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        unimplemented!()
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        unimplemented!()
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
    unimplemented!()
}

fn log_path(dir: &Path, generation: u64) -> PathBuf {
    unimplemented!()
}

fn load(
    generation: u64,
    reader: &mut Reader<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    unimplemented!();
}

fn new_log(
    path: &Path,
    generation: u64,
    readers: &mut HashMap<u64, Reader<File>>,
) -> Result<Writer<File>> {
    unimplemented!()
}

#[derive(Serialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

struct CommandPos {
    generation: u64,
    position: u64,
    length: u64,
}
