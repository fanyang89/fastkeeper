use bytes::BufMut;
use serde_lite::Intermediate;

pub struct Serializer {
    buf: bytes::BytesMut,
}

trait MaySwapBytes {
    fn may_swap_bytes(&self) -> Self;
}

macro_rules! impl_may_swap_bytes {
    ($x:ident) => {
        impl MaySwapBytes for $x {
            #[cfg(target_endian = "little")]
            fn may_swap_bytes(&self) -> Self {
                $x::swap_bytes(*self)
            }

            #[cfg(target_endian = "big")]
            fn may_swap_bytes(self) -> Self {
                self
            }
        }
    };
}

impl_may_swap_bytes!(u16);
impl_may_swap_bytes!(u32);
impl_may_swap_bytes!(u64);
impl_may_swap_bytes!(u128);
impl_may_swap_bytes!(i16);
impl_may_swap_bytes!(i32);
impl_may_swap_bytes!(i64);
impl_may_swap_bytes!(i128);

impl Serializer {
    pub fn new() -> Self {
        Serializer {
            buf: bytes::BytesMut::new(),
        }
    }

    pub fn get_buf(self) -> bytes::Bytes {
        bytes::Bytes::from(self.buf)
    }

    pub fn serialize<T>(&mut self, s: T) -> Result<(), anyhow::Error>
    where
        T: serde_lite::Serialize,
    {
        let inter = s.serialize()?;
        dbg!(&inter);
        self.visit(&inter)
    }

    fn visit(&mut self, s: &Intermediate) -> Result<(), anyhow::Error> {
        match s {
            Intermediate::None => {
                todo!()
            }

            Intermediate::Bool(b) => {
                self.buf.put_u8(if *b { 0x1 } else { 0x0 });
            }

            Intermediate::Number(n) => match n {
                serde_lite::Number::I8(v) => self.buf.put_i8(*v),
                serde_lite::Number::I16(v) => self.buf.put_i16(*v),
                serde_lite::Number::I32(v) => self.buf.put_i32(*v),
                serde_lite::Number::I64(v) => self.buf.put_i64(*v),
                serde_lite::Number::I128(v) => self.buf.put_i128(*v),

                serde_lite::Number::U8(v) => self.buf.put_u8(*v),
                serde_lite::Number::U16(v) => self.buf.put_u16(v.may_swap_bytes()),
                serde_lite::Number::U32(v) => self.buf.put_u32(v.may_swap_bytes()),
                serde_lite::Number::U64(v) => self.buf.put_u64(v.may_swap_bytes()),
                serde_lite::Number::U128(v) => self.buf.put_u128(v.may_swap_bytes()),

                serde_lite::Number::F32(v) => self.buf.put_u32(v.to_bits().may_swap_bytes()),
                serde_lite::Number::F64(v) => self.buf.put_u64(v.to_bits().may_swap_bytes()),
            },

            Intermediate::String(s) => {
                self.buf.put_u32(s.len().try_into()?);
                for x in s.as_bytes().iter() {
                    self.buf.put_u8(*x);
                }
            }

            Intermediate::Array(v) => {
                self.buf.put_u32(v.len().try_into()?);
                for x in v.iter() {
                    self.visit(x)?;
                }
            }

            Intermediate::Map(m) => {
                self.buf.put_u32(m.len().try_into()?);
                for (k, v) in m.iter() {
                    self.visit(&Intermediate::String(k.clone()))?;
                    self.visit(v)?;
                }
            }

            Intermediate::Buffer(b) => {
                self.buf.put_u32(b.len().try_into()?);
                self.buf.put_slice(b.as_slice());
            }
        }

        Ok(())
    }
}
