use std::io;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Bytes, BytesMut};
use tokio::net::TcpStream;

pub(crate) struct Frame {
    buf: BytesMut,
}

impl From<BytesMut> for Frame {
    fn from(buf: BytesMut) -> Self {
        Self { buf }
    }
}

impl Into<BytesMut> for Frame {
    fn into(self) -> BytesMut {
        self.buf
    }
}

impl Frame {
    pub fn freeze(self) -> Bytes {
        self.buf.freeze()
    }

    pub async fn send(&self, conn: &TcpStream) -> Result<(), anyhow::Error> {
        let size = self.buf.len() as u32;
        let size_buf = [0u8; 4];
        let mut cursor = io::Cursor::new(size_buf);
        cursor.write_u32::<BigEndian>(size)?;
        conn.try_write(&size_buf)?;
        conn.try_write(self.buf.as_ref())?;
        Ok(())
    }

    pub async fn recv(conn: &TcpStream) -> Result<Frame, anyhow::Error> {
        let mut size_buf = [0u8; 4];
        conn.try_read(&mut size_buf)?;
        let mut cursor = io::Cursor::new(size_buf);
        let size = cursor.read_u32::<BigEndian>()?;
        let mut buf = BytesMut::with_capacity(size as usize);
        conn.try_read(&mut buf)?;
        Ok(Frame { buf })
    }
}
