use jute_rs::{de, ser};
use serde_lite::Deserialize;
use serde_lite_derive::{Serialize, Deserialize};
use struct_iterable::Iterable;

#[derive(Serialize, Deserialize, Debug, Iterable)]
struct MyStruct {
    field1: u32,
    field2: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let mut se = ser::Serializer::new();
    se.serialize(MyStruct {
        field1: 10,
        field2: String::from("Hello, World!"),
    })?;
    let buf = se.get_buf();
    dbg!(&buf);

    let de = de::Deserializer::<MyStruct>::from_buf(&buf);
    let inter = de.deserialize()?;
    let instance = MyStruct::deserialize(&inter).unwrap();
    dbg!(&instance);

    Ok(())
}
