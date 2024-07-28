use std::io;

use crate::error::ClientError;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::{info, trace};

pub(crate) trait FrameReadWriter {
    async fn read_frame(&mut self) -> Result<Bytes, ClientError>;
    async fn write_frame(&self, buf: &Bytes) -> Result<(), ClientError>;
}

impl FrameReadWriter for TcpStream {
    async fn read_frame(&mut self) -> Result<Bytes, ClientError> {
        // read buf size
        let mut size_buf = [0u8; 4];
        self.read_exact(&mut size_buf).await?;
        let mut cursor = io::Cursor::new(size_buf);
        let size = ReadBytesExt::read_u32::<BigEndian>(&mut cursor).unwrap();

        // read buf
        let mut buf: Vec<u8> = vec![0u8; size as usize];
        self.read_exact(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn write_frame(&self, buf: &Bytes) -> Result<(), ClientError> {
        let mut size_buf = Vec::with_capacity(4);
        size_buf.write_u32::<BigEndian>(buf.len() as u32)?;
        self.try_write(&size_buf)?;
        self.try_write(&buf)?;
        Ok(())
    }
}
