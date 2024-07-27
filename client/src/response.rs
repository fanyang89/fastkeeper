use crate::{error::ClientError, messages::proto::ReplyHeader};
use bytes::Bytes;
use jute::Deserialize;

#[derive(Debug)]
pub struct Response {
    pub header: ReplyHeader,
    pub body: Option<Bytes>,
}

impl Deserialize for Response {
    fn from_buffer(mut buf: &mut Bytes) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let header = ReplyHeader::from_buffer(&mut buf)?;
        let body = if let Some(size_buf) = buf.get(0..4) {
            let size = u32::from_be_bytes(size_buf.try_into()?) as usize;
            let data: Vec<u8> = buf
                .get(4..size)
                .ok_or(ClientError::CorruptedResponse)?
                .to_vec();
            Some(Bytes::from(data))
        } else {
            None
        };
        Ok(Self { header, body })
    }
}

impl Response {
    pub fn get_message<T: Deserialize>(&mut self) -> Result<T, anyhow::Error> {
        let Response { body, .. } = self;
        if let Some(buf) = body {
            T::from_buffer(buf)
        } else {
            Err(ClientError::EmptyResponseBody.into())
        }
    }
}
