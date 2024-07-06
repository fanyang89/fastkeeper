use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("End of buffer")]
    Eof,
}

pub trait Deserialize {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

impl Deserialize for bool {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        Ok(buf.get_u8() == 1)
    }
}

impl Deserialize for i32 {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        Ok(buf.get_i32())
    }
}

impl Deserialize for i64 {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        Ok(buf.get_i64())
    }
}

impl Deserialize for f32 {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        Ok(buf.get_f32())
    }
}

impl Deserialize for f64 {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        Ok(buf.get_f64())
    }
}

pub type Buffer = Vec<u8>;

impl Deserialize for Buffer {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        let size: usize = i32::from_buffer(buf)? as usize;
        let b = buf.get(0..size).ok_or(Error::Eof)?;
        Ok(b.to_vec())
    }
}

impl Deserialize for String {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        let b = Vec::<u8>::from_buffer(buf)?;
        Ok(String::from_utf8(b)?)
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        let size: usize = i32::from_buffer(buf)? as usize;
        let mut b = Vec::new();
        for _ in 0..size {
            b.push(T::from_buffer(buf)?);
        }
        Ok(b)
    }
}

impl<K: Deserialize + Hash + Eq, V: Deserialize> Deserialize for HashMap<K, V> {
    fn from_buffer(buf: &mut Bytes) -> Result<Self, anyhow::Error> {
        let size: usize = i32::from_buffer(buf)? as usize;
        let mut m = HashMap::new();
        for _ in 0..size {
            let key = K::from_buffer(buf)?;
            let value = V::from_buffer(buf)?;
            m.insert(key, value);
        }
        Ok(m)
    }
}

pub trait Serialize {
    fn write_buffer(&self, buf: &mut BytesMut);
}

impl Serialize for bool {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_u8(if *self { 1 } else { 0 });
    }
}

impl Serialize for i32 {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i32(*self);
    }
}

impl Serialize for i64 {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i64(*self);
    }
}

impl Serialize for f32 {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_f32(*self);
    }
}

impl Serialize for f64 {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_f64(*self);
    }
}

impl Serialize for Buffer {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i32(self.len() as i32);
        buf.put_slice(self.as_slice());
    }
}

impl Serialize for String {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i32(self.len() as i32);
        buf.put_slice(self.as_bytes());
    }
}

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i32(self.len() as i32);
        for it in self.iter() {
            T::write_buffer(it, buf);
        }
    }
}

impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn write_buffer(&self, buf: &mut BytesMut) {
        buf.put_i32(self.len() as i32);
        for (k, v) in self.iter() {
            K::write_buffer(k, buf);
            V::write_buffer(v, buf);
        }
    }
}

#[macro_export]
macro_rules! jute_message {
    ( $name:ident { $( $field_name:ident : $field_type:ty $(,)?)* }) => {
        pub struct $name {
            $( pub $field_name : $field_type,)*
        }

        impl jute::Serialize for $name {
            fn write_buffer(&self, buf: &mut jute::BytesMut) {
                $(self.$field_name.write_buffer(buf); )*
            }
        }

        impl jute::Deserialize for $name {
            fn from_buffer(buf: &mut jute::Bytes) -> Result<Self, anyhow::Error> {
                Ok($name {
                    $($field_name : <$field_type>::from_buffer(buf)?, )*
                })
            }
        }
    }
}
