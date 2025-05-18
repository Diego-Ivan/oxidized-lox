use crate::interpreter::callable::Callable;
use std::cell::RefCell;
use std::collections::HashMap;
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
    fields: RefCell<HashMap<String, LoxValue>>,
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
        Self {
            class,
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<LoxValue> {
        self.fields.borrow().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: LoxValue) {
        self.fields.borrow_mut().insert(key.to_string(), value);
    }

    pub fn class_name(&self) -> &str {
        &self.class.name
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "instanceof({})", &self.class.name)
    }
}
