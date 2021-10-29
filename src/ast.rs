use std::rc::Rc;

use crate::{token::Token, types::LiteralType};
use enum_as_inner::EnumAsInner;

#[derive(EnumAsInner, Clone)]
pub enum Expr {
    Group(Rc<Expr>),
    Literal(LiteralType),
    Variable(Token),
    Binary {
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
    Unary {
        operator: Token,
        right: Rc<Expr>,
    },
    Call {
        callee: Rc<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Assign {
        name: Token,
        value: Rc<Expr>,
    },
    Logical {
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
}

#[derive(EnumAsInner, Clone)]
pub enum Stmt {
    Expression(Rc<Expr>),
    Block(Vec<Stmt>),
    VarDecl {
        name: Token,
        initializer: Option<Rc<Expr>>,
    },
    While {
        condition: Option<Rc<Expr>>,
        body: Rc<Stmt>,
    },
    ForRange {
        name: Token,
        range_start: Rc<Expr>,
        range_end: Rc<Expr>,
        body: Rc<Stmt>,
    },
    If {
        condition: Rc<Expr>,
        then_branch: Rc<Stmt>,
        else_branch: Option<Rc<Stmt>>,
    },
    FunctionDecl {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Rc<Expr>,
    },
}
