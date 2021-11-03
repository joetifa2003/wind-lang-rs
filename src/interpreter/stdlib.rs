use std::{io::Write, rc::Rc};

use crate::{error::RuntimeError, types::LiteralType};

pub trait StdLibFunc {
    fn name() -> String;
    fn function() -> LiteralType;
}

pub struct Print;

impl StdLibFunc for Print {
    fn name() -> String {
        "print".to_owned()
    }

    fn function() -> LiteralType {
        LiteralType::NativeFunction {
            name: "print".to_owned(),
            arity: 1,
            func: Rc::new(|args, _paren| {
                let value = &args[0];
                print!("{}", value);
                std::io::stdout().flush().unwrap();
                Ok(LiteralType::Nil)
            }),
        }
    }
}

pub struct PrintLn;

impl StdLibFunc for PrintLn {
    fn name() -> String {
        "println".to_owned()
    }

    fn function() -> LiteralType {
        LiteralType::NativeFunction {
            name: "println".to_owned(),
            arity: 1,
            func: Rc::new(|args, _paren| {
                let value = &args[0];
                println!("{}", value);
                Ok(LiteralType::Nil)
            }),
        }
    }
}

pub struct Input;

impl StdLibFunc for Input {
    fn name() -> String {
        "input".to_owned()
    }

    fn function() -> LiteralType {
        LiteralType::NativeFunction {
            name: "input".to_owned(),
            arity: 0,
            func: Rc::new(|_args, _paren| {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                Ok(LiteralType::String(input))
            }),
        }
    }
}

pub struct InputPrompt;

impl StdLibFunc for InputPrompt {
    fn name() -> String {
        "input_prompt".to_owned()
    }

    fn function() -> LiteralType {
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
        }
    }
}

pub struct Int;

impl StdLibFunc for Int {
    fn name() -> String {
        "int".to_owned()
    }

    fn function() -> LiteralType {
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
        }
    }
}

pub struct Str;

impl StdLibFunc for Str {
    fn name() -> String {
        "str".to_owned()
    }

    fn function() -> LiteralType {
        LiteralType::NativeFunction {
            name: "str".to_owned(),
            arity: 1,
            func: Rc::new(|args, paren| {
                let error = RuntimeError::new(paren, "cannot cast to string".to_owned());

                match &args[0] {
                    LiteralType::Number(value) => Ok(LiteralType::String(format!("{}", value))),
                    LiteralType::String(value) => Ok(LiteralType::String(format!("{}", value))),
                    _ => Err(error),
                }
            }),
        }
    }
}
