use super::{number::Number, stringifier::JsonStringifier};
use indexmap::map::IndexMap;

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Object(IndexMap<String, Value>),
    Array(Vec<Value>),
}

impl Value {
    pub fn stringified(&self) -> JsonStringifier {
        JsonStringifier::new(self)
    }
}
