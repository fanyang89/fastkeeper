use indexmap::indexmap;
use jute;
use jute::error::JuteError;
use maplit::hashmap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Serializer;

#[test]
fn serialize_primitives() -> Result<(), JuteError> {
    // bool
    {
        let b = false;
        let buf = jute::serializer::to_bytes(&b)?;
        assert_eq!(buf.len(), 1);
        assert_eq!(*buf.first().unwrap(), 0x0);

        let b = true;
        let buf = jute::serializer::to_bytes(&b)?;
        assert_eq!(buf.len(), 1);
        assert_eq!(*buf.first().unwrap(), 0x1);
    }

    // i32
    {
        let b = 1234;
        let buf = jute::serializer::to_bytes(&b)?;
        assert_eq!(buf.len(), 4);
        assert_eq!(buf, b"\0\0\x04\xd2"[..]);
    }

    // complex
    // copy from https://github.com/go-zookeeper/jute/blob/master/lib/go/jute/binary_encoder_test.go
    {
        let mut s = jute::serializer::new();
        s.serialize_bool(true)?;
        s.serialize_bool(false)?;
        s.serialize_u8(b'f')?;
        s.serialize_u8(b'b')?;
        s.serialize_i32(19406)?;
        s.serialize_i32(2147483647)?;
        s.serialize_i32(-420)?;
        s.serialize_i64(19406)?;
        s.serialize_i64(9223372036854775807)?;
        s.serialize_f32(3.14159265)?;
        s.serialize_f64(3.14159265)?;
        s.serialize_str("hello")?;
        let b: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        s.serialize_bytes(b)?;

        let expected: &[u8] = &[
            0x01, // boolean: true
            0x00, // boolean: false
            0x66, // byte: 'f'
            0x62, // byte: 'b'
            0x00, 0x00, 0x4b, 0xce, // int: 19406
            0x7f, 0xff, 0xff, 0xff, // int: 2147483647
            0xff, 0xff, 0xfe, 0x5c, // int: -420
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4b, 0xce, // long: 19406
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // long: 9,223,372,036,854,775,807
            0x40, 0x49, 0x0f, 0xdb, // float: 3.141592564
            0x40, 0x09, 0x21, 0xfb, 0x53, 0xc8, 0xd4, 0xf1, // double: 3.141592564
            // string: hello
            0x00, 0x00, 0x00, 0x05, // string len
            0x68, 0x65, 0x6c, 0x6c, 0x6f, // 'h', 'e', 'l', 'l', 'o'
            // buffer: 0x01, 0x02, 0x03, 0x04
            0x00, 0x00, 0x00, 0x04, // buffer len
            0x01, 0x02, 0x03, 0x04, // buffer contents
        ];
        let out = s.to_bytes()?;
        assert_eq!(out.len(), expected.len());
        assert_eq!(out.as_ref(), expected);
    }

    Ok(())
}

#[test]
fn serialize_seq() -> Result<(), JuteError> {
    let slice = [5, 4, 3, 2, 1];
    let mut s = jute::serializer::new();
    let mut seq = s.serialize_seq(Some(slice.len()))?;
    for x in slice {
        seq.serialize_element(&x)?;
    }
    SerializeSeq::end(seq)?;

    let expected: &[u8] = &[
        0x00, 0x00, 0x00, 0x05, // length of vector
        0x00, 0x00, 0x00, 0x05, // 5
        0x00, 0x00, 0x00, 0x04, // 4
        0x00, 0x00, 0x00, 0x03, // 3
        0x00, 0x00, 0x00, 0x02, // 2
        0x00, 0x00, 0x00, 0x01, // 1
    ];
    let out = s.to_bytes()?;
    assert_eq!(out.len(), expected.len());
    assert_eq!(out.as_ref(), expected);

    Ok(())
}

#[test]
fn serialize_map() -> Result<(), JuteError> {
    let m = indexmap! {
        "one" => 1,
        "two" => 2,
    };
    let mut s = jute::serializer::new();
    let mut map = s.serialize_map(Some(m.len()))?;
    for (k, v) in m.iter() {
        map.serialize_key(k)?;
        map.serialize_value(v)?;
    }
    SerializeMap::end(map)?;
    let out = s.to_bytes()?;
    let expected: &[u8] = &[
        0x00, 0x00, 0x00, 0x02, // length of map
        0x00, 0x00, 0x00, 0x03, // string length of "one"
        0x6f, 0x6e, 0x65, // 'o', 'n', 'e'
        0x00, 0x00, 0x00, 0x01, // int: 1
        0x00, 0x00, 0x00, 0x03, // length of "two"
        0x74, 0x77, 0x6f, // 't', 'w', 'o'
        0x00, 0x00, 0x00, 0x02, // int: 2
    ];
    assert_eq!(out.len(), expected.len());
    assert_eq!(out.as_ref(), expected);
    Ok(())
}
