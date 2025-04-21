use crate::interpreter::statement::Statement;
use crate::token::{Token, TokenType};
use crate::Expression;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Expected: {0:?}")]
    FailedMatch(TokenType),
    #[error("Expected token: {0:?}.")]
    ExpectExpression(Token),
    #[error("Invalid assignment target: {0:?}.")]
    InvalidAssignmentTarget(Expression),
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

macro_rules! consume_token {
    ($parser: ident, $pattern: pat, $error: expr) => {
        match $parser.peek() {
            Some(next_token) => {
                if matches!(next_token.token_type(), $pattern) {
                    $parser.advance();
                    Ok(*next_token)
                } else {
                    Err($error)
                }
            }
            None => Err($error),
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn statements(&mut self) -> ParserResult<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> ParserResult<Statement> {
        if match_token!(self, TokenType::Var) {
            /* Synchronize if parsing a variable declaration failed */
            self.variable_declaration()
                .inspect_err(|_| self.synchronize())
        } else {
            self.parse_statement()
        }
    }

    fn variable_declaration(&mut self) -> ParserResult<Statement> {
        let current_token = self.peek().unwrap();
        let name = if let TokenType::Identifier(ident) = current_token.token_type() {
            let ident = ident.clone();
            self.advance();
            ident
        } else {
            return Err(ParserError::FailedMatch(TokenType::Identifier(
                String::new(),
            )));
        };

        let initializer = if match_token!(self, TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        if match_token!(self, TokenType::Semicolon) {
            Ok(Statement::Declaration { name, initializer })
        } else {
            Err(ParserError::FailedMatch(TokenType::Semicolon))
        }
    }

    fn parse_statement(&mut self) -> ParserResult<Statement> {
        if match_token!(self, TokenType::Print) {
            self.parse_print_statement()
        } else if match_token!(self, TokenType::LeftBrace) {
            self.parse_block()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.expression()?;
        if match_token!(self, TokenType::Semicolon) {
            Ok(Statement::Expression(expression))
        } else {
            Err(ParserError::FailedMatch(TokenType::Semicolon))
        }
    }

    fn parse_print_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.expression()?;
        if match_token!(self, TokenType::Semicolon) {
            Ok(Statement::Print(expression))
        } else {
            Err(ParserError::FailedMatch(TokenType::Semicolon))
        }
    }

    fn parse_block(&mut self) -> ParserResult<Statement> {
        let mut statements = Vec::new();

        while !matches!(self.peek().unwrap().token_type(), TokenType::RightBrace)
            && self.is_at_end()
        {
            statements.push(self.declaration()?);
        }

        self.advance();

        if match_token!(self, TokenType::RightBrace) {
            Ok(Statement::Block(statements))
        } else {
            Err(ParserError::FailedMatch(TokenType::RightBrace))
        }
    }

    pub fn parse(&mut self) -> ParserResult<Expression> {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.equality()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expr = self.equality()?;

        if match_token!(self, TokenType::Equal) {
            let equals = self.previous().unwrap().clone();
            let value_expr = self.assignment()?;

            if let Expression::Var { name, token } = &value_expr {
                Ok(Expression::Assignment {
                    name: name.to_string(),
                    token: equals,
                    value: Box::new(value_expr),
                })
            } else {
                Err(ParserError::InvalidAssignmentTarget(value_expr))
            }
        } else {
            Ok(expr)
        }
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
            TokenType::Identifier(name) => {
                let expression = Expression::Var {
                    name: String::from(name),
                    token: self.peek().unwrap().clone(),
                };
                self.advance();
                Ok(expression)
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

    fn synchronize(&mut self) {
        use TokenType::*;

        self.advance();

        while !self.is_at_end() {
            if let Some(token) = self.previous() {
                if matches!(token.token_type(), Semicolon) {
                    return;
                }
            }

            let next = self.peek().unwrap().token_type();
            if matches!(next, Class | Fun | Var | For | If | While | Print | Return) {
                return;
            }
        }
        self.advance();
    }
}
