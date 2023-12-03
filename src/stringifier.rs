use super::parser::Value;
use std::collections::HashMap;

pub struct JsonStringifier<'v> {
    indent: u8,
    value: &'v Value,
    result: String,
}

impl<'v> JsonStringifier<'v> {
    pub fn new(value: &'v Value) -> Self {
        Self {
            indent: 2,
            value,
            result: String::new(),
        }
    }

    pub fn indent(mut self, indent: u8) -> Self {
        self.indent = indent;
        self
    }

    fn string(&mut self, string: &String) {
        self.result.push('"');
        for character in string.chars() {
            if let Some(escaped_character) = get_escaped_character(&character) {
                self.result.push_str(escaped_character);
            } else {
                self.result.push(character);
            }
        }
        self.result.push('"');
    }

    fn indentation(&mut self, indent_level: u8) {
        for _ in 0..(indent_level * self.indent) {
            self.result.push(' ')
        }
    }

    fn object(&mut self, object: &HashMap<String, Value>, indent_level: u8) {
        self.result.push('{');
        if object.len() >= 1 {
            self.result.push('\n');
            for (idx, (key, value)) in object.iter().enumerate() {
                self.indentation(indent_level + 1);
                self.string(key);
                self.result.push_str(": ");
                self.value(value, indent_level + 1);
                if idx + 1 != object.len() {
                    self.result.push(',');
                }
                self.result.push('\n');
            }
            self.indentation(indent_level);
        }
        self.result.push('}');
    }

    fn array(&mut self, array: &Vec<Value>, indent_level: u8) {
        self.result.push('[');
        if array.len() >= 1 {
            self.result.push('\n');
            for (idx, value) in array.iter().enumerate() {
                self.indentation(indent_level + 1);
                self.value(value, indent_level + 1);
                if idx + 1 != array.len() {
                    self.result.push(',')
                }
                self.result.push('\n');
            }
            self.indentation(indent_level);
        }
        self.result.push(']');
    }

    fn value(&mut self, value: &Value, indent_level: u8) {
        match value {
            Value::Null => self.result.push_str("null"),
            Value::Bool(bool) => self.result.push_str(&bool.to_string()),
            Value::Number(number) => self.result.push_str(&number.to_string()),
            Value::String(string) => self.string(string),
            Value::Object(object) => {
                self.object(object, indent_level);
            }
            Value::Array(array) => {
                self.array(array, indent_level);
            }
        }
    }

    pub fn create(mut self) -> String {
        self.value(self.value, 0);
        self.result
    }
}

fn get_escaped_character(character: &char) -> Option<&str> {
    match character {
        '"' => Some(r#"\""#),
        '\\' => Some(r#"\\"#),
        '\u{8}' => Some(r#"\b"#),
        '\u{c}' => Some(r#"\f"#),
        '\n' => Some(r#"\n"#),
        '\r' => Some(r#"\r"#),
        '\t' => Some(r#"\t"#),
        _ => None,
    }
}
