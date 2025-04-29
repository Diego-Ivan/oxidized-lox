use crate::interpreter::environment::Environment;
use crate::interpreter::{LoxValue, NativeResult};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use syntax::statement::Block;
use syntax::token::Token;

pub type NativeFunc = fn(args: &[LoxValue]) -> NativeResult<LoxValue>;

#[derive(Clone)]
pub enum Callable {
    Native {
        func: NativeFunc,
        arity: usize,
    },
    LoxFunction {
        closure: Rc<RefCell<Environment>>,
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
