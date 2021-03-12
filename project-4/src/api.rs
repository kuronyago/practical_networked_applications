use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response<T> {
    Ok(T),
    Err(String),
}
