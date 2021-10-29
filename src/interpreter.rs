mod environment;

use crate::error::{RuntimeError, WindError};
use crate::{
    ast::{Expr, Stmt},
    token::{Token, TokenType},
    types::LiteralType,
};

use self::environment::EnvironmentManger;

pub struct Interpreter {
    pub environment_manager: EnvironmentManger,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment_manager: EnvironmentManger::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in &statements {
            match self.execute(statement) {
                Err(e) => {
                    e.report();
                }
                _ => {}
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Option<LiteralType>, RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;

                Ok(None)
            }
            Stmt::Block(statements) => self.execute_block(statements),
            Stmt::VarDecl { name, initializer } => {
                let mut value: LiteralType = LiteralType::Nil;

                if let Some(initializer) = initializer {
                    value = self.evaluate(initializer)?;
                }

                self.environment_manager
                    .define(name.lexeme.to_owned(), value);

                Ok(None)
            }
            Stmt::While { condition, body } => {
                if let Some(condition) = condition {
                    let mut condition_value = self.evaluate(condition)?;

                    while self.is_truthy(&condition_value) {
                        match self.execute(&(*body))? {
                            Some(value) => return Ok(Some(value)),
                            None => (),
                        }

                        condition_value = self.evaluate(condition)?;
                    }
                }

                Ok(None)
            }
            Stmt::ForRange {
                name,
                range_start,
                range_end,
                body,
            } => {
                let start = self.evaluate(range_start)?;
                let end = self.evaluate(range_end)?;

                match (start, end) {
                    (LiteralType::Number(start_value), LiteralType::Number(end_value)) => {
                        self.environment_manager.add_env();

                        self.environment_manager
                            .define(name.lexeme.to_owned(), LiteralType::Number(start_value));

                        for i in start_value as i32..(end_value + 1.0) as i32 {
                            self.environment_manager
                                .assign(name.to_owned(), LiteralType::Number(i as f32))?;

                            match self.execute(&(*body))? {
                                Some(value) => {
                                    self.environment_manager.remove_env();
                                    return Ok(Some(value));
                                }
                                _ => {}
                            }
                        }

                        self.environment_manager.remove_env();

                        Ok(None)
                    }
                    _ => Err(RuntimeError::new(
                        name.to_owned(),
                        "range must be a number".to_owned(),
                    )),
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let if_condition = self.evaluate(condition)?;

                if self.is_truthy(&if_condition) {
                    return Ok(self.execute(&(*then_branch))?);
                } else if let Some(else_branch) = else_branch {
                    return Ok(self.execute(&(*else_branch))?);
                }

                Ok(None)
            }
            Stmt::FunctionDecl {
                name,
                params: _,
                body: _,
            } => {
                let function = LiteralType::Function {
                    deceleration: stmt.to_owned(),
                };

                self.environment_manager
                    .define(name.lexeme.to_owned(), function);

                Ok(None)
            }
            Stmt::Return { keyword: _, value } => {
                let value = self.evaluate(value)?;

                Ok(Some(value))
            }
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
    ) -> Result<Option<LiteralType>, RuntimeError> {
        self.environment_manager.add_env();

        for statement in statements {
            match self.execute(statement)? {
                Some(value) => {
                    self.environment_manager.remove_env();

                    return Ok(Some(value));
                }
                _ => (),
            };
        }

        self.environment_manager.remove_env();

        Ok(None)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LiteralType, RuntimeError> {
        match expr {
            Expr::Group(expr) => self.evaluate(expr),
            Expr::Literal(value) => Ok(value.to_owned()),
            Expr::Variable(name) => Ok(self.environment_manager.get(&name)?),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.t_type {
                    TokenType::Minus | TokenType::MinusEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Number(left_value - right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::Slash | TokenType::SlashEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Number(left_value / right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::Star | TokenType::StarEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Number(left_value * right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::Percent | TokenType::PercentEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Number(left_value % right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::Greater => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Bool(left_value > right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::GreaterEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Bool(left_value >= right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::Less => {
                        self.check_number_operands(&operator, &left, &right)?;
                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Bool(left_value < right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::LessEqual => {
                        self.check_number_operands(&operator, &left, &right)?;

                        match (left, right) {
                            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                                Ok(LiteralType::Bool(left_value <= right_value))
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokenType::EqualEqual => {
                        Ok(LiteralType::Bool(self.is_equal(&operator, left, right)?))
                    }
                    TokenType::BangEqual => {
                        Ok(LiteralType::Bool(!self.is_equal(&operator, left, right)?))
                    }
                    TokenType::Plus | TokenType::PlusEqual => match (left, right) {
                        (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                            Ok(LiteralType::Number(left_value + right_value))
                        }
                        (LiteralType::String(left_value), LiteralType::String(right_value)) => {
                            let res = [left_value.to_owned(), right_value.to_owned()].join("");
                            Ok(LiteralType::String(res))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "cannot add".to_owned(),
                        )),
                    },

                    _ => unreachable!(),
                }
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.t_type {
                    TokenType::Bang => Ok(LiteralType::Bool(self.is_truthy(&right))),
                    TokenType::Minus => {
                        self.check_number_operand(&operator, &right)?;
                        match right {
                            LiteralType::Number(value) => Ok(LiteralType::Number(-value)),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Call {
                callee,
                paren,
                args: arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let mut args: Vec<LiteralType> = Vec::new();

                for argument in arguments {
                    args.push(self.evaluate(argument)?);
                }

                let arity = callee.arity(&paren)?;

                if args.len() == arity {
                    callee.call(self, &paren, args)
                } else {
                    Err(RuntimeError::new(
                        paren.to_owned(),
                        format!("expected {} arguments but got {}", arity, args.len()),
                    ))
                }
            }
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                self.environment_manager
                    .assign(name.to_owned(), value.to_owned())?;

                Ok(value.to_owned())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator.t_type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }

                Ok(self.evaluate(right)?)
            }
        }
    }

    fn is_truthy(&self, object: &LiteralType) -> bool {
        match *object {
            LiteralType::Bool(value) => value,
            _ => false,
        }
    }

    fn check_number_operand(
        &self,
        operator: &Token,
        operand: &LiteralType,
    ) -> Result<(), RuntimeError> {
        match operand {
            LiteralType::Number(_) => Ok(()),
            _ => Err(RuntimeError::new(
                operator.to_owned(),
                "operand must be a number".to_string(),
            )),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &LiteralType,
        right: &LiteralType,
    ) -> Result<(), RuntimeError> {
        match (left, right) {
            (LiteralType::Number(_), LiteralType::Number(_)) => Ok(()),
            _ => Err(RuntimeError::new(
                operator.to_owned(),
                "operands must be a number".to_string(),
            )),
        }
    }

    fn is_equal(
        &self,
        operator: &Token,
        left: LiteralType,
        right: LiteralType,
    ) -> Result<bool, RuntimeError> {
        match (left, right) {
            (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                Ok(left_value == right_value)
            }
            (LiteralType::String(left_value), LiteralType::String(right_value)) => {
                Ok(left_value == right_value)
            }
            _ => Err(RuntimeError::new(
                operator.to_owned(),
                "cannot compare".to_owned(),
            )),
        }
    }
}
