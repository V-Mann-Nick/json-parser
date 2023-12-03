use super::parser::Value;
use std::collections::HashMap;

pub struct JsonStringifier<'v> {
    indent: u8,
    value: &'v Value,
}

impl<'v> JsonStringifier<'v> {
    pub fn new(value: &'v Value) -> Self {
        Self { indent: 2, value }
    }

    pub fn indent(mut self, indent: u8) -> Self {
        self.indent = indent;
        self
    }

    fn indentation(&self, indent_level: u8) -> String {
        " ".repeat((indent_level * self.indent) as usize)
    }

    fn object(&self, object: &HashMap<String, Value>, indent_level: u8) -> String {
        let properties = object
            .iter()
            .enumerate()
            .fold(String::new(), |s, (idx, (key, value))| {
                format!(
                    "{}{}{}: {}{}\n",
                    s,
                    self.indentation(indent_level + 1),
                    key,
                    self.value(value, indent_level + 1),
                    if idx + 1 == object.len() { "" } else { "," }
                )
            });
        if object.len() >= 1 {
            format!("{{\n{}{}}}", properties, self.indentation(indent_level))
        } else {
            "{}".to_string()
        }
    }

    fn array(&self, array: &Vec<Value>, indent_level: u8) -> String {
        let values = array
            .iter()
            .enumerate()
            .fold(String::new(), |s, (idx, value)| {
                format!(
                    "{}{}{}{}\n",
                    s,
                    self.indentation(indent_level + 1),
                    self.value(value, indent_level + 1),
                    if idx + 1 == array.len() { "" } else { "," }
                )
            });
        if array.len() >= 1 {
            format!("[\n{}{}]", values, self.indentation(indent_level))
        } else {
            return "[]".to_string();
        }
    }

    fn value(&self, value: &Value, indent_level: u8) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(bool) => bool.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => format!("\"{}\"", string),
            Value::Object(object) => self.object(object, indent_level),
            Value::Array(array) => self.array(array, indent_level),
        }
    }

    pub fn create(&self) -> String {
        self.value(self.value, 0)
    }
}
