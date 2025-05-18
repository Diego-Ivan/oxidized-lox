use crate::expression::Expression;
use crate::statement;
use crate::statement::{Block, Statement};
use crate::token::{Token, TokenType};
use ordered_float::OrderedFloat;
use thiserror::Error;

const MAX_ARGS: usize = 255;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Expected: {0:?}")]
    FailedMatch(TokenType),
    #[error("Invalid assignment target: {0:?}.")]
    InvalidAssignmentTarget(Expression),
    #[error("Token {0:?} has too many arguments (max: {MAX_ARGS})")]
    TooManyArgs(Token),
}

type ParserResult<T> = Result<T, ParserError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

macro_rules! match_token {
    ($parser: ident, $pattern: pat) => {{
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
    }};
}

macro_rules! check_token {
    ($parser: ident, $pattern: pat) => {{
        match $parser.peek() {
            Some(next_token) => {
                if matches!(next_token.token_type(), $pattern) {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }};
}

macro_rules! expect_token {
    ($parser: ident, $pattern: pat, $token_type: ident) => {{
        if !(match_token!($parser, $pattern)) {
            return Err(ParserError::FailedMatch(TokenType::$token_type));
        }
    }};
}

macro_rules! expect_token_with_param {
    ($parser: ident, $pattern: pat, $token_type: ident, $params: expr) => {{
        {
            if !(match_token!($parser, $pattern)) {
                return Err(ParserError::FailedMatch(TokenType::$token_type($params)));
            }
            $parser.previous().unwrap()
        }
    }};
}

macro_rules! expect_identifier {
    ($parser: ident) => {{
        expect_token_with_param!(
            $parser,
            TokenType::Identifier(_),
            Identifier,
            String::from("undefined")
        )
    }};
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
        if match_token!(self, TokenType::Fun) {
            Ok(Statement::FunctionDeclaration(self.function_declaration()?))
        } else if match_token!(self, TokenType::Var) {
            /* Synchronize if parsing a variable declaration failed */
            self.variable_declaration().inspect_err(|e| {
                eprintln!("{e}");
                self.synchronize();
            })
        } else if match_token!(self, TokenType::Class) {
            self.class_declaration()
        } else {
            self.parse_statement()
        }
    }

    fn class_declaration(&mut self) -> ParserResult<Statement> {
        let name = expect_identifier!(self).lexeme().to_string();
        expect_token!(self, TokenType::LeftBrace, LeftBrace);

        let mut methods = Vec::new();

        while !check_token!(self, TokenType::RightBrace) {
            methods.push(self.function_declaration()?);
        }

        expect_token!(self, TokenType::RightBrace, RightBrace);

        Ok(Statement::ClassDeclaration { name, methods })
    }

    fn function_declaration(&mut self) -> ParserResult<statement::Function> {
        let name = expect_identifier!(self).lexeme().to_string();

        expect_token!(self, TokenType::LeftParen, LeftParen);

        let mut parameters = Vec::new();
        if !check_token!(self, TokenType::RightParen) {
            let ident = expect_identifier!(self).clone();
            parameters.push(ident);

            while match_token!(self, TokenType::Comma) {
                if parameters.len() >= MAX_ARGS {
                    eprintln!("{}", ParserError::TooManyArgs(self.peek().unwrap().clone()));
                    break;
                }

                let ident = expect_identifier!(self).clone();
                parameters.push(ident);
            }
        }

        expect_token!(self, TokenType::RightParen, RightParen);

        expect_token!(self, TokenType::LeftBrace, LeftBrace);
        let body = self.parse_block()?;

        Ok(statement::Function {
            name,
            parameters,
            body,
        })
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

        expect_token!(self, TokenType::Semicolon, Semicolon);
        Ok(Statement::VariableDeclaration { name, initializer })
    }

    fn parse_statement(&mut self) -> ParserResult<Statement> {
        let token = self.peek().unwrap();

        match token.token_type() {
            TokenType::Print => {
                self.advance();
                self.parse_print_statement()
            }
            TokenType::LeftBrace => {
                self.advance();
                self.parse_block_statement()
            }
            TokenType::If => {
                self.advance();
                self.parse_if_statement()
            }
            TokenType::For => {
                self.advance();
                self.parse_for_statement()
            }
            TokenType::While => {
                self.advance();
                self.parse_while_statement()
            }
            TokenType::Return => {
                self.advance();
                self.parse_return_statement()
            }
            TokenType::Break => {
                let keyword = token.clone();

                self.advance();
                expect_token!(self, TokenType::Semicolon, Semicolon);

                Ok(Statement::Break { keyword })
            }
            TokenType::Continue => {
                let keyword = token.clone();

                self.advance();
                expect_token!(self, TokenType::Semicolon, Semicolon);

                Ok(Statement::Continue { keyword })
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.expression()?;
        expect_token!(self, TokenType::Semicolon, Semicolon);

        Ok(Statement::Expression(expression))
    }

    fn parse_print_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.expression()?;
        expect_token!(self, TokenType::Semicolon, Semicolon);

        Ok(Statement::Print(expression))
    }

    fn parse_block(&mut self) -> ParserResult<Block> {
        let mut statements = Vec::new();

        while !(matches!(self.peek().unwrap().token_type(), TokenType::RightBrace))
            && !self.is_at_end()
        {
            statements.push(self.declaration()?);
        }

        expect_token!(self, TokenType::RightBrace, RightBrace);

        Ok(statements)
    }

    fn parse_block_statement(&mut self) -> ParserResult<Statement> {
        Ok(Statement::Block(self.parse_block()?))
    }

    fn parse_if_statement(&mut self) -> ParserResult<Statement> {
        expect_token!(self, TokenType::LeftParen, LeftParen);
        let condition = self.expression()?;
        expect_token!(self, TokenType::RightParen, RightParen);

        let then_branch = self.parse_statement()?;

        let else_branch = if match_token!(self, TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> ParserResult<Statement> {
        expect_token!(self, TokenType::LeftParen, LeftParen);
        let condition = self.expression()?;
        expect_token!(self, TokenType::RightParen, RightParen);

        let body = self.parse_statement()?;

        Ok(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    fn parse_for_statement(&mut self) -> ParserResult<Statement> {
        expect_token!(self, TokenType::LeftParen, LeftParen);

        let initializer = if match_token!(self, TokenType::Semicolon) {
            None
        } else if match_token!(self, TokenType::Var) {
            Some(Box::new(self.variable_declaration()?))
        } else {
            Some(Box::new(self.parse_expression_statement()?))
        };

        let condition = if match_token!(self, TokenType::Semicolon) {
            None
        } else {
            let expr = Some(self.expression()?);
            expect_token!(self, TokenType::Semicolon, Semicolon);

            expr
        };

        let increment = if match_token!(self, TokenType::RightParen) {
            None
        } else {
            let inc = Some(self.expression()?);
            expect_token!(self, TokenType::RightParen, RightParen);
            inc
        };

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For {
            initializer,
            condition,
            increment,
            body,
        })
    }

    fn parse_return_statement(&mut self) -> ParserResult<Statement> {
        let keyword = self.previous().unwrap().clone();
        let expression = if !check_token!(self, TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        expect_token!(self, TokenType::Semicolon, Semicolon);

        Ok(Statement::Return {
            expression,
            keyword,
        })
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expr = self.or()?;

        if match_token!(self, TokenType::Equal) {
            let equals = self.previous().unwrap().clone();
            let value_expr = self.assignment()?;

            match expr {
                Expression::Var { name, token: _ } => Ok(Expression::Assignment {
                    name,
                    value: Box::new(value_expr),
                    token: equals.clone(),
                }),
                Expression::Get { token, expression } => Ok(Expression::Assignment {
                    name: token.lexeme().to_string(),
                    value: expression,
                    token: equals.clone(),
                }),
                _ => Err(ParserError::InvalidAssignmentTarget(value_expr)),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> ParserResult<Expression> {
        let mut expr = self.and()?;

        while match_token!(self, TokenType::Or) {
            let right = self.and()?;
            expr = Expression::Or {
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expression> {
        let mut expr = self.equality()?;

        while match_token!(self, TokenType::And) {
            let right = self.equality()?;
            expr = Expression::And {
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
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
        self.call()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        let mut expr = self.primary()?;
        loop {
            if match_token!(self, TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if match_token!(self, TokenType::Dot) {
                let identifier = expect_identifier!(self);
                expr = Expression::Get {
                    expression: Box::new(expr),
                    token: identifier.clone(),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expression) -> ParserResult<Expression> {
        let mut args = Vec::new();

        if !check_token!(self, TokenType::RightParen) {
            args.push(self.expression()?);

            while match_token!(self, TokenType::Comma) {
                args.push(self.expression()?);

                if args.len() >= MAX_ARGS {
                    eprintln!("{}", ParserError::TooManyArgs(self.peek().unwrap().clone()));
                    break;
                }
            }
        }

        expect_token!(self, TokenType::RightParen, RightParen);
        let token = self.previous().unwrap().clone();

        Ok(Expression::Call {
            callee: Box::new(expr),
            paren: token,
            args,
        })
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
                let expr = Expression::Number(OrderedFloat(**num));
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
        self.current >= self.tokens.len()
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
            if matches!(
                next,
                Class | Fun | Var | For | If | While | Print | Return | Continue
            ) {
                return;
            }
            self.advance();
        }
    }
}
