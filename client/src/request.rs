use bytes::BufMut;
use bytes::BytesMut;
use tokio::sync::oneshot;

use jute::SerializeToBuffer;

use crate::{
    messages::{proto::RequestHeader, RequestBody},
    response::Response,
};

pub struct Request {
    pub header: RequestHeader,
    pub body: Option<RequestBody>,
    pub wake: Option<oneshot::Sender<Response>>, // None for send and leave
}

impl SerializeToBuffer for Request {
    fn to_buffer(&self) -> BytesMut {
        let mut buf = self.header.to_buffer();
        if let Some(body) = &self.body {
            let body_buf = body.to_buffer();
            buf.put_u32(body_buf.len() as u32);
            buf.put(body_buf);
        }
        buf.clone()
    }
}
