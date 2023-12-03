use std::collections::HashMap;
use std::fmt::Display;

use crate::{
    stringifier::JsonStringifier,
    tokenizer::{Location, Number, Token, TokenType, Tokenizer},
};

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl Value {
    pub fn parse(tokenizer: &mut Tokenizer) -> Result<Self, ParseError> {
        parse_value(tokenizer, None)
    }

    pub fn stringified(&self) -> JsonStringifier {
        JsonStringifier::new(self)
    }
}

fn parse_value(tokenizer: &mut Tokenizer, token: Option<Token>) -> Result<Value, ParseError> {
    let token = if let Some(token) = token {
        token
    } else {
        unwrap_token(tokenizer)?
    };
    let expected_tokens = vec!["null", "true", "false", "number", "string", "{", "["];
    let value = match token.token_type {
        TokenType::Null => Value::Null,
        TokenType::True => Value::Bool(true),
        TokenType::False => Value::Bool(false),
        TokenType::Number(number) => Value::Number(number),
        TokenType::String(string) => Value::String(string),
        TokenType::BeginObject => parse_object(tokenizer)?,
        TokenType::BeginArray => parse_array(tokenizer)?,
        TokenType::Invalid => {
            return Err(ParseError::InvalidToken(ParseErrorArgs::new(
                token,
                expected_tokens,
            )))
        }
        _ => {
            return Err(ParseError::UnexpectedToken(ParseErrorArgs::new(
                token,
                expected_tokens,
            )))
        }
    };
    Ok(value)
}

fn parse_object(tokenizer: &mut Tokenizer) -> Result<Value, ParseError> {
    let mut properties: HashMap<String, Value> = HashMap::new();
    let mut had_comma = true;
    loop {
        let token = unwrap_token(tokenizer)?;
        if had_comma {
            if properties.len() == 0 && token.token_type == TokenType::EndObject {
                return Ok(Value::Object(properties));
            }
            let (key, value) = parse_property(tokenizer, token)?;
            properties.insert(key, value);
        } else {
            match token.token_type {
                TokenType::EndObject => return Ok(Value::Object(properties)),
                TokenType::ValueSeparator => {}
                _ => {
                    return Err(ParseError::UnexpectedToken(ParseErrorArgs::new(
                        token,
                        vec!["}", ","],
                    )))
                }
            }
        }
        had_comma = !had_comma;
    }
}

fn parse_property(tokenizer: &mut Tokenizer, token: Token) -> Result<(String, Value), ParseError> {
    let key = match token.token_type {
        TokenType::String(key) => key,
        _ => {
            return Err(ParseError::UnexpectedToken(ParseErrorArgs::new(
                token,
                vec!["string"],
            )))
        }
    };
    let token = unwrap_token(tokenizer)?;
    match token.token_type {
        TokenType::NameSeparator => {}
        _ => {
            return Err(ParseError::UnexpectedToken(ParseErrorArgs::new(
                token,
                vec![":"],
            )))
        }
    }
    let value = parse_value(tokenizer, None)?;
    Ok((key, value))
}

fn parse_array(tokenizer: &mut Tokenizer) -> Result<Value, ParseError> {
    let mut values: Vec<Value> = Vec::new();
    let mut had_comma = true;
    loop {
        let token = unwrap_token(tokenizer)?;
        if had_comma {
            if values.len() == 0 && token.token_type == TokenType::EndArray {
                return Ok(Value::Array(values));
            }
            values.push(parse_value(tokenizer, Some(token))?);
        } else {
            match token.token_type {
                TokenType::EndArray => return Ok(Value::Array(values)),
                TokenType::ValueSeparator => {}
                _ => {
                    return Err(ParseError::UnexpectedToken(ParseErrorArgs::new(
                        token,
                        vec!["]", ","],
                    )))
                }
            }
        }
        had_comma = !had_comma;
    }
}

fn unwrap_token(tokenizer: &mut Tokenizer) -> Result<Token, ParseError> {
    if let Some(token) = tokenizer.next_token() {
        return Ok(token);
    };
    return Err(ParseError::UnexpectedEndOfFile(Location {
        line: tokenizer.line,
        column: tokenizer.column,
        length: 1,
    }));
}

#[derive(Debug)]
pub struct ParseErrorArgs {
    token: Token,
    expected_tokens: Vec<String>,
}

impl ParseErrorArgs {
    fn new(token: Token, expected_tokens: Vec<&str>) -> Self {
        let expected_tokens = expected_tokens
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Self {
            token,
            expected_tokens,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfFile(Location),
    UnexpectedToken(ParseErrorArgs),
    InvalidToken(ParseErrorArgs),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile(location) => {
                write!(f, "{}: Unexpected end of file", format_location(&location))
            }
            Self::UnexpectedToken(args) => {
                write!(
                    f,
                    "{}: {}. Received token `{}`",
                    format_location(&args.token.location),
                    format_expected(&args.expected_tokens),
                    args.token.value
                )
            }
            Self::InvalidToken(args) => {
                write!(
                    f,
                    "{}: {}. Received invalid token `{}`",
                    format_location(&args.token.location),
                    format_expected(&args.expected_tokens),
                    args.token.value
                )
            }
        }
    }
}

fn format_expected(expected_tokens: &Vec<String>) -> String {
    let expected_string =
        expected_tokens
            .iter()
            .enumerate()
            .fold(String::new(), |mut s, (idx, token)| {
                if idx + 1 == expected_tokens.len() {
                    s.push_str(" or ")
                } else if idx > 0 {
                    s.push_str(", ")
                }
                s.push_str(format!("`{}`", token).as_str());
                s
            });
    format!("Expected {}", expected_string)
}

fn format_location(location: &Location) -> String {
    format!(
        ">> Parsing Error on line {} column {}",
        location.line, location.column
    )
}

#[cfg(test)]
mod tests {
    use super::*;
}
