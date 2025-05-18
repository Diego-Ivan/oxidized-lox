use crate::interpreter::callable::Callable;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
    Callable(Rc<Callable>),
    Instance(Rc<Instance>),
}

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
}

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(b) => *b,
            Self::Number(0.0) => false,
            Self::Number(_) => true,
            Self::String(_) => true,
            Self::Callable(_) => true,
            Self::Instance(_) => true,
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(str) => f.write_str(str),
            Self::Callable(callable) => Debug::fmt(callable, f),
            Self::Instance(instance) => Display::fmt(instance, f),
        }
    }
}

impl Class {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self { class }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "instanceof({})", &self.class.name)
    }
}
