use std::{any::Any, collections::HashMap, rc::Rc};

use crate::token::Token;

use super::RuntimeError;

#[derive(Debug, Clone)]
pub struct EnvironmentManger {
    index: usize,
    environments: Vec<Environment>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Rc<dyn Any>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }
}

impl EnvironmentManger {
    pub fn new() -> EnvironmentManger {
        EnvironmentManger {
            index: 0,
            environments: vec![Environment::new()],
        }
    }

    pub fn define(&mut self, name: String, value: Rc<dyn Any>) {
        self.environments[0].values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: Rc<dyn Any>) -> Result<(), RuntimeError> {
        let environment = &mut self.environments[self.index];

        if environment.values.contains_key(&name.lexeme) {
            environment.values.insert(name.lexeme, value.to_owned());

            return Ok(());
        }

        if self.index < self.environments.len() {
            self.index += 1;
            self.assign(name, value)?;
            self.index -= 1;

            return Ok(());
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("undefined variable '{}'", name.lexeme),
        ))
    }

    pub fn get(&mut self, name: &Token) -> Result<Rc<dyn Any>, RuntimeError> {
        let environment = &self.environments[self.index];

        if let Some(value) = environment.values.get(&name.lexeme) {
            return Ok(value.to_owned());
        }

        self.index += 1;
        if self.index < self.environments.len() {
            let value = self.get(name)?;
            self.index -= 1;

            return Ok(value);
        }

        Err(RuntimeError::new(
            name.to_owned(),
            format!("undefined variable '{}'", name.lexeme),
        ))
    }

    pub fn add_env(&mut self) {
        self.environments.insert(0, Environment::new());
    }

    pub fn remove_env(&mut self) {
        self.environments.remove(0);
    }
}
