use crate::interpreter::value::LoxValue;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&LoxValue> {
        self.values.get(name)
    }

    pub fn set(&mut self, name: String, value: LoxValue) -> bool {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            true
        } else {
            false
        }
    }
}
