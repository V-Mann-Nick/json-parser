#[derive(Debug, PartialEq, PartialOrd)]
enum Number {
    Integer(i64),
    UnsingedInteger(u64),
    Float(f64),
}

impl Number {
    fn parse(sequence: &String) -> Option<Self> {
        if let Ok(integer) = sequence.parse::<u64>() {
            Some(Number::UnsingedInteger(integer))
        } else if let Ok(integer) = sequence.parse::<i64>() {
            Some(Number::Integer(integer))
        } else if let Ok(float) = sequence.parse::<f64>() {
            if float.fract() == 0.0 && float >= u64::MIN as f64 && float <= u64::MAX as f64 {
                Some(Number::UnsingedInteger(float as u64))
            } else if float.fract() == 0.0 && float >= i64::MIN as f64 && float <= i64::MAX as f64 {
                Some(Number::Integer(float as i64))
            } else {
                Some(Number::Float(float))
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum TokenType {
    BeginArray,
    EndArray,

    BeginObject,
    EndObject,

    NameSeparator,
    ValueSeparator,

    False,
    True,
    Null,

    // Good type?
    Number(Number),
    // Pass by reference? -> Learn lifetime
    String(String),

    Invalid,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    token_type: TokenType,
    value: String,
    line: usize,
    start: usize,
    end: usize,
}

impl Token {
    fn from_lexer(token_type: TokenType, lexer: &Lexer) -> Self {
        Self {
            token_type,
            value: lexer.character.into(),
            line: lexer.line,
            start: lexer.position,
            end: lexer.position + 1,
        }
    }
}

pub struct Lexer {
    input: String,
    character: char,
    position: usize,
    line: usize,
}

const EOF: char = '\u{0}';

impl Lexer {
    pub fn new(input: String) -> Self {
        let position: usize = 0;
        let character = input.chars().nth(0).unwrap();
        Self {
            input,
            character,
            position,
            line: 0,
        }
    }

    fn read_char(&mut self) {
        let next_position = self.position + 1;
        let next_character = self.input.chars().nth(next_position);
        if let Some(next_character) = next_character {
            self.character = next_character;
        } else {
            self.character = EOF;
        };
        self.position = next_position;
    }

    fn advance_line(&mut self) {
        self.line = self.line + 1;
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.character {
                ' ' => {}
                '\n' => self.advance_line(),
                '\r' => self.advance_line(),
                '\t' => {}
                _ => break,
            };
            self.read_char();
        }
    }

    fn read_sequence(&mut self, is_valid: fn(character: char) -> bool) -> (usize, String) {
        let start = self.position;
        let mut sequence = String::new();
        loop {
            if !is_valid(self.character) {
                break;
            }
            sequence.push(self.character);
            self.read_char();
        }
        (start, sequence)
    }

    fn read_literal(&mut self) -> Token {
        let (start, literal) = self.read_sequence(is_letter);
        let token_type = match literal.as_str() {
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            _ => TokenType::Invalid,
        };
        Token {
            token_type,
            value: literal,
            line: self.line,
            start,
            end: self.position,
        }
    }

    fn read_number(&mut self) -> Token {
        let (start, sequence) = self.read_sequence(is_number_character);
        let parsed_value = Number::parse(&sequence);
        let token_type = if let Some(parsed_value) = parsed_value {
            TokenType::Number(parsed_value)
        } else {
            TokenType::Invalid
        };
        Token {
            token_type,
            value: sequence,
            line: self.line,
            start,
            end: self.position,
        }
    }

    // TODO: does not handle any unicode escape sequences or escaped characters
    // QUESTION: Are control characters already handled when reading files?
    fn read_string(&mut self) -> Token {
        let start = self.position;
        let mut sequence = String::new();
        loop {
            if sequence.len() >= 1
                && self.character == '"'
                && sequence.chars().nth(sequence.len() - 1).unwrap_or_default() != '\\'
            {
                sequence.push(self.character);
                self.read_char();
                break;
            }
            sequence.push(self.character);
            self.read_char();
        }
        let string_value = String::from(&sequence[1..(sequence.len() - 1)]);
        Token {
            token_type: TokenType::String(string_value),
            value: sequence,
            line: self.line,
            start,
            end: self.position,
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let result = match self.character {
            '[' => Token::from_lexer(TokenType::BeginArray, &self),
            ']' => Token::from_lexer(TokenType::EndArray, &self),
            '{' => Token::from_lexer(TokenType::BeginObject, &self),
            '}' => Token::from_lexer(TokenType::EndObject, &self),
            ':' => Token::from_lexer(TokenType::NameSeparator, &self),
            ',' => Token::from_lexer(TokenType::ValueSeparator, &self),
            EOF => return None,
            '"' => self.read_string(),
            _ => {
                if is_letter(self.character) {
                    self.read_literal()
                } else if is_number_character(self.character) {
                    self.read_number()
                } else {
                    Token::from_lexer(TokenType::Invalid, &self)
                }
            }
        };

        self.read_char();

        Some(result)
    }
}

fn is_letter(character: char) -> bool {
    character >= 'a' && character <= 'z' || character >= 'A' && character <= 'Z'
}

fn is_number_character(character: char) -> bool {
    character >= '0' && character <= '9'
        || character == '-'
        || character == '+'
        || character == '.'
        || character == 'e'
        || character == 'E'
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn collect_tokens(json: &str) -> Vec<Token> {
        let lexer = Lexer::new(String::from(json));
        let tokens: Vec<Token> = lexer.collect();
        tokens
    }

    #[test_case("-2073", -2073 ; "Simple example")]
    #[test_case("-2e2", -200 ; "Exponent example")]
    #[test_case("-2E2", -200 ; "Exponent upper case example")]
    fn integer_token(json: &str, expected_number: i64) {
        let tokens = collect_tokens(json);
        assert_eq!(
            TokenType::Number(Number::Integer(expected_number)),
            tokens[0].token_type
        )
    }

    #[test_case("2073", 2073 ; "Simple example")]
    #[test_case("2e2", 200 ; "Exponent example")]
    #[test_case("2E2", 200 ; "Exponent upper case example")]
    fn unsinged_integer_token(json: &str, expected_number: u64) {
        let tokens = collect_tokens(json);
        assert_eq!(
            TokenType::Number(Number::UnsingedInteger(expected_number)),
            tokens[0].token_type
        )
    }

    #[test]
    fn object_tokens() {
        let tokens = collect_tokens("{ \"key\": \"value\" }");
        assert_eq!(
            Token {
                token_type: TokenType::BeginObject,
                value: String::from("{"),
                line: 0,
                start: 0,
                end: 1
            },
            tokens[0]
        );
        assert_eq!(
            Token {
                token_type: TokenType::EndObject,
                value: String::from("}"),
                line: 0,
                start: 17,
                end: 18
            },
            tokens[tokens.len() - 1]
        )
    }
}
