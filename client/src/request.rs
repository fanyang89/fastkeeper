use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BytesMut;
use bytes::{Buf, BufMut, Bytes};
use jute::SerializeToBuffer;
use tokio::sync::oneshot;

use crate::{
    messages::{proto::RequestHeader, RequestBody},
    response::Response,
};

pub struct Request {
    pub header: RequestHeader,
    pub payload: Option<RequestBody>,
    pub wake: Option<oneshot::Sender<Response>>,
}

impl SerializeToBuffer for Request {
    fn to_buffer(&self) -> BytesMut {
        let mut buf = self.header.to_buffer();
        if let Some(payload) = &self.payload {
            let payload_buf = payload.to_buffer();
            buf.put_u32(buf.len() as u32);
            buf.put(payload_buf);
        }
        buf.clone()
    }
}
