mod environment;

use std::{process, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, Expr, ExprVisitor, ExpressionStmt, ForRangeStmt,
        GroupExpr, IfStmt, LiteralExpr, LogicalExpr, PrintStmt, Stmt, StmtVisitor, UnaryExpr,
        VarDeclStmt, VariableExpr, WhileSmt,
    },
    error::WindError,
    token::{Token, TokenType},
    types::LiteralType,
};

use self::environment::EnvironmentManger;

pub struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> RuntimeError {
        RuntimeError { token, message }
    }
}

impl WindError for RuntimeError {
    fn report(&self) {
        eprintln!(
            "[line {}]: near '{}' {}",
            self.token.line, self.token.lexeme, self.message
        );
        process::exit(1);
    }
}

pub struct Interpreter {
    environment_manager: EnvironmentManger,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment_manager: EnvironmentManger::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Rc<dyn Stmt>>) {
        for statement in statements {
            match self.execute(statement) {
                Err(e) => {
                    e.report();
                }
                _ => {}
            }
        }
    }

    fn execute(&mut self, stmt: Rc<dyn Stmt>) -> Result<(), RuntimeError> {
        stmt.accept_interpreter(self)
    }

    fn execute_block(&mut self, statements: Vec<Rc<dyn Stmt>>) -> Result<(), RuntimeError> {
        self.environment_manager.add_env();

        for statement in statements {
            self.execute(statement)?;
        }

        self.environment_manager.remove_env();

        Ok(())
    }

    fn evaluate(&mut self, expr: Rc<dyn Expr>) -> Result<LiteralType, RuntimeError> {
        expr.accept_interpreter(self)
    }

    fn is_truthy(&self, object: LiteralType) -> bool {
        match object {
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

impl ExprVisitor<Result<LiteralType, RuntimeError>> for Interpreter {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Result<LiteralType, RuntimeError> {
        Ok(expr.value.to_owned())
    }

    fn visit_group_expr(&mut self, expr: &GroupExpr) -> Result<LiteralType, RuntimeError> {
        let value = self.evaluate(expr.expression.to_owned())?;

        Ok(value)
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Result<LiteralType, RuntimeError> {
        let right = self.evaluate(expr.right.to_owned())?;

        match expr.operator.t_type {
            TokenType::Bang => Ok(LiteralType::Bool(self.is_truthy(right))),
            TokenType::Minus => {
                self.check_number_operand(&expr.operator, &right)?;
                match right {
                    LiteralType::Number(value) => Ok(LiteralType::Number(-value)),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Result<LiteralType, RuntimeError> {
        let left = self.evaluate(expr.left.to_owned())?;
        let right = self.evaluate(expr.right.to_owned())?;

        match expr.operator.t_type {
            TokenType::Minus | TokenType::MinusEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Number(left_value - right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::Slash | TokenType::SlashEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Number(left_value / right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::Star | TokenType::StarEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Number(left_value * right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::Percent | TokenType::PercentEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Number(left_value % right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::Greater => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Bool(left_value > right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::GreaterEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Bool(left_value >= right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::Less => {
                self.check_number_operands(&expr.operator, &left, &right)?;
                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Bool(left_value < right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::LessEqual => {
                self.check_number_operands(&expr.operator, &left, &right)?;

                match (left, right) {
                    (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                        Ok(LiteralType::Bool(left_value <= right_value))
                    }
                    _ => unreachable!(),
                }
            }
            TokenType::EqualEqual => Ok(LiteralType::Bool(self.is_equal(
                &expr.operator,
                left,
                right,
            )?)),
            TokenType::BangEqual => Ok(LiteralType::Bool(!self.is_equal(
                &expr.operator,
                left,
                right,
            )?)),
            TokenType::Plus | TokenType::PlusEqual => match (left, right) {
                (LiteralType::Number(left_value), LiteralType::Number(right_value)) => {
                    Ok(LiteralType::Number(left_value + right_value))
                }
                (LiteralType::String(left_value), LiteralType::String(right_value)) => {
                    let res = [left_value.to_owned(), right_value.to_owned()].join("");
                    Ok(LiteralType::String(res))
                }
                _ => Err(RuntimeError::new(
                    expr.operator.to_owned(),
                    "cannot add".to_owned(),
                )),
            },

            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Result<LiteralType, RuntimeError> {
        Ok(self.environment_manager.get(&expr.name)?)
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Result<LiteralType, RuntimeError> {
        let value = self.evaluate(expr.value.to_owned())?;

        self.environment_manager
            .assign(expr.name.to_owned(), value.to_owned())?;

        Ok(value)
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Result<LiteralType, RuntimeError> {
        let left = self.evaluate(expr.left.to_owned())?;

        if expr.operator.t_type == TokenType::Or {
            if self.is_truthy(left.to_owned()) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(left.to_owned()) {
                return Ok(left);
            }
        }

        Ok(self.evaluate(expr.right.to_owned())?)
    }
}

impl StmtVisitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_expression_smt(&mut self, stmt: &ExpressionStmt) -> Result<(), RuntimeError> {
        self.evaluate(stmt.expression.to_owned())?;

        Ok(())
    }

    fn visit_print_smt(&mut self, stmt: &PrintStmt) -> Result<(), RuntimeError> {
        let value = self.evaluate(stmt.expression.to_owned())?;

        match value {
            LiteralType::Nil => println!("{}", "nil"),
            LiteralType::Number(number_value) => println!("{}", number_value),
            LiteralType::String(string_value) => println!("{}", string_value),
            LiteralType::Bool(bool_value) => println!("{}", bool_value),
        };

        Ok(())
    }

    fn visit_var_decl_smt(&mut self, stmt: &VarDeclStmt) -> Result<(), RuntimeError> {
        let mut value: LiteralType = LiteralType::Nil;

        if let Some(initializer) = &stmt.initializer {
            value = self.evaluate(initializer.to_owned())?;
        }

        self.environment_manager
            .define(stmt.name.lexeme.to_owned(), value);

        Ok(())
    }

    fn visit_while_smt(&mut self, stmt: &WhileSmt) -> Result<(), RuntimeError> {
        if let Some(condition) = &stmt.condition {
            let mut condition_value = self.evaluate(condition.to_owned())?;

            while self.is_truthy(condition_value) {
                self.execute(stmt.body.to_owned())?;

                condition_value = self.evaluate(condition.to_owned())?;
            }
        }

        Ok(())
    }

    fn visit_for_range_smt(&mut self, stmt: &ForRangeStmt) -> Result<(), RuntimeError> {
        let start = self.evaluate(stmt.range_start.to_owned())?;
        let end = self.evaluate(stmt.range_end.to_owned())?;

        match (start, end) {
            (LiteralType::Number(start_value), LiteralType::Number(end_value)) => {
                self.environment_manager.add_env();

                self.environment_manager.define(
                    stmt.name.lexeme.to_owned(),
                    LiteralType::Number(start_value),
                );

                for i in start_value as i32..(end_value + 1.0) as i32 {
                    self.environment_manager
                        .assign(stmt.name.to_owned(), LiteralType::Number(i as f32))?;

                    self.execute(stmt.body.to_owned())?;
                }

                self.environment_manager.remove_env();

                Ok(())
            }
            _ => Err(RuntimeError::new(
                stmt.name.to_owned(),
                "range must be a number".to_owned(),
            )),
        }
    }

    fn visit_block_smt(&mut self, stmt: &BlockStmt) -> Result<(), RuntimeError> {
        self.execute_block(stmt.statements.to_owned())
    }

    fn visit_if_smt(&mut self, stmt: &IfStmt) -> Result<(), RuntimeError> {
        let if_condition = self.evaluate(stmt.condition.to_owned())?;

        if self.is_truthy(if_condition) {
            self.execute(stmt.then_branch.to_owned())?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch.to_owned())?;
        }

        Ok(())
    }
}
