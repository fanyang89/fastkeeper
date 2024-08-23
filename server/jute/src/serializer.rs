use bytes::{BufMut, BytesMut};
use serde::Serialize;
use std::any::type_name;

use crate::error::JuteError;

pub struct Serializer {
    buf: BytesMut,
}

pub fn to_bytes<T>(value: &T) -> Result<bytes::Bytes, JuteError>
where
    T: Serialize,
{
    let mut s = Serializer::new();
    value.serialize(&mut s)?;
    Ok(s.buf.freeze())
}

pub fn new() -> Serializer {
    Serializer::new()
}

pub fn with_capacity(capacity: usize) -> Serializer {
    Serializer::with_capacity(capacity)
}

impl Serializer {
    pub fn new() -> Serializer {
        Serializer {
            buf: BytesMut::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Serializer {
        Serializer {
            buf: BytesMut::with_capacity(capacity),
        }
    }

    pub fn to_bytes(self) -> Result<bytes::Bytes, JuteError> {
        Ok(self.buf.freeze())
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType("tuple struct"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType("tuple struct"))
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType("tuple"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType("tuple"))
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType("tuple variant"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType("tuple variant"))
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType("struct variant"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType("struct variant"))
    }
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = JuteError;
    type SerializeSeq = Self;
    type SerializeTuple = Self; // as seq
    type SerializeTupleStruct = Self; // not supported
    type SerializeTupleVariant = Self; // not supported
    type SerializeMap = Self;
    type SerializeStruct = Self; // as map
    type SerializeStructVariant = Self; // not supported

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let b = if v { 0x1 } else { 0x0 };
        self.serialize_u8(b)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(v as i32)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(v as i32)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i32(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.buf.put_i64(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u8(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as u32)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u32(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.buf.put_u64(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.buf.put_f32(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.buf.put_f64(v);
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType(type_name::<char>()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if v.is_empty() {
            return self.serialize_i32(-1);
        }
        self.serialize_i32(v.len() as i32)?;
        self.buf.put_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType("None"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType("Some"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType(type_name::<()>()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_u32(len.ok_or(JuteError::NoneLength)? as u32)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(JuteError::NotSupportedType(name))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_u32(len.ok_or(JuteError::NoneLength)? as u32)?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(JuteError::NotSupportedType(name))
    }
}
