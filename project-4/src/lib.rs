use anyhow::Result;
use thiserror::Error;

mod engines;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("key not found")]
    KeyNotFound,
    #[error("serialization failed")]
    Serde,
    #[error("unexpected command")]
    UnexpectedCommand,
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub trait Engine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> StoreResult<()>;
    fn get(&self, key: String) -> StoreResult<Option<String>>;
    fn remove(&self, key: String) -> StoreResult<()>;
}

pub struct ThreadSpawner;

impl ThreadSpawner {
    fn new(_threads: u32) -> StoreResult<Self> {
        Ok(ThreadSpawner)
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(f);
    }
}
