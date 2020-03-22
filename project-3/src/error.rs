use failure::Fail;

#[derive(Fail, Debug)]
pub enum KvsError {
    #[fail(display = "key not found")]
    KeyNotFound,
}

pub type Result<T> = std::result::Result<T, KvsError>;
