use crate::{
    ast::Stmt,
    interpreter::{Interpreter, RuntimeError},
    token::Token,
};

#[derive(Clone)]
pub enum LiteralType {
    Nil,
    Number(f32),
    String(String),
    Bool(bool),
    Function { deceleration: Stmt },
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

                interpreter.execute_block(&body)?;

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
            LiteralType::Function { deceleration } => {
                let (_, params, _) = deceleration.as_function_decl().unwrap();

                Ok(params.len())
            }
            _ => Err(RuntimeError::new(
                paren.to_owned(),
                "can only call functions and classes".to_owned(),
            )),
        }
    }
}
