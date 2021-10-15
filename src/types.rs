use std::rc::Rc;

use crate::{
    ast::FunctionDeclStmt,
    interpreter::{Interpreter, RuntimeError},
    token::Token,
};

#[derive(Clone)]
pub enum LiteralType {
    Nil,
    Number(f32),
    String(String),
    Bool(bool),
    Function(Rc<FunctionDeclStmt>),
}

impl LiteralType {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        args: Vec<LiteralType>,
    ) -> Result<LiteralType, RuntimeError> {
        match self {
            LiteralType::Function(deceleration) => {
                interpreter.environment_manager.add_env();

                for i in 0..deceleration.params.len() {
                    interpreter
                        .environment_manager
                        .define(deceleration.params[i].lexeme.to_owned(), args[i].to_owned());
                }

                interpreter.execute_block(deceleration.body.to_owned())?;

                interpreter.environment_manager.remove_env();

                Ok(LiteralType::Nil)
            }
            _ => Err(RuntimeError::new(
                paren.to_owned(),
                "can only call functions and classes".to_owned(),
            )),
        }
    }

    pub fn arity(&self, paren: &Token) -> Result<usize, RuntimeError> {
        match self {
            LiteralType::Function(decl) => Ok(decl.params.len()),
            _ => Err(RuntimeError::new(
                paren.to_owned(),
                "can only call functions and classes".to_owned(),
            )),
        }
    }
}
