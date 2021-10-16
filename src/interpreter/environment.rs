use super::RuntimeError;
use crate::{token::Token, types::LiteralType};
use hashbrown::HashMap;

#[derive()]
pub struct EnvironmentManger {
    index: usize,
    environments: Vec<Environment>,
}

#[derive()]
pub struct Environment {
    values: HashMap<String, LiteralType>,
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
        let global_env = Environment::new();

        EnvironmentManger {
            index: 0,
            environments: vec![global_env],
        }
    }

    pub fn define(&mut self, name: String, value: LiteralType) {
        self.environments[0].values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: LiteralType) -> Result<(), RuntimeError> {
        let environment = &mut self.environments[self.index];

        if environment.values.contains_key(&name.lexeme) {
            environment.values.insert(name.lexeme, value);

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

    pub fn get(&mut self, name: &Token) -> Result<LiteralType, RuntimeError> {
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
