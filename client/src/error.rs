use crate::request::Request;
use jute::JuteError;
use std::io;
use std::sync::mpsc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("corrupted response")]
    CorruptedResponse,

    #[error("empty response body")]
    EmptyResponseBody,

    #[error("invalid config")]
    InvalidConfig,

    #[error("anyhow error, {0}")]
    UnknownError(anyhow::Error),

    #[error("request send failed, {0}")]
    SendError(#[from] mpsc::SendError<Request>),

    #[error("response recv failed, {0}")]
    RecvError(#[from] mpsc::RecvError),

    #[error("jute error: {0}")]
    JuteError(#[from] JuteError),

    #[error("session expired")]
    SessionExpired,

    #[error("io error, {0}")]
    IoError(#[from] io::Error),

    #[error("timeout {0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),
}
