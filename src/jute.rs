use bytes::BufMut;

pub struct Serializer {
    buf: bytes::BytesMut,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer {
            buf: bytes::BytesMut::new(),
        }
    }

    pub fn get_buf(self) -> bytes::Bytes {
        bytes::Bytes::from(self.buf)
    }

    pub fn serialize<T>(&mut self, s: T) -> Result<(), serde_lite::Error>
    where
        T: serde_lite::Serialize,
    {
        let inter = s.serialize()?;
        dbg!(&inter);
        self.visit(&inter);
        Ok(())
    }

    fn visit(&mut self, s: &serde_lite::Intermediate) {
        match s {
            serde_lite::Intermediate::None => {}
            serde_lite::Intermediate::Bool(b) => {
                self.buf.put_u8(if *b { 0x1 } else { 0x0 });
            }
            serde_lite::Intermediate::Number(n) => {
                dbg!(&n);
                todo!();
            }
            serde_lite::Intermediate::String(s) => {
                todo!();
            }
            serde_lite::Intermediate::Array(a) => {
                todo!();
            }
            serde_lite::Intermediate::Map(m) => {}
        }
    }
}
