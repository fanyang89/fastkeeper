use crate::request::Request;
use std::backtrace::Backtrace;
use std::io;
use std::net;
use std::sync::mpsc;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("corrupted response")]
    CorruptedResponse,

    #[error("empty body")]
    EmptyBody,

    #[error("session expired")]
    SessionExpired,

    #[error("empty config")]
    EmptyHosts,

    #[error("invalid config, {0}")]
    InvalidConfig(#[from] net::AddrParseError),

    #[error("{0}")]
    UnknownError(#[from] anyhow::Error),

    #[error("request send failed, {0}")]
    SendError(#[from] mpsc::SendError<Request>),

    #[error("response recv failed, {0}")]
    RecvError(#[from] mpsc::RecvError),

    #[error("jute error: {0}")]
    JuteError(#[from] jute::JuteError),

    #[error("io error, {0}")]
    IoError(#[from] io::Error),

    #[error("timeout, {0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),
}
