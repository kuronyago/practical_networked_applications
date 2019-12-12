#[derive(Debug)]
pub enum KvStoreError {}

pub type Result<T> = std::result::Result<T, KvStoreError>;
