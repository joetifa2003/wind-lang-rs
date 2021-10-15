use std::{any::Any, rc::Rc};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    token::Token,
    types::LiteralType,
};

pub trait ExprVisitor<T> {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> T;
    fn visit_group_expr(&mut self, expr: &GroupExpr) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> T;
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> T;
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> T;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_expression_smt(&mut self, stmt: &ExpressionStmt) -> T;
    fn visit_print_smt(&mut self, stmt: &PrintStmt) -> T;
    fn visit_function_decl_stmt(&mut self, stmt: &FunctionDeclStmt) -> T;
    fn visit_var_decl_smt(&mut self, stmt: &VarDeclStmt) -> T;
    fn visit_while_smt(&mut self, stmt: &WhileSmt) -> T;
    fn visit_for_range_smt(&mut self, stmt: &ForRangeStmt) -> T;
    fn visit_block_smt(&mut self, stmt: &BlockStmt) -> T;
    fn visit_if_smt(&mut self, stmt: &IfStmt) -> T;
}

pub trait Expr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError>;

    fn as_any(&self) -> &dyn Any;
}

pub trait Stmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError>;
}

pub struct BinaryExpr {
    pub left: Rc<dyn Expr>,
    pub operator: Token,
    pub right: Rc<dyn Expr>,
}

impl BinaryExpr {
    pub fn new(left: Rc<dyn Expr>, operator: Token, right: Rc<dyn Expr>) -> Rc<BinaryExpr> {
        Rc::new(BinaryExpr {
            left,
            operator,
            right,
        })
    }
}

impl Expr for BinaryExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_binary_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct GroupExpr {
    pub expression: Rc<dyn Expr>,
}

impl GroupExpr {
    pub fn new(expression: Rc<dyn Expr>) -> Rc<GroupExpr> {
        Rc::new(GroupExpr { expression })
    }
}

impl Expr for GroupExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_group_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct LiteralExpr {
    pub value: LiteralType,
}

impl LiteralExpr {
    pub fn new(value: LiteralType) -> Rc<LiteralExpr> {
        Rc::new(LiteralExpr { value })
    }
}

impl Expr for LiteralExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_literal_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct UnaryExpr {
    pub operator: Token,
    pub right: Rc<dyn Expr>,
}

impl UnaryExpr {
    pub fn new(operator: Token, right: Rc<dyn Expr>) -> Rc<UnaryExpr> {
        Rc::new(UnaryExpr { operator, right })
    }
}

impl Expr for UnaryExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_unary_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct CallExpr {
    pub callee: Rc<dyn Expr>,
    pub paren: Token,
    pub arguments: Vec<Rc<dyn Expr>>,
}

impl CallExpr {
    pub fn new(callee: Rc<dyn Expr>, paren: Token, arguments: Vec<Rc<dyn Expr>>) -> Rc<dyn Expr> {
        Rc::new(CallExpr {
            callee,
            paren,
            arguments,
        })
    }
}

impl Expr for CallExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_call_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct VariableExpr {
    pub name: Token,
}

impl VariableExpr {
    pub fn new(name: Token) -> Rc<VariableExpr> {
        Rc::new(VariableExpr { name })
    }
}

impl Expr for VariableExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_variable_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct AssignExpr {
    pub name: Token,
    pub value: Rc<dyn Expr>,
}

impl AssignExpr {
    pub fn new(name: Token, value: Rc<dyn Expr>) -> Rc<AssignExpr> {
        Rc::new(AssignExpr { name, value })
    }
}

impl Expr for AssignExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_assign_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct LogicalExpr {
    pub left: Rc<dyn Expr>,
    pub operator: Token,
    pub right: Rc<dyn Expr>,
}

impl LogicalExpr {
    pub fn new(left: Rc<dyn Expr>, operator: Token, right: Rc<dyn Expr>) -> Rc<LogicalExpr> {
        Rc::new(LogicalExpr {
            left,
            operator,
            right,
        })
    }
}

impl Expr for LogicalExpr {
    fn accept_interpreter(
        &self,
        interpreter: &mut Interpreter,
    ) -> Result<LiteralType, RuntimeError> {
        interpreter.visit_logical_expr(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ExpressionStmt {
    pub expression: Rc<dyn Expr>,
}

impl ExpressionStmt {
    pub fn new(expression: Rc<dyn Expr>) -> Rc<ExpressionStmt> {
        Rc::new(ExpressionStmt { expression })
    }
}

impl Stmt for ExpressionStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_expression_smt(self)
    }
}

pub struct PrintStmt {
    pub expression: Rc<dyn Expr>,
}

impl PrintStmt {
    pub fn new(expression: Rc<dyn Expr>) -> Rc<PrintStmt> {
        Rc::new(PrintStmt { expression })
    }
}

impl Stmt for PrintStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_print_smt(self)
    }
}

pub struct VarDeclStmt {
    pub name: Token,
    pub initializer: Option<Rc<dyn Expr>>,
}

impl VarDeclStmt {
    pub fn new(name: Token, initializer: Option<Rc<dyn Expr>>) -> Rc<VarDeclStmt> {
        Rc::new(VarDeclStmt { name, initializer })
    }
}

impl Stmt for VarDeclStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_var_decl_smt(self)
    }
}

pub struct WhileSmt {
    pub condition: Option<Rc<dyn Expr>>,
    pub body: Rc<dyn Stmt>,
}

impl WhileSmt {
    pub fn new(condition: Option<Rc<dyn Expr>>, body: Rc<dyn Stmt>) -> Rc<WhileSmt> {
        Rc::new(WhileSmt { condition, body })
    }
}

impl Stmt for WhileSmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_while_smt(self)
    }
}

pub struct ForRangeStmt {
    pub name: Token,
    pub range_start: Rc<dyn Expr>,
    pub range_end: Rc<dyn Expr>,
    pub body: Rc<dyn Stmt>,
}

impl ForRangeStmt {
    pub fn new(
        name: Token,
        range_start: Rc<dyn Expr>,
        range_end: Rc<dyn Expr>,
        body: Rc<dyn Stmt>,
    ) -> Rc<ForRangeStmt> {
        Rc::new(ForRangeStmt {
            name,
            range_start,
            range_end,
            body,
        })
    }
}

impl Stmt for ForRangeStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_for_range_smt(&self)
    }
}

pub struct BlockStmt {
    pub statements: Vec<Rc<dyn Stmt>>,
}

impl BlockStmt {
    pub fn new(statements: Vec<Rc<dyn Stmt>>) -> Rc<BlockStmt> {
        Rc::new(BlockStmt { statements })
    }
}

impl Stmt for BlockStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_block_smt(self)
    }
}

pub struct IfStmt {
    pub condition: Rc<dyn Expr>,
    pub then_branch: Rc<dyn Stmt>,
    pub else_branch: Option<Rc<dyn Stmt>>,
}

impl IfStmt {
    pub fn new(
        condition: Rc<dyn Expr>,
        then_branch: Rc<dyn Stmt>,
        else_branch: Option<Rc<dyn Stmt>>,
    ) -> Rc<IfStmt> {
        Rc::new(IfStmt {
            condition,
            then_branch,
            else_branch,
        })
    }
}

impl Stmt for IfStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_if_smt(self)
    }
}

pub struct FunctionDeclStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Rc<dyn Stmt>>,
}

impl FunctionDeclStmt {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Rc<dyn Stmt>>) -> Rc<FunctionDeclStmt> {
        Rc::new(FunctionDeclStmt { name, params, body })
    }
}

impl Stmt for FunctionDeclStmt {
    fn accept_interpreter(&self, interpreter: &mut Interpreter) -> Result<(), RuntimeError> {
        interpreter.visit_function_decl_stmt(self)
    }
}
