use crate::interpreter::{Interpreter, InterpreterResult, LoxValue, Statement};
use crate::token::Token;

pub type NativeFunc<'a> =
    fn(interpreter: &'a Interpreter, args: &[Token]) -> InterpreterResult<'a, LoxValue>;

#[derive(Debug)]
pub enum Callable<'a> {
    Native {
        func: NativeFunc<'a>,
        args: Vec<Token>,
    },
    LoxFunction {
        statement: Statement,
        arguments: Vec<Token>,
    },
}

impl<'a> Callable<'a> {
    pub fn call(&self, interpreter: &'a Interpreter) -> InterpreterResult<'a, LoxValue> {
        match self {
            Self::Native { func, args } => func(interpreter, args),
            Self::LoxFunction {
                statement: _,
                arguments: _,
            } => todo!(),
        }
    }
}
