use serde_lite::Intermediate;
use std::marker::PhantomData;

pub struct Deserializer<'a, T>
where
    T: serde_lite::Deserialize + struct_iterable::Iterable,
{
    buf: &'a bytes::Bytes,
    phantom_t: PhantomData<&'a T>,
}

impl<'a, T> Deserializer<'a, T>
where
    T: serde_lite::Deserialize + struct_iterable::Iterable,
{
    pub fn from_buf(buf: &'a bytes::Bytes) -> Self {
        Deserializer {
            buf,
            phantom_t: PhantomData,
        }
    }

    pub fn deserialize(self) -> Result<Intermediate, anyhow::Error> {
        todo!()
    }
}
