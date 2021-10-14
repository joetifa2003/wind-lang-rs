mod environment;

use std::{
    any::{Any, TypeId},
    process,
    rc::Rc,
};

use crate::{
    ast::{Expr, ExprVisitor, NilType, Stmt, StmtVisitor},
    error::WindError,
    token::{Token, TokenType},
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

    fn evaluate(&mut self, expr: Rc<dyn Expr>) -> Result<Rc<dyn Any>, RuntimeError> {
        expr.accept_interpreter(self)
    }

    fn is_truthy(&self, object: Rc<dyn Any>) -> bool {
        if let Some(value) = object.downcast_ref::<bool>() {
            return *value;
        }

        return false;
    }

    fn check_number_operand(
        &self,
        operator: &Token,
        operand: Rc<dyn Any>,
    ) -> Result<(), RuntimeError> {
        if operand.as_ref().type_id() != TypeId::of::<f32>() {
            return Err(RuntimeError::new(
                operator.to_owned(),
                "operand must be a number".to_string(),
            ));
        }

        Ok(())
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: Rc<dyn Any>,
        right: Rc<dyn Any>,
    ) -> Result<(), RuntimeError> {
        let float_id = TypeId::of::<f32>();

        if left.as_ref().type_id() != float_id || right.as_ref().type_id() != float_id {
            return Err(RuntimeError::new(
                operator.to_owned(),
                "operands must be a number".to_string(),
            ));
        }

        Ok(())
    }

    fn is_equal(
        &self,
        operator: &Token,
        left: Rc<dyn Any>,
        right: Rc<dyn Any>,
    ) -> Result<bool, RuntimeError> {
        if let (Some(left_value), Some(right_value)) =
            (left.downcast_ref::<f32>(), right.downcast_ref::<f32>())
        {
            return Ok(*left_value == *right_value);
        }

        if let (Some(left_value), Some(right_value)) = (
            left.downcast_ref::<String>(),
            right.downcast_ref::<String>(),
        ) {
            return Ok(*left_value == *right_value);
        }

        Err(RuntimeError::new(
            operator.to_owned(),
            "cannot compare".to_owned(),
        ))
    }
}

impl ExprVisitor<Result<Rc<dyn Any>, RuntimeError>> for Interpreter {
    fn visit_literal_expr(
        &mut self,
        expr: &crate::ast::LiteralExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        Ok(expr.value.to_owned())
    }

    fn visit_group_expr(
        &mut self,
        expr: &crate::ast::GroupExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        let value = self.evaluate(expr.expression.to_owned())?;

        Ok(value)
    }

    fn visit_unary_expr(
        &mut self,
        expr: &crate::ast::UnaryExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        let right = self.evaluate(expr.right.to_owned())?;

        match expr.operator.t_type {
            TokenType::Bang => Ok(Rc::new(self.is_truthy(right))),
            TokenType::Minus => {
                self.check_number_operand(&expr.operator, right.to_owned())?;
                let value = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(value))
            }
            _ => unreachable!(),
        }
    }

    fn visit_binary_expr(
        &mut self,
        expr: &crate::ast::BinaryExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        let left = self.evaluate(expr.left.to_owned())?;
        let right = self.evaluate(expr.right.to_owned())?;

        match expr.operator.t_type {
            TokenType::Minus | TokenType::MinusEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number - right_number))
            }
            TokenType::Slash | TokenType::SlashEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number / right_number))
            }
            TokenType::Star | TokenType::StarEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number * right_number))
            }
            TokenType::Percent | TokenType::PercentEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number % right_number))
            }
            TokenType::Greater => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number > right_number))
            }
            TokenType::GreaterEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number >= right_number))
            }
            TokenType::Less => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number < right_number))
            }
            TokenType::LessEqual => {
                self.check_number_operands(&expr.operator, left.to_owned(), right.to_owned())?;

                let left_number = *left.downcast::<f32>().unwrap();
                let right_number = *right.downcast::<f32>().unwrap();

                Ok(Rc::new(left_number <= right_number))
            }
            TokenType::EqualEqual => Ok(Rc::new(self.is_equal(&expr.operator, left, right)?)),
            TokenType::BangEqual => Ok(Rc::new(!self.is_equal(&expr.operator, left, right)?)),
            TokenType::Plus | TokenType::PlusEqual => {
                if let (Some(left_value), Some(right_value)) =
                    (left.downcast_ref::<f32>(), right.downcast_ref::<f32>())
                {
                    return Ok(Rc::new(*left_value + *right_value));
                }

                if let (Some(left_value), Some(right_value)) = (
                    left.downcast_ref::<String>(),
                    right.downcast_ref::<String>(),
                ) {
                    let res = [left_value.to_owned(), right_value.to_owned()].join("");
                    return Ok(Rc::new(res));
                }

                Err(RuntimeError::new(
                    expr.operator.to_owned(),
                    "cannot add".to_owned(),
                ))
            }

            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(
        &mut self,
        expr: &crate::ast::VariableExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        Ok(self.environment_manager.get(&expr.name)?)
    }

    fn visit_assign_expr(
        &mut self,
        expr: &crate::ast::AssignExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
        let value = self.evaluate(expr.value.to_owned())?;

        self.environment_manager
            .assign(expr.name.to_owned(), value.to_owned())?;

        Ok(value)
    }

    fn visit_logical_expr(
        &mut self,
        expr: &crate::ast::LogicalExpr,
    ) -> Result<Rc<dyn Any>, RuntimeError> {
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
    fn visit_expression_smt(
        &mut self,
        stmt: &crate::ast::ExpressionStmt,
    ) -> Result<(), RuntimeError> {
        self.evaluate(stmt.expression.to_owned())?;

        Ok(())
    }

    fn visit_print_smt(&mut self, stmt: &crate::ast::PrintStmt) -> Result<(), RuntimeError> {
        let value = self.evaluate(stmt.expression.to_owned())?;

        if let Some(value) = value.downcast_ref::<f32>() {
            println!("{}", value);

            return Ok(());
        }

        if let Some(value) = value.downcast_ref::<String>() {
            println!("{}", value);

            return Ok(());
        }

        if let Some(value) = value.downcast_ref::<bool>() {
            println!("{}", value);

            return Ok(());
        }

        if let Some(_) = value.downcast_ref::<NilType>() {
            println!("nil");

            return Ok(());
        }

        Ok(())
    }

    fn visit_var_decl_smt(&mut self, stmt: &crate::ast::VarDeclStmt) -> Result<(), RuntimeError> {
        let mut value: Rc<dyn Any> = Rc::new(NilType::new());

        if let Some(initializer) = &stmt.initializer {
            value = self.evaluate(initializer.to_owned())?;
        }

        self.environment_manager
            .define(stmt.name.lexeme.to_owned(), value);

        Ok(())
    }

    fn visit_while_smt(&mut self, stmt: &crate::ast::WhileSmt) -> Result<(), RuntimeError> {
        if let Some(condition) = &stmt.condition {
            let mut condition_value = self.evaluate(condition.to_owned())?;

            while self.is_truthy(condition_value) {
                self.execute(stmt.body.to_owned())?;

                condition_value = self.evaluate(condition.to_owned())?;
            }
        }

        Ok(())
    }

    fn visit_block_smt(&mut self, stmt: &crate::ast::BlockStmt) -> Result<(), RuntimeError> {
        self.execute_block(stmt.statements.to_owned())
    }

    fn visit_if_smt(&mut self, stmt: &crate::ast::IfStmt) -> Result<(), RuntimeError> {
        let if_condition = self.evaluate(stmt.condition.to_owned())?;

        if self.is_truthy(if_condition) {
            self.execute(stmt.then_branch.to_owned())?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch.to_owned())?;
        }

        Ok(())
    }

    fn visit_for_range_smt(&mut self, stmt: &crate::ast::ForRangeStmt) -> Result<(), RuntimeError> {
        let start = self.evaluate(stmt.range_start.to_owned())?;
        let end = self.evaluate(stmt.range_end.to_owned())?;

        if let (Some(start), Some(end)) = (start.downcast_ref::<f32>(), end.downcast_ref::<f32>()) {
            self.environment_manager.add_env();

            self.environment_manager
                .define(stmt.name.lexeme.to_owned(), Rc::new(*start));

            for i in (*start) as i32..(*end + 1.0) as i32 {
                self.environment_manager
                    .assign(stmt.name.to_owned(), Rc::new(i as f32))?;

                self.execute(stmt.body.to_owned())?;
            }

            self.environment_manager.remove_env();

            Ok(())
        } else {
            Err(RuntimeError::new(
                stmt.name.to_owned(),
                "range must be a number".to_owned(),
            ))
        }
    }
}
