use crate::token::*;
use std::collections::HashMap;

static DECIMAL_SEPARATOR: u8 = b'.';

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

    fn scan_token(&mut self) {
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
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'"' => self.consume_string(),
            // An identifier can start with an alphabetic character or with an underscore.
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => self.consume_identifier(),
            b'0'..=b'9' => self.consume_number(),
            any => crate::error(self.line, &format!("Unexpected character {any}")),
        };
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

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = Vec::from(&self.source[self.start..self.current]);
        let lexeme = match String::from_utf8(lexeme) {
            Ok(lexeme) => lexeme,
            Err(e) => panic!("Could not parse lexeme into UTF-8: {e}"),
        };
        let token = Token::new(token_type, lexeme, self.line);
        self.tokens.push(token);
    }

    pub fn scan_tokens(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));

        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn consume_string(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                b'\n' => self.line += 1,
                b'"' => break,
                _ => _ = self.advance(),
            }
        }

        self.advance();

        if self.is_at_end() {
            println!("{:?}", self.peek());
            crate::error(self.line, "Unterminated string literal");
            return;
        }

        let string = &self.source[self.start + 1..self.current - 1];
        let string = crate::utf8::convert_byte_slice_into_utf8(string);

        self.add_token(TokenType::String(string));
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
