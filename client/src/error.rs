use crate::request::Request;
use jute::JuteError;
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

    #[error("unknown error, {0}")]
    UnknownError(anyhow::Error),

    #[error("request send failed, {0}")]
    SendError(#[from] mpsc::SendError<Request>),

    #[error("response recv failed, {0}")]
    RecvError(#[from] mpsc::RecvError),

    #[error("{0}")]
    JuteError(#[from] JuteError),
}
