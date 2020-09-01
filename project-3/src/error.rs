use failure::Fail;
use std::io::Error as ErrorIO;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "key not found")]
    KeyNotFound,
    #[fail(display = "io error")]
    IO(ErrorIO),
    #[fail(display = "serde error")]
    Serde(serde_json::Error),
    #[fail(display = "unexpected command type")]
    UnexpectedCommand,
}

impl From<ErrorIO> for Error {
    fn from(err: ErrorIO) -> Self {
        Error::IO(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
