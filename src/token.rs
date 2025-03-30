use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum TokenType {
    /* Single character tokens */
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    /* 1-2 character tokens */
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    /* Literals */
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: Implement literal reading
        write!(f, "{:?} {} ", self.token_type, self.lexeme)
    }
}
