use crate::{error::ClientError, messages::proto::ReplyHeader};
use bytes::{Buf, Bytes};
use jute::{Deserialize, JuteError};

#[derive(Debug)]
pub struct Response {
    pub header: ReplyHeader,
    pub body: Option<Bytes>,
}

impl Deserialize for Response {
    fn from_buffer(mut buf: &mut Bytes) -> Result<Self, JuteError>
    where
        Self: Sized,
    {
        let header = ReplyHeader::from_buffer(&mut buf)?;
        let size = buf.get_u32() as usize;
        let body = if let Some(slice) = buf.get(0..size) {
            let mut x = vec![0u8; slice.len()];
            x[0..].clone_from_slice(slice);
            Some(Bytes::from(x))
        } else {
            None
        };
        Ok(Response { header, body })
    }
}

impl Response {
    pub fn get_message<T: Deserialize>(&mut self) -> Result<T, ClientError> {
        match &mut self.body {
            None => Err(ClientError::EmptyResponseBody),
            Some(ref mut buf) => T::from_buffer(buf).map_err(ClientError::JuteError),
        }
    }
}
