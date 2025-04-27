use super::LoxValue;
use crate::token::{Token, TokenType};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct InterpreterError<'a> {
    pub error_type: InterpreterErrorType<'a>,
    pub token: &'a Token,
}

impl Display for InterpreterError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_message = match &self.error_type {
            InterpreterErrorType::WrongUnaryOperands(op, t) => {
                format!("The unary operation {op:?} is not valid over token of type: {t}")
            }
            InterpreterErrorType::DivisionByZero => String::from("Division by zero"),
            InterpreterErrorType::WrongBinaryOperands(t1, op, t2) => {
                format!("Operation of type: {op:?} cannot be applied over operands of types {t1:?} and {t2:?}")
            }
            InterpreterErrorType::UndefinedVariable(name) => {
                format!("Variable {name} is undefined")
            }
            InterpreterErrorType::NotACallable => {
                format!(
                    "Value {} at line {} is not a callable",
                    self.token.lexeme(),
                    self.token.line()
                )
            }
            InterpreterErrorType::WrongArity { original, user } => {
                format!(
                    "Function {} called with {user} arguments, but required {original}",
                    self.token.lexeme()
                )
            }
        };

        write!(f, "{err_message}\n[line {}]", self.token.line())
    }
}

impl std::error::Error for InterpreterError<'_> {}

#[derive(Debug)]
pub enum InterpreterErrorType<'a> {
    WrongUnaryOperands(&'a TokenType, LoxValue),
    WrongBinaryOperands(LoxValue, &'a TokenType, LoxValue),
    DivisionByZero,
    UndefinedVariable(String),
    NotACallable,
    WrongArity { original: usize, user: usize },
}

pub type InterpreterResult<'a, T> = Result<T, InterpreterError<'a>>;
