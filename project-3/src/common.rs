use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}
