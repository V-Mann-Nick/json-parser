use super::tokenizer::{Location, Token};
use std::fmt::Display;

#[derive(Debug)]
pub struct ParseErrorArgs {
    token: Token,
    expected_tokens: Vec<String>,
}

impl ParseErrorArgs {
    pub fn new(token: Token, expected_tokens: Vec<&str>) -> Self {
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
