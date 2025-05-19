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
    pub is_initializer: bool,
    pub params: Vec<Token>,
    pub block: Block,
}

#[derive(Clone)]
pub enum Callable {
    Native {
        func: NativeFunc,
        arity: usize,
    },
    LoxFunction(LoxFunction),
    Constructor {
        class: Rc<super::value::Class>,
        arity: usize,
    },
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Self::Native { arity, .. } => *arity,
            Self::LoxFunction(function) => function.params.len(),
            Self::Constructor { arity, .. } => *arity,
        }
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { func: _, arity: _ } => f.write_str("<native fun>"),
            Self::LoxFunction(function) => write!(f, "<fun {}>", function.name),
            Self::Constructor { class, .. } => write!(f, "<constructor {class}>"),
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
            is_initializer: true,
            params: self.params.clone(),
            block: self.block.clone(),
        }
    }
}
