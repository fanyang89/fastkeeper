use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("corrupted response")]
    CorruptedResponse,

    #[error("empty response body")]
    EmptyResponseBody
}
