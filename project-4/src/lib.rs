// use anyhow::Result;
// use thiserror::Error;

use slog::{o, Drain, Logger};

pub use api::{Request, Response};

mod api;
mod engines;
mod pool;
mod server;

pub type StoreResult<T> = anyhow::Result<T, StoreError>;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("key not found")]
    KeyNotFound,
    #[error("serialization failed")]
    Serde(#[from] serde_json::Error),
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

pub trait ThreadPool {
    fn new(threads: u32) -> StoreResult<Self>
    where
        Self: Sized;
    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static;
}

pub fn logger(module: &str) -> Logger {
    let drain = slog_json::Json::new(std::io::stdout())
        .set_pretty(false)
        .add_default_keys()
        .build()
        .fuse();

    let fuse = slog_async::Async::new(drain).build().fuse();

    Logger::root(fuse, o!("module" => module.to_owned()))
}
