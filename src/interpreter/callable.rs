use crate::interpreter::statement::Block;
use crate::interpreter::{Interpreter, InterpreterResult, LoxValue};
use crate::token::Token;

pub type NativeFunc<'a> =
    fn(interpreter: &Interpreter, args: &'a [Token]) -> InterpreterResult<'a, LoxValue>;

#[derive(Debug)]
pub enum Callable<'a> {
    Native {
        func: NativeFunc<'a>,
        args: Vec<Token>,
    },
    LoxFunction {
        block: Block,
        arguments: Vec<Token>,
    },
}

impl<'a> Callable<'a> {
    pub fn call(&'a self, interpreter: &'a Interpreter) -> InterpreterResult<'a, LoxValue> {
        match self {
            Self::Native { func, args } => func(interpreter, args),
            Self::LoxFunction {
                block: _,
                arguments: _,
            } => todo!(),
        }
    }
}
