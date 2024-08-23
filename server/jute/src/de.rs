use serde::de::Visitor;
use serde::forward_to_deserialize_any;
use crate::error::JuteError;

pub struct Deserializer {}

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = JuteError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
