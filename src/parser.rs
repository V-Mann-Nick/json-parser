use std::collections::HashMap;
use std::fmt::Display;

use super::tokenizer::{Number, Token, TokenType, Tokenizer};

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
}

fn parse_value(tokenizer: &mut Tokenizer, token: Option<Token>) -> Result<Value, ParseError> {
    let token = if let Some(token) = token {
        token
    } else {
        unwrap_token(tokenizer)?
    };
    let value = match token.token_type {
        TokenType::Null => Value::Null,
        TokenType::True => Value::Bool(true),
        TokenType::False => Value::Bool(false),
        TokenType::Number(number) => Value::Number(number),
        TokenType::String(string) => Value::String(string),
        TokenType::BeginObject => parse_object(tokenizer)?,
        TokenType::BeginArray => parse_array(tokenizer)?,
        TokenType::Invalid => return Err(ParseError::InvalidToken(token)),
        _ => return Err(ParseError::UnexpectedToken(token)),
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
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        had_comma = !had_comma;
    }
}

fn parse_property(tokenizer: &mut Tokenizer, token: Token) -> Result<(String, Value), ParseError> {
    let key = match token.token_type {
        TokenType::String(key) => key,
        _ => return Err(ParseError::UnexpectedToken(token)),
    };
    let token = unwrap_token(tokenizer)?;
    match token.token_type {
        TokenType::NameSeparator => {}
        _ => return Err(ParseError::UnexpectedToken(token)),
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
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        had_comma = !had_comma;
    }
}

fn unwrap_token(tokenizer: &mut Tokenizer) -> Result<Token, ParseError> {
    if let Some(token) = tokenizer.next_token() {
        return Ok(token);
    };
    return Err(ParseError::UnexpectedEndOfFile(tokenizer.line()));
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfFile(usize),
    UnexpectedToken(Token),
    InvalidToken(Token),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile(line) => {
                write!(f, "Parsing Error on line {}: Unexpected end of file", line)
            }
            Self::UnexpectedToken(token) => {
                write!(
                    f,
                    "Parsing Error on line {}: Unexpected token {:?}",
                    token.line, token
                )
            }
            Self::InvalidToken(token) => {
                write!(
                    f,
                    "Parsing Error on line {}: Invalid token {:?}",
                    token.line, token
                )
            }
        }
    }
}
