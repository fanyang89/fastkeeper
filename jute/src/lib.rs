pub mod de;
pub mod ser;

#[derive(Debug)]
pub enum FieldType {
    Bool,
    Integer,
    Long,
    Float,
    Double,
    String,
    Buffer,
    Vector(Box<FieldType>),
    Map(Box<FieldType>), // map, key is string
}

pub trait Reflect {
    fn type_name() -> &'static str;
    fn field_types() -> Vec<FieldType>;
}
