use std::{any::Any, rc::Rc};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    DotDot,
    Minus,
    MinusEqual,
    Plus,
    PlusEqual,
    Percent,
    PercentEqual,
    Semicolon,
    Slash,
    SlashEqual,
    Star,
    StarEqual,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    In,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub t_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Rc<dyn Any>>,
    pub line: i32,
}

impl Token {
    pub fn new(
        t_type: TokenType,
        lexeme: String,
        literal: Option<Rc<dyn Any>>,
        line: i32,
    ) -> Token {
        Token {
            t_type,
            lexeme,
            literal,
            line,
        }
    }
}
