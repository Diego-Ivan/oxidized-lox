use crate::interpreter::environment::Environment;
use crate::interpreter::{LoxValue, NativeResult};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use syntax::statement::Block;
use syntax::token::Token;

use super::value::Instance;

pub type NativeFunc = fn(args: &[LoxValue]) -> NativeResult<LoxValue>;

#[derive(Clone)]
pub struct LoxFunction {
    pub closure: Rc<RefCell<Environment>>,
    pub name: String,
    pub params: Vec<Token>,
    pub block: Block,
}

#[derive(Clone)]
pub enum Callable {
    Native { func: NativeFunc, arity: usize },
    LoxFunction(LoxFunction),
    Constructor(Rc<super::value::Class>),
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { func: _, arity: _ } => f.write_str("<native fun>"),
            Self::LoxFunction(function) => write!(f, "<fun {}>", function.name),
            Self::Constructor(name) => write!(f, "<constructor {name}>"),
        }
    }
}

impl LoxFunction {
    pub fn bind(&self, instance: Rc<Instance>) -> LoxFunction {
        let mut environment = Environment::new_enclosed(self.closure.clone());
        environment.define(String::from("this"), LoxValue::Instance(instance.clone()));

        LoxFunction {
            closure: Rc::new(RefCell::new(environment)),
            name: self.name.to_string(),
            params: self.params.clone(),
            block: self.block.clone(),
        }
    }
}
