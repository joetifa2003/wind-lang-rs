use std::process;

use crate::token::Token;

pub trait WindError {
    fn report(&self);
}

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

pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    pub fn new(token: Token, message: String) -> ParseError {
        ParseError { token, message }
    }
}

impl WindError for ParseError {
    fn report(&self) {
        eprintln!("[line {}]: {}", self.token.line, self.message);
        process::exit(1);
    }
}

pub struct ScannerError {
    line: i32,
    message: String,
}

impl ScannerError {
    pub fn new(line: i32, message: String) -> ScannerError {
        ScannerError { line, message }
    }
}

impl WindError for ScannerError {
    fn report(&self) {
        eprintln!("[line {}]: {}", self.line, self.message);
        process::exit(65);
    }
}
