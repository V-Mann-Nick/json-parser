#[derive(Debug, PartialEq, PartialOrd)]
pub enum Number {
    Integer(i64),
    UnsingedInteger(u64),
    Float(f64),
}

impl Number {
    pub fn parse(sequence: &String) -> Option<Self> {
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
pub enum TokenType {
    BeginArray,
    EndArray,

    BeginObject,
    EndObject,

    NameSeparator,
    ValueSeparator,

    False,
    True,
    Null,

    Number(Number),
    // Pass by reference? -> Learn lifetime
    String(String),

    Invalid,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    pub token_type: TokenType,
    value: String,
    pub line: usize,
    start: usize,
    end: usize,
}

const EOF: char = '\u{0}';

const ESCAPED_CHARACTERS: [(char, char); 8] = [
    ('"', '"'),
    ('\\', '\\'),
    ('/', '/'),
    ('b', '\u{8}'),
    ('f', '\u{c}'),
    ('n', '\n'),
    ('r', '\r'),
    ('t', '\t'),
];

pub struct Tokenizer {
    input: String,
    character: char,
    characters: Vec<char>,
    position: usize,
    line: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        let characters = input.chars().collect::<Vec<char>>();
        Self {
            input,
            character: characters[0],
            characters,
            position: 0,
            line: 0,
        }
    }

    pub fn line(&self) -> usize {
        return self.line;
    }

    fn read_char(&mut self) {
        let next_position = self.position + 1;
        let next_character = self.characters.get(next_position);
        if let Some(next_character) = next_character {
            self.character = *next_character;
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

    // TODO: Handle unicode escape sequences
    fn read_string(&mut self) -> Token {
        let start = self.position;
        let mut sequence = String::new();
        loop {
            if self.character == '\\' {
                self.read_char();
                if let Some(escaped_character) = ESCAPED_CHARACTERS
                    .iter()
                    .find(|(escaped, _)| *escaped == self.character)
                {
                    sequence.push(escaped_character.1);
                    self.read_char();
                } else {
                    if self.character == 'u' {
                        eprintln!("Unicode escape sequences are not supported yet");
                    }
                    sequence.push(self.character);
                    self.read_char();
                    return Token {
                        token_type: TokenType::Invalid,
                        value: sequence,
                        line: self.line,
                        start,
                        end: self.position,
                    };
                }
            } else if sequence.len() >= 1 && self.character == '"' {
                sequence.push(self.character);
                self.read_char();
                break;
            } else if self.character == EOF {
                return Token {
                    token_type: TokenType::Invalid,
                    value: sequence,
                    line: self.line,
                    start,
                    end: self.position,
                };
            } else {
                sequence.push(self.character);
                self.read_char();
            }
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

    fn read_plain_token(&mut self, token_type: TokenType) -> Token {
        let token = Token {
            token_type,
            value: self.character.into(),
            line: self.line,
            start: self.position,
            end: self.position + 1,
        };
        self.read_char();
        token
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let result = match self.character {
            EOF => return None,
            '[' => self.read_plain_token(TokenType::BeginArray),
            ']' => self.read_plain_token(TokenType::EndArray),
            '{' => self.read_plain_token(TokenType::BeginObject),
            '}' => self.read_plain_token(TokenType::EndObject),
            ':' => self.read_plain_token(TokenType::NameSeparator),
            ',' => self.read_plain_token(TokenType::ValueSeparator),
            '"' => self.read_string(),
            _ => {
                if is_letter(self.character) {
                    self.read_literal()
                } else if is_number_character(self.character) {
                    self.read_number()
                } else {
                    self.read_plain_token(TokenType::Invalid)
                }
            }
        };

        // println!("{:?}", result);

        Some(result)
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        return self.next_token();
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
        let lexer = Tokenizer::new(String::from(json));
        let tokens: Vec<Token> = lexer.collect();
        tokens
    }

    #[test_case("2073", Number::UnsingedInteger(2073) ; "Plain integer")]
    #[test_case("2e2", Number::UnsingedInteger(200) ; "Integer with exponent notation")]
    #[test_case("2E2", Number::UnsingedInteger(200) ; "Integer with upper case exponent notation")]
    #[test_case("0.22e2", Number::UnsingedInteger(22) ; "Integer with weird exponent notation")]
    #[test_case("-2073", Number::Integer(-2073) ; "Plain negative integer")]
    #[test_case("-2e2", Number::Integer(-200) ; "Negative integer with exponent notation")]
    #[test_case("-2E2", Number::Integer(-200) ; "Negative integer with upper case exponent notation")]
    #[test_case("0.534", Number::Float(0.534) ; "Float: 0 < x < 1")]
    #[test_case("234.534", Number::Float(234.534) ; "Float: x > 1")]
    #[test_case("0.22e-2", Number::Float(0.0022) ; "Float with exponent notation")]
    fn number_tokens(json: &str, expected_number: Number) {
        let tokens = collect_tokens(json);
        assert_eq!(TokenType::Number(expected_number), tokens[0].token_type)
    }

    #[test_case(r#""Hello""#, "Hello" ; "Plain example")]
    #[test_case(r#""Hellö""#, "Hellö" ; "String with non-ASCII")]
    #[test_case(r#""\"""#, "\"" ; "String with escaped quote")]
    #[test_case(r#""\\""#, "\\" ; "String with escaped reverse solidus")]
    #[test_case(r#""\/""#, "/" ; "String with escaped solidus")]
    #[test_case(r#""\b""#, "\u{8}" ; "String with backspace")]
    #[test_case(r#""\f""#, "\u{C}" ; "String with form feed")]
    #[test_case(r#""\n""#, "\n" ; "String with line feed")]
    #[test_case(r#""\r""#, "\r" ; "String with carriage return")]
    #[test_case(r#""\t""#, "\t" ; "String with tab")]
    fn string_tokens(json: &str, expected_string: &str) {
        let tokens = collect_tokens(json);
        assert_eq!(
            tokens[0].token_type,
            TokenType::String(String::from(expected_string))
        );
    }

    #[test_case(r#""\u0022""# ; "String with unicode escape sequence")]
    #[test_case(r#""\uD834\uDD1E""# ; "String with unicode escape sequence and surrogate pair")]
    fn non_supported_string_tokens(json: &str) {
        let tokens = collect_tokens(json);
        assert_eq!(tokens[0].token_type, TokenType::Invalid);
    }

    #[test]
    fn array_tokens() {
        let tokens = collect_tokens("[5]");
        assert_eq!(
            Token {
                token_type: TokenType::BeginArray,
                value: String::from("["),
                line: 0,
                start: 0,
                end: 1
            },
            tokens[0]
        );
        assert_eq!(
            Token {
                token_type: TokenType::Number(Number::UnsingedInteger(5)),
                value: String::from("5"),
                line: 0,
                start: 1,
                end: 2
            },
            tokens[1]
        );
        assert_eq!(
            Token {
                token_type: TokenType::EndArray,
                value: String::from("]"),
                line: 0,
                start: 2,
                end: 3,
            },
            tokens[2]
        );
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
                token_type: TokenType::String(String::from("key")),
                value: String::from("\"key\""),
                line: 0,
                start: 2,
                end: 7,
            },
            tokens[1]
        );
        assert_eq!(
            Token {
                token_type: TokenType::NameSeparator,
                value: String::from(":"),
                line: 0,
                start: 7,
                end: 8,
            },
            tokens[2]
        );
        assert_eq!(
            Token {
                token_type: TokenType::String(String::from("value")),
                value: String::from("\"value\""),
                line: 0,
                start: 9,
                end: 16,
            },
            tokens[3]
        );
        assert_eq!(
            Token {
                token_type: TokenType::EndObject,
                value: String::from("}"),
                line: 0,
                start: 17,
                end: 18
            },
            tokens[4]
        )
    }
}
