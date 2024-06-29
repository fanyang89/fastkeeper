use jute_rs::jute;
use serde_lite_derive::Serialize;

#[derive(Serialize)]
struct MyStruct {
    field1: u32,
    field2: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut se = jute::Serializer::new();
    se.serialize(MyStruct {
        field1: 10,
        field2: String::from("Hello, World!"),
    })
    .unwrap();
    let buf = se.get_buf();
    dbg!(&buf);
}
