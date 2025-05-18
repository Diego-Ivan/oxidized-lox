use super::LoxValue;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct InterpreterError {
    pub error_type: InterpreterErrorType,
    pub token: syntax::Token,
}

#[derive(Debug)]
pub enum InterpreterErrorType {
    WrongUnaryOperands(syntax::token::TokenType, LoxValue),
    WrongBinaryOperands(LoxValue, syntax::token::TokenType, LoxValue),
    DivisionByZero,
    UndefinedVariable(String),
    NotACallable,
    WrongArity { original: usize, user: usize },
    Native(NativeError),
    NotInLoop,
}

pub type InterpreterResult<T> = Result<T, Box<InterpreterError>>;

#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error parsing number: {0}")]
    NumParse(#[from] std::num::ParseFloatError),
    #[error("System Time Error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

pub type NativeResult<T> = Result<T, NativeError>;

impl Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_message = match &self.error_type {
            InterpreterErrorType::WrongUnaryOperands(op, t) => {
                format!("The unary operation {op:?} is not valid over token of type: {t}")
            }
            InterpreterErrorType::DivisionByZero => String::from("Division by zero"),
            InterpreterErrorType::WrongBinaryOperands(t1, op, t2) => {
                format!(
                    "Operation of type: {op:?} cannot be applied over operands of types {t1:?} and {t2:?}"
                )
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
            InterpreterErrorType::Native(err) => {
                format!("Native Error - {err}")
            }
            InterpreterErrorType::NotInLoop => {
                format!("Used {} statement outside a loop", self.token.lexeme())
            }
        };

        write!(f, "{err_message}\n[line {}]", self.token.line())
    }
}

impl std::error::Error for InterpreterError {}
