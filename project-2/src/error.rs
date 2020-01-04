use std::io;

#[derive(Debug)]
pub enum KvStoreError {
    IO(io::Error),
    KeyNotFound,
}

impl From<io::Error> for KvStoreError {
    fn from(err: io::Error) -> Self {
        KvStoreError::IO(err)
    }
}

pub type Result<T> = std::result::Result<T, KvStoreError>;
