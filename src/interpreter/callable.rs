use crate::interpreter::statement::Block;
use crate::interpreter::{InterpreterResult, LoxValue};
use crate::token::Token;
use std::fmt::{Debug, Formatter};

pub type NativeFunc = fn(args: &[LoxValue]) -> InterpreterResult<LoxValue>;

#[derive(Clone)]
pub enum Callable {
    Native {
        func: NativeFunc,
        arity: usize,
    },
    LoxFunction {
        name: String,
        params: Vec<Token>,
        block: Block,
    },
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { func: _, arity: _ } => f.write_str("<native fun>"),
            Self::LoxFunction { name, .. } => write!(f, "<fun {name}>"),
        }
    }
}
