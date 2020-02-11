use std::io;

#[derive(Debug)]
pub enum KvStoreError {
    IO(io::Error),
    Serde(serde_json::Error),
    KeyNotFound,
    UnexpectedCommandType,
}

impl From<io::Error> for KvStoreError {
    fn from(err: io::Error) -> Self {
        KvStoreError::IO(err)
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(err: serde_json::Error) -> KvStoreError {
        KvStoreError::Serde(err)
    }
}

pub type Result<T> = std::result::Result<T, KvStoreError>;
