use crate::token::{Token, TokenType};
use crate::Expression;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Expected: {0:?}")]
    FailedMatch(TokenType),
    #[error("Expected expression")]
    ExpectExpression(Token),
}

type ParserResult<T> = Result<T, ParserError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

macro_rules! match_token {
    ($parser: ident, $pattern: pat) => {
        match $parser.peek() {
            Some(next_token) => {
                if matches!(next_token.token_type(), $pattern) {
                    $parser.advance();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParserResult<Expression> {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult<Expression> {
        let mut expression = self.comparison()?;

        while match_token!(self, TokenType::BangEqual | TokenType::EqualEqual) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };
            let right = self.comparison()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> ParserResult<Expression> {
        let mut expression = self.term()?;

        while match_token!(
            self,
            TokenType::GreaterEqual | TokenType::Greater | TokenType::Less | TokenType::LessEqual
        ) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };
            let right = self.term()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn term(&mut self) -> ParserResult<Expression> {
        let mut expression = self.factor()?;
        while match_token!(self, TokenType::Minus | TokenType::Plus) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };

            let right = self.factor()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn factor(&mut self) -> ParserResult<Expression> {
        let mut expression = self.unary()?;

        while match_token!(self, TokenType::Slash | TokenType::Star) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => break,
            };

            let right = self.unary()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn unary(&mut self) -> ParserResult<Expression> {
        if match_token!(self, TokenType::Bang | TokenType::Minus) {
            let operator = match self.previous() {
                Some(operator) => operator.clone(),
                None => panic!("Expected finding an operator while parsing an unary expression"),
            };
            let right = self.unary()?;
            return Ok(Expression::Unary(operator, Box::new(right)));
        }
        self.primary()
    }

    fn get_current(&self) -> &Token {
        self.peek().unwrap()
    }
    fn primary(&mut self) -> ParserResult<Expression> {
        match self.peek().unwrap().token_type() {
            TokenType::False => {
                self.advance();
                Ok(Expression::False)
            }
            TokenType::True => {
                self.advance();
                Ok(Expression::True)
            }
            TokenType::Nil => {
                self.advance();
                Ok(Expression::Nil)
            }
            TokenType::Number(num) => {
                let expr = Expression::Number(*num);
                self.advance();
                Ok(expr)
            }
            TokenType::String(str) => {
                let expr = Expression::String(str.clone());
                self.advance();
                Ok(expr)
            }
            TokenType::LeftParen => {
                self.advance();

                let expression = self.expression()?;

                if match_token!(self, TokenType::RightParen) {
                    Ok(Expression::Grouping(Box::new(expression)))
                } else {
                    Err(ParserError::FailedMatch(TokenType::RightParen))
                }
            }
            a => Err(ParserError::FailedMatch(a.clone())),
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            None
        } else {
            Some(&self.tokens[self.current - 1])
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.tokens[self.current].token_type(), TokenType::Eof)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
}
