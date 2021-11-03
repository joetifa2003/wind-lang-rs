use std::{fmt::Display, rc::Rc};

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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expr::Group(expr) => format!("{}", expr),
                Expr::Literal(literal) => match literal {
                    LiteralType::String(string) => format!("\"{}\"", string),
                    _ => format!("{}", literal),
                },
                Expr::Variable(name) => format!("{}", name.lexeme),
                Expr::Binary {
                    left,
                    operator,
                    right,
                } => format!("({} {} {})", left, operator.lexeme, right),
                Expr::Unary { operator, right } => format!("({}{})", operator.lexeme, right),
                Expr::Call {
                    callee,
                    paren: _,
                    args,
                } => {
                    let mut arg_str = String::new();

                    for (index, arg) in args.iter().enumerate() {
                        arg_str += format!("{}", arg).as_str();

                        if index < args.len() - 1 {
                            arg_str += ", ";
                        }
                    }

                    format!("{}({})", callee, arg_str)
                }
                Expr::Assign { name, value } => format!("{} = {}", name.lexeme, value),
                Expr::Logical {
                    left,
                    operator,
                    right,
                } => format!("{} {} {}", left, operator.lexeme, right),
            }
        )
    }
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

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} ",
            match self {
                Stmt::Expression(expr) => format!("{}", expr),
                Stmt::Block(statements) => {
                    let mut block_str = String::new();

                    for stmt in statements {
                        block_str += format!("{}", stmt).as_str();
                    }

                    format!("{{{}}}", block_str)
                }
                Stmt::VarDecl { name, initializer } => {
                    let initializer_str = match initializer {
                        Some(expr) => format!("{}", expr),
                        None => "nil".to_owned(),
                    };

                    format!("var {} = {}", name.lexeme, initializer_str)
                }
                Stmt::While { condition, body } => {
                    let condition_str = match condition {
                        Some(expr) => format!("{}", expr),
                        None => "None".to_owned(),
                    };

                    format!("while (condition {}) {}", condition_str, body)
                }
                Stmt::ForRange {
                    name,
                    range_start,
                    range_end,
                    body,
                } => format!(
                    "for {} range (start {}) (end {}) {}",
                    name.lexeme,
                    format!("{}", range_start),
                    format!("{}", range_end),
                    format!("{}", body)
                ),
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let else_str = match else_branch {
                        Some(stmt) => format!("{}", stmt),
                        None => "None".to_owned(),
                    };

                    format!(
                        "if (condition {}) (then {}) (else {})",
                        format!("{}", condition),
                        format!("{}", then_branch),
                        format!("{}", else_str)
                    )
                }
                Stmt::FunctionDecl { name, params, body } => {
                    let mut params_str = String::new();

                    for (index, param) in params.iter().enumerate() {
                        params_str += format!("{}", param.lexeme).as_str();

                        if index < params.len() - 1 {
                            params_str += ", ";
                        }
                    }

                    let mut body_str = String::new();

                    for stmt in body {
                        body_str += format!("{}", stmt).as_str();
                    }

                    format!(
                        "fun {} (params {}) {}",
                        name.lexeme,
                        params_str,
                        format!("{{{}}}", body_str)
                    )
                }
                Stmt::Return { keyword: _, value } => format!("return {}", format!("{}", value)),
            }
        )
    }
}
