use std::{cell::RefCell, rc::Rc};

use crate::{error::RuntimeError, token::Token, types::LiteralType};
use fnv::FnvHashMap;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: FnvHashMap<String, LiteralType>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            enclosing: None,
            values: FnvHashMap::default(),
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing),
            values: FnvHashMap::default(),
        }))
    }

    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: LiteralType) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);

            return Ok(());
        }

        if let Some(ecn) = &self.enclosing {
            return ecn.borrow_mut().assign(name, value);
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("'{}' variable undefined", name.lexeme),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<LiteralType, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.to_owned());
        }

        if let Some(ecn) = &self.enclosing {
            return ecn.borrow().get(name);
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("'{}' is undefined", name.lexeme),
        ))
    }
}
