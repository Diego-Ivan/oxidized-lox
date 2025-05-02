use crate::token::*;
use std::collections::HashMap;

static DECIMAL_SEPARATOR: u8 = b'.';

#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("Unterminated string literal")]
    UnterminatedStringLiteral,
    #[error("Unexpected character: {0}")]
    UnexpectedCharacter(u8),
    #[error("Failed to parse lexeme in line {0}, not an UTF-8 character")]
    NotUtf8(usize),
}

pub type ScannerResult<T> = Result<T, ScannerError>;

pub struct Scanner<'a> {
    source: &'a [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

    identifier_map: HashMap<String, TokenType>,
}

#[derive(Debug, PartialEq, Eq)]
enum NumberParseSection {
    Integer,
    Decimal,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut identifier_map = HashMap::new();
        macro_rules! insert_token {
            ($str: expr, $tkn: ident) => {
                identifier_map.insert(String::from($str), TokenType::$tkn);
            };
        }

        insert_token!("and", And);
        insert_token!("class", Class);
        insert_token!("else", Else);
        insert_token!("false", False);
        insert_token!("for", For);
        insert_token!("fun", Fun);
        insert_token!("if", If);
        insert_token!("nil", Nil);
        insert_token!("or", Or);
        insert_token!("print", Print);
        insert_token!("return", Return);
        insert_token!("break", Break);
        insert_token!("continue", Continue);
        insert_token!("super", Super);
        insert_token!("this", This);
        insert_token!("true", True);
        insert_token!("var", Var);
        insert_token!("while", While);

        Scanner {
            source: source.as_bytes(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            identifier_map,
        }
    }

    fn scan_token(&mut self) -> ScannerResult<()> {
        let current = self.advance();
        match current {
            b'(' => self.add_token(TokenType::LeftParen),
            b')' => self.add_token(TokenType::RightParen),
            b'{' => self.add_token(TokenType::LeftBrace),
            b'}' => self.add_token(TokenType::RightBrace),
            b',' => self.add_token(TokenType::Comma),
            b'.' => self.add_token(TokenType::Dot),
            b'-' => self.add_token(TokenType::Minus),
            b'+' => self.add_token(TokenType::Plus),
            b';' => self.add_token(TokenType::Semicolon),
            b'*' => self.add_token(TokenType::Star),
            b'!' => {
                if self.match_character(b'=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            b'=' => {
                if self.match_character(b'=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            b'<' => {
                if self.match_character(b'=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            b'>' => {
                if self.match_character(b'=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            b'/' => {
                if self.match_character(b'/') {
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.advance();
                    }
                    Ok(())
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            b' ' | b'\r' | b'\t' => Ok(()),
            b'\n' => {
                self.line += 1;
                Ok(())
            }
            b'"' => self.consume_string(),
            // An identifier can start with an alphabetic character or with an underscore.
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                self.consume_identifier();
                Ok(())
            }
            b'0'..=b'9' => {
                self.consume_number();
                Ok(())
            }
            any => Err(ScannerError::UnexpectedCharacter(any)),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.current).copied()
    }

    fn match_character(&mut self, character: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != character {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> u8 {
        match self.source.get(self.current) {
            Some(byte) => {
                self.current += 1;
                *byte
            }
            None => 0,
        }
    }

    fn add_token(&mut self, token_type: TokenType) -> ScannerResult<()> {
        let lexeme = Vec::from(&self.source[self.start..self.current]);
        let lexeme = match String::from_utf8(lexeme) {
            Ok(lexeme) => lexeme,
            Err(e) => return Err(ScannerError::NotUtf8(self.line)),
        };
        let token = Token::new(token_type, lexeme, self.line);
        self.tokens.push(token);
        Ok(())
    }

    pub fn scan_tokens(&mut self) -> ScannerResult<&[Token]> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));

        Ok(&self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn consume_string(&mut self) -> ScannerResult<()> {
        let mut completed = false;
        while let Some(c) = self.peek() {
            match c {
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'"' => {
                    completed = true;
                    break;
                }
                _ => _ = self.advance(),
            }
        }

        self.advance();

        if self.is_at_end() && !completed {
            println!("{:?}", self.peek());
            return Err(ScannerError::UnterminatedStringLiteral);
        }

        let string = &self.source[self.start + 1..self.current - 1];
        let string = crate::utf8::convert_byte_slice_into_utf8(string);

        self.add_token(TokenType::String(string));

        Ok(())
    }

    fn consume_number(&mut self) {
        // Parse the first digit.
        let mut decimal: f64 = (self.source[self.start] - 0x30) as f64;
        let mut decimal_power = 0;
        let mut current_part = NumberParseSection::Integer;

        while let Some(c) = self.peek() {
            if c == DECIMAL_SEPARATOR {
                if current_part == NumberParseSection::Decimal {
                    break;
                }
                current_part = NumberParseSection::Decimal;
                self.advance();
                continue;
            }

            if !c.is_ascii_digit() {
                break;
            }

            let current_value = (c - 0x30) as f64;

            match current_part {
                NumberParseSection::Integer => {
                    decimal *= 10f64;
                    decimal += current_value;
                }
                NumberParseSection::Decimal => {
                    decimal_power -= 1;
                    decimal += current_value * 10f64.powi(decimal_power);
                }
            }
            self.advance();
        }

        self.add_token(TokenType::Number(decimal));
    }

    fn consume_identifier(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphanumeric() && c != b'_' {
                break;
            }
            self.advance();
        }

        let identifier = &self.source[self.start..self.current];
        let identifier = crate::utf8::convert_byte_slice_into_utf8(identifier);

        let token = match self.identifier_map.get(&identifier) {
            Some(token) => token.clone(),
            None => TokenType::Identifier(identifier),
        };

        self.add_token(token);
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;
    use crate::Token;

    #[test]
    fn single_line_string_literal() {
        let source = "a = \"Hello World\"";
        let mut scanner = super::Scanner::new(source);
        let result = scanner.scan_tokens().unwrap();
        assert_eq!(
            result,
            [
                Token::new(
                    TokenType::Identifier(String::from("a")),
                    String::from("a"),
                    1
                ),
                Token::new(TokenType::Equal, String::from("="), 1),
                Token::new(
                    TokenType::String(String::from("Hello World"),),
                    String::from("\"Hello World\""),
                    1
                ),
                Token::new(TokenType::Eof, String::from(""), 1),
            ]
        )
    }

    #[test]
    fn multi_line_string_literal() {
        let source = "a = \"hello\ncrayon\nlets go\"";
        let mut scanner = super::Scanner::new(source);
        let result = scanner.scan_tokens().unwrap();
        assert_eq!(
            result,
            [
                Token::new(
                    TokenType::Identifier(String::from("a")),
                    String::from("a"),
                    1
                ),
                Token::new(TokenType::Equal, String::from("="), 1),
                Token::new(
                    TokenType::String(String::from("hello\ncrayon\nlets go"),),
                    String::from("\"hello\ncrayon\nlets go\""),
                    3
                ),
                Token::new(TokenType::Eof, String::from(""), 3),
            ]
        )
    }

    #[test]
    fn test_multibyte_tokens() {
        let source = "== >= <= !=";
        let mut scanner = super::Scanner::new(source);
        let result = scanner.scan_tokens().unwrap();
        assert_eq!(
            result,
            [
                Token::new(TokenType::EqualEqual, String::from("=="), 1),
                Token::new(TokenType::GreaterEqual, String::from(">="), 1),
                Token::new(TokenType::LessEqual, String::from("<="), 1),
                Token::new(TokenType::BangEqual, String::from("!="), 1),
                Token::new(TokenType::Eof, String::from(""), 1),
            ]
        );
    }
}
