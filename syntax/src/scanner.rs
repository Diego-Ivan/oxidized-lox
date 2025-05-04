use crate::token::*;
use std::collections::HashMap;
use std::io::BufRead;

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

pub struct Scanner<R: BufRead> {
    reader: R,
    line: usize,
    current_byte: Option<u8>,
    identifier_map: HashMap<String, TokenType>,

    started: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum NumberParseSection {
    Integer,
    Decimal,
}

impl<R: BufRead> Scanner<R> {
    pub fn new(reader: R) -> Self {
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
            reader,
            line: 1,
            current_byte: None,
            identifier_map,
            started: false,
        }
    }

    fn scan_token(&mut self) -> Option<ScannerResult<Token>> {
        use TokenType::*;

        let mut lexeme = Vec::new();

        macro_rules! add_single_byte {
            ($c: ident, $token_type: ident) => {{
                lexeme.push($c);
                self.add_token($token_type, lexeme)
            }};
        }

        macro_rules! add_multiple_if_match {
            ($current: expr, $c: expr, $if_match: ident, $else_match: ident) => {{
                lexeme.push($current);
                if self.match_character($c) {
                    lexeme.push($c);
                    self.add_token($if_match, lexeme)
                } else {
                    self.add_token($else_match, lexeme)
                }
            }};
        }

        let current = self.consume_whitespace()?;
        let token = match current {
            b'(' => add_single_byte!(current, LeftParen),
            b')' => add_single_byte!(current, RightParen),
            b'{' => add_single_byte!(current, LeftBrace),
            b'}' => add_single_byte!(current, RightBrace),
            b',' => add_single_byte!(current, Comma),
            b'.' => add_single_byte!(current, Dot),
            b'-' => add_single_byte!(current, Minus),
            b'+' => add_single_byte!(current, Plus),
            b';' => add_single_byte!(current, Semicolon),
            b'*' => add_single_byte!(current, Star),
            b'!' => add_multiple_if_match!(current, b'=', BangEqual, Bang),
            b'=' => add_multiple_if_match!(current, b'=', EqualEqual, Equal),
            b'<' => add_multiple_if_match!(current, b'=', LessEqual, Less),
            b'>' => add_multiple_if_match!(current, b'=', GreaterEqual, Greater),
            b'/' => add_single_byte!(current, Slash),
            b'"' => {
                lexeme.push(current);
                self.consume_string(lexeme)
            }
            b'0'..=b'9' => {
                lexeme.push(current);
                self.consume_number(lexeme)
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                lexeme.push(current);
                self.consume_identifier(lexeme)
            }
            a => Err(ScannerError::UnexpectedCharacter(a)),
        };
        Some(token)
    }

    fn add_token(&mut self, token_type: TokenType, lexeme: Vec<u8>) -> ScannerResult<Token> {
        let lexeme = match String::from_utf8(lexeme) {
            Ok(s) => s,
            Err(_) => return Err(ScannerError::NotUtf8(self.line)),
        };

        Ok(Token::new(token_type, lexeme, self.line))
    }

    fn consume_string(&mut self, mut lexeme: Vec<u8>) -> ScannerResult<Token> {
        let mut completed = false;
        while let Some(c) = self.current_byte {
            match c {
                b'\n' => {
                    self.line += 1;
                    lexeme.push(c);
                    self.advance();
                }
                b'"' => {
                    completed = true;
                    lexeme.push(c);
                    break;
                }
                _ => {
                    lexeme.push(c);
                    self.advance();
                }
            }
        }

        self.advance();

        if self.current_byte.is_none() && !completed {
            return Err(ScannerError::UnterminatedStringLiteral);
        }

        let string = &lexeme[1..lexeme.len() - 1];
        let string = crate::utf8::convert_byte_slice_into_utf8(string);

        self.add_token(TokenType::String(string), lexeme)
    }

    fn consume_number(&mut self, mut lexeme: Vec<u8>) -> ScannerResult<Token> {
        // Parse the first digit.
        let mut decimal: f64 = (lexeme[0] - 0x30) as f64;
        let mut decimal_power = 0;
        let mut current_part = NumberParseSection::Integer;

        while let Some(c) = self.current_byte {
            if c == DECIMAL_SEPARATOR {
                if current_part == NumberParseSection::Decimal {
                    break;
                }
                current_part = NumberParseSection::Decimal;
                self.advance();
                lexeme.push(c);
                continue;
            }

            if !c.is_ascii_digit() {
                break;
            }

            let current_value = (c - 0x30) as f64;
            lexeme.push(c);

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

        self.add_token(TokenType::Number(decimal), lexeme)
    }

    fn consume_identifier(&mut self, mut lexeme: Vec<u8>) -> ScannerResult<Token> {
        while let Some(c) = self.current_byte {
            if !c.is_ascii_alphanumeric() && c != b'_' {
                break;
            }
            lexeme.push(c);
            self.advance();
        }

        let identifier = crate::utf8::convert_byte_slice_into_utf8(&lexeme);

        let token_type = match self.identifier_map.get(&identifier) {
            Some(token_type) => token_type.clone(),
            None => TokenType::Identifier(identifier),
        };

        self.add_token(token_type, lexeme)
    }

    fn consume_whitespace(&mut self) -> Option<u8> {
        loop {
            let current = self.advance()?;
            match current {
                b'\n' | b'\r' => {
                    self.line += 1;
                }
                b' ' | b'\t' => {}
                // Consume comments, if they are there.
                b'/' => {
                    if !self.match_character(b'/') {
                        break Some(current);
                    }
                    while let Some(current) = self.current_byte {
                        if current == b'\n' {
                            break;
                        }
                        self.advance();
                    }
                }

                _ => break Some(current),
            }
        }
    }

    fn match_character(&mut self, other: u8) -> bool {
        let current = match self.current_byte {
            Some(current) => current,
            None => return false,
        };

        if current == other {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<u8> {
        let mut buf = [0u8; 1];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => {
                let current_byte = self.current_byte.take();

                self.current_byte = Some(buf[0]);
                // This will only happen on the last byte
                current_byte
            }
            /*
             * If we have finished reading from the Reader, it is still also possible that
             * we have one single byte remaining on the scanner, which would be the current byte
             */
            Err(_) => self.current_byte.take(),
        }
    }
    pub fn scan_tokens(self) -> ScannerResult<Vec<Token>> {
        let mut tokens = Vec::new();
        for token in self.into_iter() {
            tokens.push(token?);
        }
        Ok(tokens)
    }
}

impl<R: BufRead> Iterator for Scanner<R> {
    type Item = ScannerResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        /*
         * If we have not started reading from the reader, then we need to start parsing
         * the first character.
         */
        if !self.started {
            let mut buf = [0u8; 1];
            match self.reader.read_exact(&mut buf) {
                Ok(_) => self.current_byte = Some(buf[0]),
                Err(_) => return None,
            }

            self.started = true;
        }
        self.scan_token()
    }
}

impl<R: BufRead> std::iter::FusedIterator for Scanner<R> {}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;
    use crate::Token;
    use std::io::Cursor;

    macro_rules! semicolon_token {
        ($line: expr) => {
            Token::new(TokenType::Semicolon, String::from(";"), $line)
        };
    }

    macro_rules! identifier {
        ($lexeme: expr, $line: expr) => {{
            Token::new(
                TokenType::Identifier(String::from($lexeme)),
                String::from($lexeme),
                $line,
            )
        }};
    }

    #[test]
    fn single_byte_tokens() {
        let source = "   =/+-    (){}   ;   // this is a comment that should be ignored.\n = +";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result = scanner.scan_tokens().unwrap();
        assert_eq!(
            result,
            [
                Token::new(TokenType::Equal, String::from("="), 1),
                Token::new(TokenType::Slash, String::from("/"), 1),
                Token::new(TokenType::Plus, String::from("+"), 1),
                Token::new(TokenType::Minus, String::from("-"), 1),
                Token::new(TokenType::LeftParen, String::from("("), 1),
                Token::new(TokenType::RightParen, String::from(")"), 1),
                Token::new(TokenType::LeftBrace, String::from("{"), 1),
                Token::new(TokenType::RightBrace, String::from("}"), 1),
                Token::new(TokenType::Semicolon, String::from(";"), 1),
                Token::new(TokenType::Equal, String::from("="), 2),
                Token::new(TokenType::Plus, String::from("+"), 2),
            ]
        )
    }

    #[test]
    fn single_line_string_literal() {
        let source = "= \"Hello World\"";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.scan_tokens().unwrap();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Equal, String::from("="), 1),
                Token::new(
                    TokenType::String(String::from("Hello World"),),
                    String::from("\"Hello World\""),
                    1
                ),
            ]
        )
    }

    #[test]
    fn multi_line_string_literal() {
        let source = " = \"hello\ncrayon\nlets go\"";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|i| i.unwrap()).collect();
        assert_eq!(
            result,
            [
                Token::new(TokenType::Equal, String::from("="), 1),
                Token::new(
                    TokenType::String(String::from("hello\ncrayon\nlets go"),),
                    String::from("\"hello\ncrayon\nlets go\""),
                    3
                ),
            ]
        )
    }

    #[test]
    fn test_number_parsing() {
        let source = "    30.5    ;    ";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.scan_tokens().unwrap();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Number(30.5), String::from("30.5"), 1),
                semicolon_token!(1)
            ]
        )
    }

    #[test]
    fn test_identifiers() {
        let source = "print\nfoo\nand or bar // sample\nbreak\nfun\nsuper\ncontinue return while";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result = scanner.scan_tokens().unwrap();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Print, String::from("print"), 1),
                Token::new(
                    TokenType::Identifier(String::from("foo")),
                    String::from("foo"),
                    2
                ),
                Token::new(TokenType::And, String::from("and"), 3),
                Token::new(TokenType::Or, String::from("or"), 3),
                Token::new(
                    TokenType::Identifier(String::from("bar")),
                    String::from("bar"),
                    3
                ),
                Token::new(TokenType::Break, String::from("break"), 4),
                Token::new(TokenType::Fun, String::from("fun"), 5),
                Token::new(TokenType::Super, String::from("super"), 6),
                Token::new(TokenType::Continue, String::from("continue"), 7),
                Token::new(TokenType::Return, String::from("return"), 7),
                Token::new(TokenType::While, String::from("while"), 7),
            ]
        )
    }

    #[test]
    fn test_combined_identifiers() {
        let source = "andor\nwhiletrue\nfalsebreak\n oror";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result = scanner.scan_tokens().unwrap();

        assert_eq!(
            result,
            [
                Token::new(
                    TokenType::Identifier(String::from("andor")),
                    String::from("andor"),
                    1
                ),
                Token::new(
                    TokenType::Identifier(String::from("whiletrue")),
                    String::from("whiletrue"),
                    2
                ),
                Token::new(
                    TokenType::Identifier(String::from("falsebreak")),
                    String::from("falsebreak"),
                    3
                ),
                Token::new(
                    TokenType::Identifier(String::from("oror")),
                    String::from("oror"),
                    4
                ),
            ]
        )
    }

    #[test]
    fn test_multibyte_tokens() {
        let source = "== >= <= !=";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|i| i.unwrap()).collect();
        assert_eq!(
            result,
            [
                Token::new(TokenType::EqualEqual, String::from("=="), 1),
                Token::new(TokenType::GreaterEqual, String::from(">="), 1),
                Token::new(TokenType::LessEqual, String::from("<="), 1),
                Token::new(TokenType::BangEqual, String::from("!="), 1),
            ]
        );
    }

    #[test]
    fn test_whitespace_skipping() {
        let source = "     = hola";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|i| i.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Equal, String::from("="), 1,),
                Token::new(
                    TokenType::Identifier(String::from("hola")),
                    String::from("hola"),
                    1
                ),
            ]
        )
    }

    #[test]
    fn test_comment_skip() {
        let source = r#" // C
    print hola; // This is another comment
    print a;"#;
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|i| i.unwrap()).collect();

        assert_eq![
            result,
            [
                Token::new(TokenType::Print, String::from("print"), 2),
                Token::new(
                    TokenType::Identifier(String::from("hola")),
                    String::from("hola"),
                    2
                ),
                semicolon_token!(2),
                Token::new(TokenType::Print, String::from("print"), 3),
                Token::new(
                    TokenType::Identifier(String::from("a")),
                    String::from("a"),
                    3
                ),
                semicolon_token!(3),
            ]
        ]
    }

    #[test]
    fn division_expression() {
        let source = "a / b;";
        let scanner = super::Scanner::new(Cursor::new(source));
        let result: Vec<Token> = scanner.map(|i| i.unwrap()).collect();

        assert_eq!(
            result,
            [
                Token::new(
                    TokenType::Identifier(String::from("a")),
                    String::from("a"),
                    1
                ),
                Token::new(TokenType::Slash, String::from("/"), 1),
                Token::new(
                    TokenType::Identifier(String::from("b")),
                    String::from("b"),
                    1
                ),
                semicolon_token!(1),
            ]
        )
    }

    #[test]
    fn function_declaration_syntax() {
        let source = r#"fun function_example(param1) {
            print param1;
            return "param1";
        }"#;
        let scanner = super::Scanner::new(Cursor::new(source));
        let result = scanner.scan_tokens().unwrap();

        assert_eq!(
            result,
            [
                Token::new(TokenType::Fun, String::from("fun"), 1),
                identifier!("function_example", 1),
                Token::new(TokenType::LeftParen, String::from("("), 1),
                identifier!("param1", 1),
                Token::new(TokenType::RightParen, String::from(")"), 1),
                Token::new(TokenType::LeftBrace, String::from("{"), 1),
                Token::new(TokenType::Print, String::from("print"), 2),
                identifier!("param1", 2),
                semicolon_token!(2),
                Token::new(TokenType::Return, String::from("return"), 3),
                Token::new(
                    TokenType::String(String::from("param1")),
                    String::from("\"param1\""),
                    3
                ),
                semicolon_token!(3),
                Token::new(TokenType::RightBrace, String::from("}"), 4),
            ]
        )
    }
}
