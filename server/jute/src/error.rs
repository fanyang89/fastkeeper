use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum JuteError {
    #[error("{0}")]
    CastFailed(#[from] std::num::TryFromIntError),

    #[error("not supported type: {0}")]
    NotSupportedType(&'static str),

    #[error("length is none")]
    NoneLength,

    #[error("serde serialize error: {0}")]
    SerializeFailed(String),

    #[error("serde deserialize error: {0}")]
    DeserializeFailed(String),
}

impl serde::ser::Error for JuteError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        JuteError::SerializeFailed(msg.to_string())
    }
}

impl serde::de::Error for JuteError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        JuteError::SerializeFailed(msg.to_string())
    }
}
