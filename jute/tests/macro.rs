use jute_rs::Jute;
use std::{collections::HashMap, string};

#[warn(dead_code)]
#[derive(jute_derive::Jute)]
struct Foo {
    field01: bool,
    field02: i32,
    field03: i64,
    field04: f32,
    field05: f64,
    field06: String,
    field07: string::String,
    field08: std::string::String,

    field09: Vec<u8>,
    field10: Vec<bool>,
    field11: Vec<i32>,
    field12: Vec<i64>,
    field13: Vec<f32>,
    field14: Vec<f64>,
    field15: Vec<String>,
    field16: Vec<Vec<u8>>,

    field17: HashMap<String, bool>,
    field18: HashMap<String, i32>,
    field19: HashMap<String, i64>,
    field20: HashMap<String, f32>,
    field21: HashMap<String, f64>,
    field22: HashMap<String, String>,
    field23: HashMap<String, Vec<u8>>,

    field24: HashMap<String, Vec<u8>>,
    field25: HashMap<String, Vec<i32>>,
    field26: HashMap<String, Vec<i64>>,
    field27: HashMap<String, Vec<f32>>,
    field28: HashMap<String, Vec<f64>>,
    field29: HashMap<String, Vec<String>>,
    field30: HashMap<String, Vec<Vec<u8>>>,
}

#[test]
fn reflect_basic() {
    println!("Type name: {}", Foo::type_name());
    println!("Field type: {:?}", Foo::field_types());
}
