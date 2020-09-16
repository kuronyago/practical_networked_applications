use failure::Fail;
use std::io::Error as ErrorIO;
use std::string::FromUtf8Error;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "io error: {}", _0)]
    IO(ErrorIO),
    #[fail(display = "serde error")]
    Serde(serde_json::Error),
    #[fail(display = "unexpected command type")]
    UnexpectedCommand,
    #[fail(display = "error: {}", _0)]
    WithMessage(String),
    #[fail(display = "error: {}", _0)]
    Sled(sled::Error),
    #[fail(display = "UTF-8 error: {}", _0)]
    UTF8(FromUtf8Error),
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

impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Error::Sled(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::UTF8(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
