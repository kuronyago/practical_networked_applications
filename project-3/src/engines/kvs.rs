use std::path::{Path, PathBuf};

use crate::{KvsError, Result};

pub struct KvStore {}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        unimplemented!()
    }
}
