use bytes::BufMut;
use bytes::BytesMut;
use jute::SerializeToBuffer;
use tokio::sync::oneshot;
use tracing::{info, trace};

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
        trace!("buf len: {}", buf.len());
        if let Some(body) = &self.body {
            let body_buf = body.to_buffer();
            trace!("body len: {}", body_buf.len());
            buf.put(body_buf);
        }
        buf.clone()
    }
}
