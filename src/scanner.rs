use std::{any::Any, process, rc::Rc};

use crate::token::{Token, TokenType};

pub struct ScannerError {
    line: i32,
    message: String,
}

impl ScannerError {
    fn new(line: i32, message: String) -> ScannerError {
        ScannerError { line, message }
    }

    fn report(&self) {
        eprintln!("[line {}] Error: {}", self.line, self.message);
        process::exit(65);
    }
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: i32,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;

            match self.scan_token() {
                Err(e) => {
                    e.report();
                }
                _ => (),
            }
        }

        self.add_token(TokenType::EOF, None);

        self.tokens.to_owned()
    }

    fn scan_token(&mut self) -> Result<(), ScannerError> {
        let current_char = self.advance();

        match current_char {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenType::MinusEqual, None);
                } else {
                    self.add_token(TokenType::Minus, None);
                }
            }
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PlusEqual, None);
                } else {
                    self.add_token(TokenType::Plus, None);
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.add_token(TokenType::StarEqual, None);
                } else {
                    self.add_token(TokenType::Star, None);
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PercentEqual, None);
                } else {
                    self.add_token(TokenType::Percent, None);
                }
            }
            '/' => {
                if self.match_char('/') {
                    while !self.is_at_end() && self.peak() != '\n' {
                        self.advance();
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenType::SlashEqual, None);
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual, None);
                } else {
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual, None);
                } else {
                    self.add_token(TokenType::Equal, None);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual, None);
                } else {
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual, None);
                } else {
                    self.add_token(TokenType::Greater, None);
                }
            }
            '"' => self.scan_string()?,
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            _ => {
                if current_char.is_numeric() {
                    self.scan_number()?;
                } else if current_char.is_alphabetic() {
                    self.scan_identifier();
                } else {
                    return Err(ScannerError::new(
                        self.line,
                        format!("unexpected character '{}'", current_char),
                    ));
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> char {
        let current_char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        current_char
    }

    fn scan_string(&mut self) -> Result<(), ScannerError> {
        while !self.is_at_end() && self.peak() != '"' {
            if self.peak() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return Err(ScannerError::new(
                self.line,
                "unterminated string".to_owned(),
            ));
        }

        self.advance(); // "

        let literal = self.source[self.start + 1..self.current - 1]
            .to_owned()
            .replace("\\n", "\n");
        self.add_token(TokenType::String, Some(Rc::new(literal)));

        Ok(())
    }

    fn scan_number(&mut self) -> Result<(), ScannerError> {
        while self.peak().is_numeric() {
            self.advance();
        }

        if self.peak() == '.' && self.peak().is_numeric() {
            self.advance();

            while self.peak().is_numeric() {
                self.advance();
            }
        }

        let literal = self.source[self.start..self.current].to_owned();
        let float: f32 = match literal.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(ScannerError::new(
                    self.line,
                    "cannot parse number".to_owned(),
                ));
            }
        };

        self.add_token(TokenType::Number, Some(Rc::new(float)));

        Ok(())
    }

    fn scan_identifier(&mut self) {
        while self.peak().is_alphanumeric() || self.peak() == '_' {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_owned();
        let token_type = self.match_keyword(&text);

        self.add_token(token_type, None);
    }

    fn add_token(&mut self, t_type: TokenType, literal: Option<Rc<dyn Any>>) {
        let text = self.source[self.start..self.current].to_owned();

        self.tokens
            .push(Token::new(t_type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peak() != expected {
            return false;
        } else {
            self.advance();

            return true;
        }
    }

    fn peak(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn match_keyword(&mut self, name: &str) -> TokenType {
        match name {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "var" => TokenType::Var,
            _ => TokenType::Identifier,
        }
    }
}
