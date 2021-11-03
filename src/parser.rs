use std::rc::Rc;

use crate::{
    ast::{Expr, Stmt},
    error::{ParseError, WindError},
    token::{Token, TokenType},
    types::LiteralType,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => {
                    statements.push(decl);
                }
                Err(err) => {
                    err.report();
                }
            };
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::Fun]) {
            return Ok(self.function_declaration("function")?);
        }

        if self.match_token(&[TokenType::Var]) {
            return Ok(self.var_declaration()?);
        }

        let statement = self.statement()?;
        Ok(statement)
    }

    fn function_declaration(&mut self, kind: &'static str) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenType::Identifier,
            format!("expect {} name", kind).as_str(),
        )?;

        self.consume(
            TokenType::LeftParen,
            format!("expect '(' after {} name", kind).as_str(),
        )?;

        let mut params: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, "expect parameter name.")?);
            while self.match_token(&[TokenType::Comma]) {
                params.push(self.consume(TokenType::Identifier, "expect parameter name.")?);
            }
        }

        self.consume(
            TokenType::RightParen,
            format!("expect ')' after {} parameters", kind).as_str(),
        )?;

        self.consume(
            TokenType::LeftBrace,
            format!("expect '{{' before {} body", kind).as_str(),
        )?;
        let body = self.block()?;

        Ok(Stmt::FunctionDecl { name, body, params })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "expect variable name")?;
        let mut initializer: Option<Rc<Expr>> = None;

        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(Rc::new(self.expression()?));
        }

        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        )?;

        Ok(Stmt::VarDecl { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::If]) {
            return Ok(self.if_statement()?);
        }

        if self.match_token(&[TokenType::Return]) {
            return Ok(self.return_statement()?);
        }

        if self.match_token(&[TokenType::While]) {
            return Ok(self.while_statement()?);
        }

        if self.match_token(&[TokenType::For]) {
            return Ok(self.for_statement()?);
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        Ok(self.expression_statement()?)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_optional(TokenType::LeftParen);
        let condition = self.expression()?;
        self.consume_optional(TokenType::RightParen);

        let then_branch = self.statement()?;
        let mut else_branch: Option<Rc<Stmt>> = None;

        if self.match_token(&[TokenType::Else]) {
            else_branch = Some(Rc::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition: Rc::new(condition),
            then_branch: Rc::new(then_branch),
            else_branch,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous();
        let mut value = Expr::Literal(LiteralType::Nil);
        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }

        self.consume(TokenType::Semicolon, "expect ';' after return value")?;

        Ok(Stmt::Return {
            keyword,
            value: Rc::new(value),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_optional(TokenType::LeftParen);

        let stmt: Stmt;
        if self.check(TokenType::Identifier) {
            stmt = self.range_for_loop()?;
        } else {
            stmt = self.manual_for_loop()?;
        }

        self.consume_optional(TokenType::RightParen);

        Ok(stmt)
    }

    fn range_for_loop(&mut self) -> Result<Stmt, ParseError> {
        let name = self.advance();
        self.consume(TokenType::In, "expected 'in' after name")?;
        let range_start = self.expression()?;
        self.consume(TokenType::DotDot, "expected '..' after expression")?;
        let range_end = self.expression()?;

        let body = self.statement()?;

        Ok(Stmt::ForRange {
            name,
            range_start: Rc::new(range_start),
            range_end: Rc::new(range_end),
            body: Rc::new(body),
        })
    }

    fn manual_for_loop(&mut self) -> Result<Stmt, ParseError> {
        // self.consume(TokenType::LeftParen, "expect '(' after 'for'")?;

        let initializer: Option<Stmt>;
        if self.match_token(&[TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_token(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Rc<Expr>> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(Rc::new(self.expression()?));
        }

        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?)
        }

        // self.consume(TokenType::RightParen, "expect ')' after 'for'")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(Rc::new(increment))]);
        }

        if let None = condition {
            condition = Some(Rc::new(Expr::Literal(LiteralType::Bool(true))));
        }

        body = Stmt::While {
            condition,
            body: Rc::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_optional(TokenType::LeftParen);
        let condition = self.expression()?;
        self.consume_optional(TokenType::RightParen);

        let body = self.statement()?;

        Ok(Stmt::While {
            condition: Some(Rc::new(condition)),
            body: Rc::new(body),
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let declaration = self.declaration()?;

            statements.push(declaration);
        }

        self.consume(TokenType::RightBrace, "expect '}' after block")?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, "expect ';' after expression")?;

        Ok(Stmt::Expression(Rc::new(expr)))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let name = match expr.as_variable() {
                Some(name) => name,
                None => {
                    return Err(ParseError::new(
                        self.peak().to_owned(),
                        "invalid assignment target".to_owned(),
                    ))
                }
            };

            let value = self.assignment()?;

            return Ok(Expr::Assign {
                name: name.to_owned(),
                value: Rc::new(value),
            });
        } else if self.match_token(&[
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
            TokenType::PercentEqual,
        ]) {
            let name = match expr.as_variable() {
                Some(name) => name,
                None => {
                    return Err(ParseError::new(
                        self.peak().to_owned(),
                        "invalid assignment target".to_owned(),
                    ))
                }
            };

            let operator = self.previous();
            let value = self.assignment()?;

            return Ok(Expr::Assign {
                name: name.to_owned(),
                value: Rc::new(Expr::Binary {
                    left: Rc::new(Expr::Variable(name.to_owned())),
                    operator: operator,
                    right: Rc::new(value),
                }),
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let expr = self.and()?;

        if self.match_token(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.or()?;

            return Ok(Expr::Logical {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.and()?;

            return Ok(Expr::Logical {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::Binary {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[
            TokenType::Slash,
            TokenType::Star,
            TokenType::Percent,
            TokenType::PercentEqual,
        ]) {
            let operator = self.previous();
            let right = self.unary()?;

            expr = Expr::Binary {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Rc::new(right),
            });
        }

        Ok(self.call()?)
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args: Vec<Expr> = Vec::new();

        if !self.check(TokenType::RightParen) {
            args.push(self.expression()?);
            while self.match_token(&[TokenType::Comma]) {
                args.push(self.expression()?);
            }
        }

        let paren = self.consume(TokenType::RightParen, "expect ')' after arguments")?;

        Ok(Expr::Call {
            callee: Rc::new(callee),
            paren,
            args,
        })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralType::Bool(false)));
        }

        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralType::Bool(true)));
        }

        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralType::Nil));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            let value = self.previous().literal;
            return Ok(Expr::Literal(*value));
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenType::RightParen, "expected ')' after expression")?;

            return Ok(Expr::Group(Rc::new(expr)));
        }

        Err(ParseError::new(
            self.peak().to_owned(),
            "expect expression".to_owned(),
        ))
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peak().t_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peak().t_type == TokenType::EOF
    }

    fn peak(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().to_owned()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(ParseError::new(self.peak().to_owned(), message.to_owned()))
    }

    fn consume_optional(&mut self, token_type: TokenType) {
        if self.check(token_type) {
            self.advance();
        }
    }
}
