use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum JuteError {
    #[error("UTF-8 decode Error, buffer is not a valid string")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("End of buffer")]
    Eof,
}
