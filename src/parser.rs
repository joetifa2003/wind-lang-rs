use std::{process, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, Expr, ExpressionStmt, ForRangeStmt,
        FunctionDeclStmt, GroupExpr, IfStmt, LiteralExpr, LogicalExpr, PrintStmt, Stmt, UnaryExpr,
        VarDeclStmt, VariableExpr, WhileSmt,
    },
    error::WindError,
    token::{Token, TokenType},
    types::LiteralType,
};

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

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Rc<dyn Stmt>> {
        let mut statements: Vec<Rc<dyn Stmt>> = Vec::new();

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

    fn declaration(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        if self.match_token(&[TokenType::Fun]) {
            return Ok(self.function_declaration("function")?);
        }

        if self.match_token(&[TokenType::Var]) {
            return Ok(self.var_declaration()?);
        }

        let statement = self.statement()?;
        Ok(statement)
    }

    fn function_declaration(&mut self, kind: &'static str) -> Result<Rc<dyn Stmt>, ParseError> {
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

        Ok(FunctionDeclStmt::new(name, params, body))
    }

    fn var_declaration(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        let name = self.consume(TokenType::Identifier, "expect variable name")?;
        let mut initializer: Option<Rc<dyn Expr>> = None;

        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        )?;

        Ok(VarDeclStmt::new(name.to_owned(), initializer))
    }

    fn statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        if self.match_token(&[TokenType::If]) {
            return Ok(self.if_statement()?);
        }

        if self.match_token(&[TokenType::Print]) {
            return Ok(self.print_statement()?);
        }

        if self.match_token(&[TokenType::While]) {
            return Ok(self.while_statement()?);
        }

        if self.match_token(&[TokenType::For]) {
            return Ok(self.for_statement()?);
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(BlockStmt::new(self.block()?));
        }

        Ok(self.expression_statement()?)
    }

    fn if_statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        // self.consume(TokenType::LeftParen, "expect '(' after 'if'")?;
        let condition = self.expression()?;
        // self.consume(TokenType::RightParen, "expect ')' after 'if'")?;

        let then_branch = self.statement()?;
        let mut else_branch: Option<Rc<dyn Stmt>> = None;

        if self.match_token(&[TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(IfStmt::new(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "expect ';' after value")?;

        Ok(PrintStmt::new(value))
    }

    fn for_statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        if self.check(TokenType::Identifier) {
            return self.range_for_loop();
        } else {
            return self.manual_for_loop();
        }
    }

    fn range_for_loop(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        let name = self.advance();
        self.consume(TokenType::In, "expected 'in' after name")?;
        let range_start = self.expression()?;
        self.consume(TokenType::DotDot, "expected '..' after expression")?;
        let range_end = self.expression()?;

        let body = self.statement()?;

        Ok(ForRangeStmt::new(name, range_start, range_end, body))
    }

    fn manual_for_loop(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        // self.consume(TokenType::LeftParen, "expect '(' after 'for'")?;

        let initializer: Option<Rc<dyn Stmt>>;
        if self.match_token(&[TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_token(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Rc<dyn Expr>> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let mut increment: Option<Rc<dyn Expr>> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?)
        }

        // self.consume(TokenType::RightParen, "expect ')' after 'for'")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = BlockStmt::new(vec![body, ExpressionStmt::new(increment)]);
        }

        if let None = condition {
            condition = Some(LiteralExpr::new(LiteralType::Bool(true)));
        }

        body = WhileSmt::new(condition, body);

        if let Some(initializer) = initializer {
            body = BlockStmt::new(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        // self.consume(TokenType::LeftParen, "expect '(' after 'while'")?;
        let condition = self.expression()?;
        // self.consume(TokenType::RightParen, "expect ')' after 'while'")?;

        let body = self.statement()?;

        Ok(WhileSmt::new(Some(condition), body))
    }

    fn block(&mut self) -> Result<Vec<Rc<dyn Stmt>>, ParseError> {
        let mut statements: Vec<Rc<dyn Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let declaration = self.declaration()?;

            statements.push(declaration);
        }

        self.consume(TokenType::RightBrace, "expect '}' after block")?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Rc<dyn Stmt>, ParseError> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, "expect ';' after expression")?;

        Ok(ExpressionStmt::new(expr))
    }

    fn expression(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;
            let variable = match expr.as_any().downcast_ref::<VariableExpr>() {
                Some(v) => v,
                None => {
                    return Err(ParseError::new(
                        self.peak().to_owned(),
                        "invalid assignment target".to_owned(),
                    ));
                }
            };

            return Ok(AssignExpr::new(variable.name.to_owned(), value));
        } else if self.match_token(&[
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::SlashEqual,
            TokenType::PercentEqual,
        ]) {
            let operator = self.previous();
            let value = self.assignment()?;
            let variable = match expr.as_any().downcast_ref::<VariableExpr>() {
                Some(v) => v,
                None => {
                    return Err(ParseError::new(
                        self.peak().to_owned(),
                        "invalid assignment target".to_owned(),
                    ));
                }
            };

            return Ok(AssignExpr::new(
                variable.name.to_owned(),
                BinaryExpr::new(Rc::new(variable.to_owned()), operator.to_owned(), value),
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let expr = self.and()?;

        if self.match_token(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.or()?;

            return Ok(LogicalExpr::new(expr, operator.to_owned(), right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.and()?;

            return Ok(LogicalExpr::new(expr, operator.to_owned(), right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;

            expr = BinaryExpr::new(expr, operator.to_owned(), right)
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = BinaryExpr::new(expr, operator.to_owned(), right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = BinaryExpr::new(expr, operator.to_owned(), right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[
            TokenType::Slash,
            TokenType::Star,
            TokenType::Percent,
            TokenType::PercentEqual,
        ]) {
            let operator = self.previous();
            let right = self.unary()?;

            expr = BinaryExpr::new(expr, operator.to_owned(), right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(UnaryExpr::new(operator.to_owned(), right));
        }

        Ok(self.call()?)
    }

    fn call(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
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

    fn finish_call(&mut self, callee: Rc<dyn Expr>) -> Result<Rc<dyn Expr>, ParseError> {
        let mut args: Vec<Rc<dyn Expr>> = Vec::new();

        if !self.check(TokenType::RightParen) {
            args.push(self.expression()?);
            while self.match_token(&[TokenType::Comma]) {
                args.push(self.expression()?);
            }
        }

        let paren = self.consume(TokenType::RightParen, "expect ')' after arguments")?;

        Ok(CallExpr::new(callee, paren, args))
    }

    fn primary(&mut self) -> Result<Rc<dyn Expr>, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(LiteralExpr::new(LiteralType::Bool(false)));
        }

        if self.match_token(&[TokenType::True]) {
            return Ok(LiteralExpr::new(LiteralType::Bool(true)));
        }

        if self.match_token(&[TokenType::Nil]) {
            return Ok(LiteralExpr::new(LiteralType::Nil));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            let value = self.previous().literal;
            return Ok(LiteralExpr::new(value));
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(VariableExpr::new(self.previous().to_owned()));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenType::RightParen, "expected ')' after expression")?;

            return Ok(GroupExpr::new(expr));
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
}
