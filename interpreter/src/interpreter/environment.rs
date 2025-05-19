use crate::interpreter::value::LoxValue;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
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

    pub fn assign_at(&mut self, name: &str, value: LoxValue, distance: usize) -> bool {
        match self.ancestor(distance) {
            Some(ancestor) => {
                if let Entry::Occupied(mut entry) =
                    ancestor.borrow_mut().values.entry(String::from(name))
                {
                    entry.insert(value);
                    true
                } else {
                    false
                }
            }
            // If the return value is None, then the environment is self
            None => {
                if let Entry::Occupied(mut entry) = self.values.entry(String::from(name)) {
                    entry.insert(value);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn get_at(&self, name: &str, distance: usize) -> Option<LoxValue> {
        if distance == 0 {
            return self.values.get(name).cloned();
        }

        match self.ancestor(distance) {
            Some(env) => env.borrow().values.get(name).cloned(),
            None => self.values.get(name).cloned(),
        }
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let mut environment: Option<Rc<RefCell<Environment>>> = self.enclosing.clone();

        for _ in 0..distance {
            environment = match environment {
                Some(env) => env.borrow().enclosing.clone(),
                None => None,
            }
        }

        environment
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
}
