mod number;
mod parse_error;
mod stringifier;
mod tokenizer;
mod value;

use indexmap::map::IndexMap;
pub use parse_error::ParseError;
use parse_error::ParseErrorArgs;
use tokenizer::{Location, Token, TokenType, Tokenizer};
pub use value::Value;

pub fn parse(json: &str) -> Result<Value, ParseError> {
    let mut tokenizer = Tokenizer::new(json.to_string());
    parse_value(&mut tokenizer, None)
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
    let mut properties: IndexMap<String, Value> = IndexMap::new();
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

#[cfg(test)]
mod tests {
    use super::*;
}
