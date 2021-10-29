use std::{io::Write, rc::Rc};

use crate::{error::RuntimeError, token::Token, types::LiteralType};
use fnv::FnvHashMap;

pub struct EnvironmentManger {
    index: usize,
    environments: Vec<Environment>,
}

pub struct Environment {
    values: FnvHashMap<String, LiteralType>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: FnvHashMap::default(),
        }
    }
}

impl EnvironmentManger {
    pub fn new() -> EnvironmentManger {
        let mut global_env = Environment::new();

        global_env.values.insert(
            "print".to_owned(),
            LiteralType::NativeFunction {
                name: "print".to_owned(),
                arity: 1,
                func: Rc::new(|args, _paren| {
                    let value = &args[0];
                    print!("{}", get_type_string(value));
                    std::io::stdout().flush().unwrap();
                    Ok(LiteralType::Nil)
                }),
            },
        );

        global_env.values.insert(
            "println".to_owned(),
            LiteralType::NativeFunction {
                name: "println".to_owned(),
                arity: 1,
                func: Rc::new(|args, _paren| {
                    let value = &args[0];
                    println!("{}", get_type_string(value));
                    Ok(LiteralType::Nil)
                }),
            },
        );

        global_env.values.insert(
            "input".to_owned(),
            LiteralType::NativeFunction {
                name: "input".to_owned(),
                arity: 0,
                func: Rc::new(|_args, _paren| {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();

                    Ok(LiteralType::String(input))
                }),
            },
        );

        global_env.values.insert(
            "input_prompt".to_owned(),
            LiteralType::NativeFunction {
                name: "input_prompt".to_owned(),
                arity: 1,
                func: Rc::new(|args, paren| {
                    let prompt = match &args[0] {
                        LiteralType::String(value) => value,
                        _ => return Err(RuntimeError::new(paren, "expected a string".to_owned())),
                    };

                    print!("{}", prompt);
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();

                    Ok(LiteralType::String(input[0..input.len() - 1].to_owned()))
                }),
            },
        );

        global_env.values.insert(
            "int".to_owned(),
            LiteralType::NativeFunction {
                name: "int".to_owned(),
                arity: 1,
                func: Rc::new(|args, paren| {
                    let error = RuntimeError::new(paren, "cannot cast to int".to_owned());

                    match &args[0] {
                        LiteralType::Number(value) => Ok(LiteralType::Number(*value as i32 as f32)),
                        LiteralType::String(value) => match value.parse::<f32>() {
                            Ok(value) => Ok(LiteralType::Number(value as i32 as f32)),
                            Err(_) => return Err(error),
                        },
                        _ => Err(error),
                    }
                }),
            },
        );

        global_env.values.insert(
            "str".to_owned(),
            LiteralType::NativeFunction {
                name: "str".to_owned(),
                arity: 1,
                func: Rc::new(|args, paren| {
                    let error = RuntimeError::new(paren, "cannot cast to int".to_owned());

                    match &args[0] {
                        LiteralType::Number(value) => Ok(LiteralType::String(format!("{}", value))),
                        LiteralType::String(value) => Ok(LiteralType::String(format!("{}", value))),
                        _ => Err(error),
                    }
                }),
            },
        );

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
            format!("'{}' variable undefined", name.lexeme),
        ))
    }

    pub fn get(&mut self, name: &Token) -> Result<LiteralType, RuntimeError> {
        let environment = self.environments.get(self.index).unwrap();

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
            format!("'{}' is undefined", name.lexeme),
        ))
    }

    pub fn add_env(&mut self) {
        self.environments.insert(0, Environment::new());
    }

    pub fn remove_env(&mut self) {
        self.environments.remove(0);
    }
}

fn get_type_string(value: &LiteralType) -> String {
    match value {
        LiteralType::Nil => "nil".to_owned(),
        LiteralType::Number(number_value) => format!("{}", number_value),
        LiteralType::String(string_value) => format!("{}", string_value),
        LiteralType::Bool(bool_value) => format!("{}", bool_value),
        LiteralType::Function { deceleration } => {
            let (name, _, _) = deceleration.as_function_decl().unwrap();

            format!("<fn {}>", name.lexeme)
        }
        LiteralType::NativeFunction {
            name,
            arity: _,
            func: _,
        } => format!("<fn {}>", name),
    }
}
