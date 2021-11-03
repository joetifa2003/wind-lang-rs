use std::{fmt::Display, rc::Rc};

use crate::{
    ast::Stmt,
    error::RuntimeError,
    interpreter::{environment::Environment, Interpreter},
    token::Token,
};

#[derive(Clone)]
pub enum LiteralType {
    Nil,
    Number(f32),
    String(String),
    Bool(bool),
    Function {
        deceleration: Stmt,
    },
    NativeFunction {
        name: String,
        arity: usize,
        func: Rc<dyn Fn(Vec<LiteralType>, Token) -> Result<LiteralType, RuntimeError>>,
    },
}

impl Display for LiteralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
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
        )
    }
}

impl LiteralType {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        args: Vec<LiteralType>,
    ) -> Result<LiteralType, RuntimeError> {
        match self {
            LiteralType::Function { deceleration } => {
                let (_, params, body) = deceleration.as_function_decl().unwrap();
                let environment = Environment::with_enclosing(interpreter.environment.clone());

                for i in 0..params.len() {
                    environment
                        .borrow_mut()
                        .define(params[i].lexeme.to_owned(), args[i].to_owned());
                }

                match interpreter.execute_block(&body, environment)? {
                    Some(value) => {
                        return Ok(value);
                    }
                    None => (),
                }

                Ok(LiteralType::Nil)
            }
            LiteralType::NativeFunction {
                name: _,
                arity: _,
                func,
            } => func(args, paren.to_owned()),
            _ => Err(RuntimeError::new(
                paren.to_owned(),
                "can only call functions and classes".to_owned(),
            )),
        }
    }

    pub fn arity(&self, paren: &Token) -> Result<usize, RuntimeError> {
        match self {
            LiteralType::Function { deceleration } => {
                let (_, params, _) = deceleration.as_function_decl().unwrap();

                Ok(params.len())
            }
            LiteralType::NativeFunction {
                name: _,
                arity,
                func: _,
            } => Ok(*arity),
            _ => Err(RuntimeError::new(
                paren.to_owned(),
                "can only call functions and classes".to_owned(),
            )),
        }
    }
}
