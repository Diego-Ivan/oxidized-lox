use crate::interpreter::value::LoxValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, LoxValue>,
    enclosing: Option<Rc<RefCell<Self>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosed(enclosing: Rc<RefCell<Self>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Self::new()
        }
    }

    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<LoxValue> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => match self.enclosing.clone() {
                Some(enclosing) => enclosing.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: LoxValue) -> bool {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.values.entry(name) {
            e.insert(value);
            true
        } else {
            false
        }
    }
}
