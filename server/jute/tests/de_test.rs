use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SimpleStruct {
    a: i32,
}

#[test]
fn test_deserializer() {
    let x = SimpleStruct { a: 1 };
    let b = jute::serializer::to_bytes(&x);
    dbg!(&b);
}
