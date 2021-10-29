use std::rc::Rc;

use crate::{ast::Stmt, error::RuntimeError, interpreter::Interpreter, token::Token};

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
                interpreter.environment_manager.add_env();

                for i in 0..params.len() {
                    interpreter
                        .environment_manager
                        .define(params[i].lexeme.to_owned(), args[i].to_owned());
                }

                match interpreter.execute_block(&body)? {
                    Some(value) => {
                        interpreter.environment_manager.remove_env();

                        return Ok(value);
                    }
                    None => (),
                }

                interpreter.environment_manager.remove_env();

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
