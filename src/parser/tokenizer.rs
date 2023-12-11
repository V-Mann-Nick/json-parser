use super::number::Number;

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
    String(String),

    Invalid,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Location {
    fn from_tokenizer(tokenizer: &Tokenizer, column_start: &usize) -> Self {
        Self {
            line: tokenizer.line,
            column: *column_start,
            length: tokenizer.column - *column_start,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub location: Location,
}

const EOF: char = '\u{0}';

pub struct Tokenizer {
    character: char,
    characters: Vec<char>,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        let characters = input.chars().collect::<Vec<char>>();
        Self {
            character: characters[0],
            characters,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn read_char(&mut self) {
        self.position = self.position + 1;
        self.column = self.column + 1;
        let next_character = self.characters.get(self.position);
        if let Some(next_character) = next_character {
            self.character = *next_character;
        } else {
            self.character = EOF;
        };
    }

    fn read_chars(&mut self, number: usize) {
        for _ in 0..number {
            self.read_char();
        }
    }

    fn peak_char(&self, offset: usize) -> char {
        if let Some(next_character) = self.characters.get(self.position + offset) {
            *next_character
        } else {
            EOF
        }
    }

    fn advance_line(&mut self) {
        self.line = self.line + 1;
        self.column = 0;
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
        let column_start = self.column;
        let mut sequence = String::new();
        loop {
            if !is_valid(self.character) {
                break;
            }
            sequence.push(self.character);
            self.read_char();
        }
        (column_start, sequence)
    }

    fn read_literal(&mut self) -> Token {
        let (column_start, literal) = self.read_sequence(is_letter);
        let token_type = match literal.as_str() {
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            _ => TokenType::Invalid,
        };
        Token {
            token_type,
            value: literal,
            location: Location::from_tokenizer(&self, &column_start),
        }
    }

    fn read_number(&mut self) -> Token {
        let (column_start, sequence) = self.read_sequence(is_number_character);
        let parsed_value = Number::parse(&sequence);
        let token_type = if let Some(parsed_value) = parsed_value {
            TokenType::Number(parsed_value)
        } else {
            TokenType::Invalid
        };
        Token {
            token_type,
            value: sequence,
            location: Location::from_tokenizer(&self, &column_start),
        }
    }

    fn read_hex_sequence(&mut self) -> Option<String> {
        let mut sequence = String::with_capacity(4);
        for _ in 0..4 {
            self.read_char();
            if !is_hex_character(self.character) {
                return None;
            }
            sequence.push(self.character);
        }
        Some(sequence)
    }

    fn read_unicode_escape_sequence(&mut self) -> Option<char> {
        let unicode_sequence = self.read_hex_sequence()?;
        let mut code_point = u32::from_str_radix(&unicode_sequence, 16).ok()?;
        let is_expecting_surrogate_pair = code_point >= 0xD800
            && code_point <= 0xDBFF
            && self.peak_char(1) == '\\'
            && self.peak_char(2) == 'u';
        if is_expecting_surrogate_pair {
            self.read_chars(2);
            let surrogate_sequence = self.read_hex_sequence()?;
            let high_surrogate = code_point;
            let low_surrogate = u32::from_str_radix(&surrogate_sequence, 16).ok()?;
            code_point = 0x10000 + ((high_surrogate - 0xD800) << 10) + (low_surrogate - 0xDC00);
        }
        char::from_u32(code_point)
    }

    fn read_escape_sequence(&mut self) -> Option<char> {
        self.read_char();
        match self.character {
            '"' => Some('"'),
            '\\' => Some('\\'),
            '/' => Some('/'),
            'b' => Some('\u{8}'),
            'f' => Some('\u{c}'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            'u' => self.read_unicode_escape_sequence(),
            _ => None,
        }
    }

    fn read_string(&mut self) -> Token {
        let position_start = self.position;
        let column_start = self.column;
        let mut sequence = Vec::<char>::new();
        loop {
            let should_break = sequence.len() >= 1 && self.character == '"';
            let maybe_character = match self.character {
                '\\' => self.read_escape_sequence(),
                EOF => None,
                _ => Some(self.character),
            };
            let Some(character) = maybe_character else {
                return Token {
                    token_type: TokenType::Invalid,
                    value: sequence.iter().collect(),
                    location: Location::from_tokenizer(&self, &column_start),
                };
            };
            sequence.push(character);
            self.read_char();
            if should_break {
                break;
            }
        }
        let string_value = sequence[1..(sequence.len() - 1)].iter().collect();
        let original_sequence = self.characters[position_start..self.position]
            .iter()
            .collect();
        Token {
            token_type: TokenType::String(string_value),
            value: original_sequence,
            location: Location::from_tokenizer(&self, &column_start),
        }
    }

    fn read_plain_token(&mut self, token_type: TokenType) -> Token {
        let token = Token {
            token_type,
            value: self.character.into(),
            location: Location {
                line: self.line,
                column: self.column,
                length: 1,
            },
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

fn is_hex_character(character: char) -> bool {
    character >= '0' && character <= '9'
        || character >= 'a' && character <= 'f'
        || character >= 'A' && character <= 'F'
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
    #[test_case(r#""\u0022""#, "\"" ; "String with unicode escape sequence")]
    #[test_case(r#""\u00FC\u00F6""#, "üö" ; "String with two unicode escape sequences")]
    #[test_case(r#""\uD834\uDD1E""#, "𝄞" ; "String with unicode escape sequence and surrogate pair")]
    fn string_tokens(json: &str, expected_string: &str) {
        let tokens = collect_tokens(json);
        assert_eq!(
            tokens[0].token_type,
            TokenType::String(String::from(expected_string))
        );
    }

    #[test]
    fn array_tokens() {
        let tokens = collect_tokens("[5]");
        assert_eq!(
            Token {
                token_type: TokenType::BeginArray,
                value: String::from("["),
                location: Location {
                    line: 1,
                    column: 1,
                    length: 1,
                }
            },
            tokens[0]
        );
        assert_eq!(
            Token {
                token_type: TokenType::Number(Number::UnsingedInteger(5)),
                value: String::from("5"),
                location: Location {
                    line: 1,
                    column: 2,
                    length: 1,
                }
            },
            tokens[1]
        );
        assert_eq!(
            Token {
                token_type: TokenType::EndArray,
                value: String::from("]"),
                location: Location {
                    line: 1,
                    column: 3,
                    length: 1,
                }
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
                location: Location {
                    line: 1,
                    column: 1,
                    length: 1,
                }
            },
            tokens[0]
        );
        assert_eq!(
            Token {
                token_type: TokenType::String(String::from("key")),
                value: String::from("\"key\""),
                location: Location {
                    line: 1,
                    column: 3,
                    length: 5,
                }
            },
            tokens[1]
        );
        assert_eq!(
            Token {
                token_type: TokenType::NameSeparator,
                value: String::from(":"),
                location: Location {
                    line: 1,
                    column: 8,
                    length: 1,
                }
            },
            tokens[2]
        );
        assert_eq!(
            Token {
                token_type: TokenType::String(String::from("value")),
                value: String::from("\"value\""),
                location: Location {
                    line: 1,
                    column: 10,
                    length: 7,
                }
            },
            tokens[3]
        );
        assert_eq!(
            Token {
                token_type: TokenType::EndObject,
                value: String::from("}"),
                location: Location {
                    line: 1,
                    column: 18,
                    length: 1,
                }
            },
            tokens[4]
        )
    }
}
