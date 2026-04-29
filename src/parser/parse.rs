//! Recursive descent parser for Aether

use super::ast::*;
use crate::lexer::{Token, TokenKind};
use std::fmt;
use std::rc::Rc;

/// Parse error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token },
    UnexpectedEof,
    InvalidAssignmentTarget,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found } => {
                write!(
                    f,
                    "Expected {}, found '{}' at line {}",
                    expected, found.lexeme, found.line
                )
            }
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidAssignmentTarget => write!(f, "Invalid assignment target"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Recursive descent parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parses the tokens into a Program
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(Program::new(statements))
    }

    // Parser helper methods
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek_at(&self, offset: usize) -> &Token {
        let idx = self.current + offset;
        if idx < self.tokens.len() {
            &self.tokens[idx]
        } else {
            &self.tokens[self.tokens.len() - 1] // Eof
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: self.peek().clone(),
            })
        }
    }

    // Parse declarations (let statements, function declarations, import statements)
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenKind::Let]) {
            return self.let_declaration();
        }
        if self.match_token(&[TokenKind::Struct]) {
            return self.struct_declaration();
        }
        if self.match_token(&[TokenKind::Enum]) {
            return self.enum_declaration();
        }
        if self.match_token(&[TokenKind::Import]) {
            return self.import_statement();
        }
        if self.match_token(&[TokenKind::From]) {
            return self.parse_from_import_statement();
        }
        // Check for async fn name(...) declaration
        if self.check(&TokenKind::Async)
            && matches!(self.peek_at(1).kind, TokenKind::Fn)
            && matches!(self.peek_at(2).kind, TokenKind::Identifier(_))
        {
            self.advance(); // consume 'async'
            self.advance(); // consume 'fn'
            return self.async_function_declaration();
        }
        // Check if this is a function declaration (fn identifier) or function expression (fn()
        if self.check(&TokenKind::Fn) {
            // Peek ahead to see what follows 'fn'
            if self.current + 1 < self.tokens.len() {
                let next_token = &self.tokens[self.current + 1];
                if matches!(next_token.kind, TokenKind::Identifier(_)) {
                    // It's a function declaration: fn name(...)
                    self.advance(); // consume 'fn'
                    return self.function_declaration();
                }
            }
            // Otherwise, it's a function expression, fall through to statement parsing
        }
        // Detect labeled loop: identifier ':' for/while
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if matches!(self.peek_at(1).kind, TokenKind::Colon)
                && matches!(self.peek_at(2).kind, TokenKind::For | TokenKind::While)
            {
                let label = name.clone();
                self.advance(); // consume identifier
                self.advance(); // consume ':'
                let inner = self.statement()?;
                return Ok(Stmt::Labeled(label, Box::new(inner)));
            }
        }
        self.statement()
    }

    // Parse let declaration: let name = expr
    fn let_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "variable name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        self.consume(TokenKind::Equal, "=")?;
        let initializer = self.expression()?;

        Ok(Stmt::Let(name, initializer))
    }

    // Parse function declaration: fn name(params) { body }
    fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        // Parse function name
        let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "function name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        let params = self.parse_params()?;
        self.consume(TokenKind::LeftBrace, "{")?;
        let body = self.block_statement()?;

        Ok(Stmt::Function(name, params, Rc::new(body)))
    }

    // Parse statements
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.match_token(&[TokenKind::For]) {
            return self.for_statement();
        }
        if self.match_token(&[TokenKind::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenKind::Break]) {
            // Optional label: break outer
            let label = if let TokenKind::Identifier(name) = &self.peek().kind {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                None
            };
            return Ok(Stmt::Break(label));
        }
        if self.match_token(&[TokenKind::Continue]) {
            // Optional label: continue outer
            let label = if let TokenKind::Identifier(name) = &self.peek().kind {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                None
            };
            return Ok(Stmt::Continue(label));
        }
        if self.match_token(&[TokenKind::LeftBrace]) {
            return self.block_statement();
        }
        if self.match_token(&[TokenKind::Try]) {
            return self.try_catch_statement();
        }
        if self.match_token(&[TokenKind::Throw]) {
            return self.throw_statement();
        }
        if self.match_token(&[TokenKind::Match]) {
            return self.parse_match_statement();
        }
        self.expression_statement()
    }

    // Parse block statement: { stmt* }
    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let line = self.peek().line;
            statements.push(Stmt::Line(line));
            statements.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "}")?;
        Ok(Stmt::Block(statements))
    }

    // Parse a parenthesised parameter list: (a, b, c) — returns names only
    fn parse_params(&mut self) -> Result<Vec<String>, ParseError> {
        self.consume(TokenKind::LeftParen, "(")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                if let TokenKind::Identifier(param) = &self.peek().kind {
                    params.push(param.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "parameter name".to_string(),
                        found: self.peek().clone(),
                    });
                }
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, ")")?;
        Ok(params)
    }

    // Parse if statement: if (condition) then_branch [else else_branch]
    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "(")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, ")")?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = if self.match_token(&[TokenKind::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    // Parse while loop: while (condition) body
    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "(")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, ")")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While(condition, body))
    }

    // Parse for loop: for variable in iterable body
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        let variable = if let TokenKind::Identifier(name) = &self.peek().kind {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "variable name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        self.consume(TokenKind::In, "in")?;
        let iterable = self.expression()?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::For(variable, iterable, body))
    }

    // Parse return statement: return [expr]
    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = if self.check(&TokenKind::RightBrace) || self.is_at_end() {
            None
        } else {
            Some(self.expression()?)
        };

        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        // Check if this is actually an assignment
        if self.match_token(&[TokenKind::Equal]) {
            // Simple assignment: target = value
            self.validate_assignment_target(&expr)?;
            let value = self.expression()?;
            return Ok(Stmt::Assign(expr, value));
        } else if self.match_token(&[
            TokenKind::PlusEqual,
            TokenKind::MinusEqual,
            TokenKind::StarEqual,
            TokenKind::SlashEqual,
        ]) {
            // Compound assignment: target += value, etc.
            self.validate_assignment_target(&expr)?;
            let op = match self.previous().kind {
                TokenKind::PlusEqual => BinaryOp::Add,
                TokenKind::MinusEqual => BinaryOp::Subtract,
                TokenKind::StarEqual => BinaryOp::Multiply,
                TokenKind::SlashEqual => BinaryOp::Divide,
                _ => unreachable!(),
            };
            let value = self.expression()?;
            return Ok(Stmt::CompoundAssign(expr, op, value));
        }

        Ok(Stmt::Expr(expr))
    }

    // Validate that the expression can be used as an assignment target
    fn validate_assignment_target(&self, expr: &Expr) -> Result<(), ParseError> {
        match expr {
            Expr::Identifier(_) | Expr::Index(_, _) | Expr::Member(_, _) => Ok(()),
            _ => Err(ParseError::InvalidAssignmentTarget),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.null_coalesce()
    }

    // Parse null coalescing: expr ?? expr (lower precedence than ||, short-circuit)
    fn null_coalesce(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_or()?;
        while self.match_token(&[TokenKind::QuestionQuestion]) {
            let right = self.logical_or()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::NullCoalesce, Box::new(right));
        }
        Ok(expr)
    }

    // Parse logical OR: expr || expr
    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;

        while self.match_token(&[TokenKind::Or]) {
            let right = self.logical_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    // Parse logical AND: expr && expr
    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenKind::And]) {
            let right = self.equality()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }

        Ok(expr)
    }

    // Parse equality: expr == expr, expr != expr
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenKind::EqualEqual, TokenKind::NotEqual]) {
            let op = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    // Parse comparison: expr < expr, expr > expr, expr <= expr, expr >= expr
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.addition()?;

        while self.match_token(&[
            TokenKind::Less,
            TokenKind::Greater,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
        ]) {
            let op = match self.previous().kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.addition()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    // Parse addition and subtraction: expr + expr, expr - expr
    fn addition(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.multiplication()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.multiplication()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    // Parse multiplication, division, and modulo: expr * expr, expr / expr, expr % expr
    fn multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match self.previous().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    // Parse unary expressions: -expr, !expr
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenKind::Await]) {
            let expr = self.unary()?;
            return Ok(Expr::Await(Box::new(expr)));
        }
        if self.match_token(&[TokenKind::Minus, TokenKind::Not]) {
            let op = match self.previous().kind {
                TokenKind::Minus => UnaryOp::Negate,
                TokenKind::Not => UnaryOp::Not,
                _ => unreachable!(),
            };
            let expr = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(expr)));
        }

        self.call()
    }

    // Parse postfix operations: function calls, indexing, member access
    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenKind::LeftParen]) {
                // Function call
                expr = self.finish_call(expr)?;
            } else if self.peek().kind == TokenKind::LeftBracket
                && self.peek().line == self.previous().line
            {
                // Array indexing or slice (only on same line to avoid ambiguity with array literals)
                self.advance(); // consume '['
                if self.check(&TokenKind::Colon) {
                    // [:end] or [:]
                    self.advance(); // consume ':'
                    let end = if self.check(&TokenKind::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.expression()?))
                    };
                    self.consume(TokenKind::RightBracket, "]")?;
                    expr = Expr::Slice(Box::new(expr), None, end);
                } else {
                    let first = self.expression()?;
                    if self.match_token(&[TokenKind::Colon]) {
                        // [start:end] or [start:]
                        let end = if self.check(&TokenKind::RightBracket) {
                            None
                        } else {
                            Some(Box::new(self.expression()?))
                        };
                        self.consume(TokenKind::RightBracket, "]")?;
                        expr = Expr::Slice(Box::new(expr), Some(Box::new(first)), end);
                    } else {
                        self.consume(TokenKind::RightBracket, "]")?;
                        expr = Expr::Index(Box::new(expr), Box::new(first));
                    }
                }
            } else if self.match_token(&[TokenKind::Dot]) {
                // Member access
                let member = if let TokenKind::Identifier(name) = &self.peek().kind {
                    name.clone()
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "property name".to_string(),
                        found: self.peek().clone(),
                    });
                };
                self.advance();
                expr = Expr::Member(Box::new(expr), member);
            } else if self.match_token(&[TokenKind::QuestionDot]) {
                // Optional chaining: expr?.member or expr?.method(args)
                let member = if let TokenKind::Identifier(name) = &self.peek().kind {
                    name.clone()
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "property name after ?.".to_string(),
                        found: self.peek().clone(),
                    });
                };
                self.advance();
                // Check if this is a method call: expr?.method(args)
                if self.match_token(&[TokenKind::LeftParen]) {
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RightParen, ")")?;
                    expr = Expr::OptionalCall(Box::new(expr), member, args);
                } else {
                    expr = Expr::OptionalMember(Box::new(expr), member);
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // Parse argument list and create Call expression
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, ")")?;

        Ok(Expr::Call(Box::new(callee), arguments))
    }

    // Parse primary expressions: literals, identifiers, grouped expressions, arrays, function expressions
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance().clone();

        match &token.kind {
            TokenKind::Integer(n) => Ok(Expr::Integer(*n)),
            TokenKind::Float(f) => Ok(Expr::Float(*f)),
            TokenKind::String(s) => Ok(Expr::String(s.clone())),
            TokenKind::StringInterp(parts) => {
                use crate::lexer::token::StringPart;
                let parts = parts.clone();
                let mut exprs: Vec<Expr> = Vec::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => exprs.push(Expr::String(s)),
                        StringPart::Placeholder(src) => {
                            // Re-lex and re-parse the placeholder expression
                            let mut scanner = crate::lexer::Scanner::new(&src);
                            let tokens = scanner.scan_tokens().map_err(|_e| {
                                ParseError::UnexpectedToken {
                                    expected: "valid expression in interpolation".to_string(),
                                    found: token.clone(),
                                }
                            })?;
                            let mut inner_parser = crate::parser::Parser::new(tokens);
                            exprs.push(inner_parser.expression()?);
                        }
                    }
                }
                Ok(Expr::StringInterp(exprs))
            }
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Null => Ok(Expr::Null),
            TokenKind::Identifier(name) => {
                let name = name.clone();
                // Same-line '{' after identifier may be struct init.
                // Disambiguate: struct init requires 'identifier :' or '}' inside the braces.
                if self.peek().kind == TokenKind::LeftBrace
                    && self.peek().line == self.previous().line
                    && self.is_struct_init()
                {
                    self.advance(); // consume '{'
                    return self.struct_init(name);
                }
                Ok(Expr::Identifier(name))
            }
            TokenKind::Fn => self.function_expression(),
            TokenKind::Async => {
                self.consume(TokenKind::Fn, "fn after async")?;
                self.async_function_expression()
            }
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, ")")?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                // Array literal (supports spread: [...arr, elem])
                let mut elements = Vec::new();

                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        if self.match_token(&[TokenKind::Spread]) {
                            let expr = self.expression()?;
                            elements.push(Expr::Spread(Box::new(expr)));
                        } else {
                            elements.push(self.expression()?);
                        }
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBracket, "]")?;
                Ok(Expr::Array(elements))
            }
            TokenKind::LeftBrace => {
                // Dict literal: { key: value, ... }
                let mut pairs = Vec::new();

                if !self.check(&TokenKind::RightBrace) {
                    loop {
                        let key = self.expression()?;
                        self.consume(TokenKind::Colon, ":")?;
                        let value = self.expression()?;
                        pairs.push((key, value));
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBrace, "}")?;
                Ok(Expr::Dict(pairs))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: token,
            }),
        }
    }

    // Parse function expression: fn(params) { body }
    fn function_expression(&mut self) -> Result<Expr, ParseError> {
        let params = self.parse_params()?;
        self.consume(TokenKind::LeftBrace, "{")?;
        let body = self.block_statement()?;
        Ok(Expr::FunctionExpr(params, Rc::new(body)))
    }

    // Parse async function declaration: async fn name(params) { body }
    fn async_function_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "function name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        let params = self.parse_params()?;
        self.consume(TokenKind::LeftBrace, "{")?;
        let body = self.block_statement()?;
        Ok(Stmt::AsyncFunction(name, params, Rc::new(body)))
    }

    // Parse async function expression: async fn(params) { body }
    fn async_function_expression(&mut self) -> Result<Expr, ParseError> {
        let params = self.parse_params()?;
        self.consume(TokenKind::LeftBrace, "{")?;
        let body = self.block_statement()?;
        Ok(Expr::AsyncFunctionExpr(params, Rc::new(body)))
    }

    // Returns true if the upcoming '{' begins a struct init (identifier ':' or just '}')
    // self.peek() must be '{' before calling this.
    fn is_struct_init(&self) -> bool {
        // peek_at(0) is '{', peek_at(1) is what's inside
        let inside = self.peek_at(1);
        match &inside.kind {
            // Empty braces: Name {} — could be struct init
            TokenKind::RightBrace => true,
            // identifier followed by ':' — struct field
            TokenKind::Identifier(_) => {
                matches!(self.peek_at(2).kind, TokenKind::Colon)
            }
            _ => false,
        }
    }

    // Parse struct declaration: struct Name { field, ... fn method(self, ...) { body } }
    fn struct_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "struct name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        self.consume(TokenKind::LeftBrace, "{")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.match_token(&[TokenKind::Fn]) {
                // Method declaration: fn name(self, ...) { body }
                let method_name = if let TokenKind::Identifier(n) = &self.peek().kind {
                    n.clone()
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "method name".to_string(),
                        found: self.peek().clone(),
                    });
                };
                self.advance();

                let params = self.parse_params()?;
                self.consume(TokenKind::LeftBrace, "{")?;
                let body = self.block_statement()?;
                methods.push((method_name, params, Box::new(body)));
            } else if let TokenKind::Identifier(field) = &self.peek().kind {
                // Field declaration (comma-separated: x, y or one per line)
                fields.push(field.clone());
                self.advance();
                while self.match_token(&[TokenKind::Comma]) {
                    if let TokenKind::Identifier(f) = &self.peek().kind {
                        fields.push(f.clone());
                        self.advance();
                    } else {
                        break;
                    }
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "field name or fn".to_string(),
                    found: self.peek().clone(),
                });
            }
        }

        self.consume(TokenKind::RightBrace, "}")?;

        Ok(Stmt::StructDecl {
            name,
            fields,
            methods,
        })
    }

    // Parse enum declaration: enum Name { Variant(fields) UnitVariant ... }
    fn enum_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "enum name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();
        self.consume(TokenKind::LeftBrace, "{")?;

        let mut variants: Vec<(String, Vec<String>)> = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let variant_name = if let TokenKind::Identifier(v) = &self.peek().kind {
                v.clone()
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "variant name".to_string(),
                    found: self.peek().clone(),
                });
            };
            self.advance();

            let fields = if self.match_token(&[TokenKind::LeftParen]) {
                let mut f = Vec::new();
                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    if let TokenKind::Identifier(field) = &self.peek().kind {
                        f.push(field.clone());
                        self.advance();
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            expected: "field name".to_string(),
                            found: self.peek().clone(),
                        });
                    }
                }
                self.consume(TokenKind::RightParen, ")")?;
                f
            } else {
                Vec::new()
            };

            variants.push((variant_name, fields));
        }

        self.consume(TokenKind::RightBrace, "}")?;
        Ok(Stmt::EnumDecl { name, variants })
    }

    // Parse struct initialization: StructName { field: value, ... }
    fn struct_init(&mut self, name: String) -> Result<Expr, ParseError> {
        let mut fields = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let field_name = if let TokenKind::Identifier(n) = &self.peek().kind {
                    n.clone()
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "field name".to_string(),
                        found: self.peek().clone(),
                    });
                };
                self.advance();
                self.consume(TokenKind::Colon, ":")?;
                let value = self.expression()?;
                fields.push((field_name, value));
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightBrace, "}")?;
        Ok(Expr::StructInit { name, fields })
    }

    // Parse import statement: import module [as alias]
    fn import_statement(&mut self) -> Result<Stmt, ParseError> {
        let module_name = if let TokenKind::Identifier(name) = &self.peek().kind {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "module name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        // Check for 'as alias'
        if self.match_token(&[TokenKind::As]) {
            let alias = if let TokenKind::Identifier(name) = &self.peek().kind {
                name.clone()
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "alias name".to_string(),
                    found: self.peek().clone(),
                });
            };
            self.advance();
            Ok(Stmt::ImportAs(module_name, alias))
        } else {
            Ok(Stmt::Import(module_name))
        }
    }

    // Parse from import statement: from module import item1, item2, ...
    fn parse_from_import_statement(&mut self) -> Result<Stmt, ParseError> {
        let module_name = if let TokenKind::Identifier(name) = &self.peek().kind {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "module name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        self.consume(TokenKind::Import, "import")?;

        // Parse list of items to import, each optionally followed by "as alias"
        let mut items: Vec<String> = Vec::new();
        let mut aliased: Vec<(String, String)> = Vec::new();
        let mut has_alias = false;

        loop {
            let item = if let TokenKind::Identifier(name) = &self.peek().kind {
                let n = name.clone();
                self.advance();
                n
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "item name".to_string(),
                    found: self.peek().clone(),
                });
            };

            if self.match_token(&[TokenKind::As]) {
                has_alias = true;
                let alias = if let TokenKind::Identifier(name) = &self.peek().kind {
                    let n = name.clone();
                    self.advance();
                    n
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "alias name".to_string(),
                        found: self.peek().clone(),
                    });
                };
                aliased.push((item.clone(), alias));
            } else {
                aliased.push((item.clone(), item.clone()));
            }
            items.push(item);

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        if has_alias {
            Ok(Stmt::FromImportAs(module_name, aliased))
        } else {
            Ok(Stmt::FromImport(module_name, items))
        }
    }

    // Parse try/catch: try { ... } catch(e) { ... }
    fn try_catch_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftBrace, "{")?;
        let try_body = self.block_statement()?;

        self.consume(TokenKind::Catch, "catch")?;
        self.consume(TokenKind::LeftParen, "(")?;

        let error_var = if let TokenKind::Identifier(name) = &self.peek().kind {
            name.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "error variable name".to_string(),
                found: self.peek().clone(),
            });
        };
        self.advance();

        self.consume(TokenKind::RightParen, ")")?;
        self.consume(TokenKind::LeftBrace, "{")?;
        let catch_body = self.block_statement()?;

        let finally_body = if self.check(&TokenKind::Finally) {
            self.advance();
            self.consume(TokenKind::LeftBrace, "{")?;
            Some(Box::new(self.block_statement()?))
        } else {
            None
        };

        Ok(Stmt::TryCatch(
            Box::new(try_body),
            error_var,
            Box::new(catch_body),
            finally_body,
        ))
    }

    // Parse throw: throw expr
    fn throw_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        Ok(Stmt::Throw(value))
    }

    // Parse a single pattern for a match arm
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let pat = self.parse_single_pattern()?;

        // Or-pattern: pat1 | pat2 | ...
        if self.check(&TokenKind::Pipe) {
            let mut alts = vec![pat];
            while self.match_token(&[TokenKind::Pipe]) {
                alts.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(alts));
        }

        Ok(pat)
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Wildcard: _
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if name == "_" {
                self.advance();
                return Ok(Pattern::Wildcard);
            }
        }

        // Literal patterns
        if self.match_token(&[TokenKind::True]) {
            return Ok(Pattern::Literal(Expr::Bool(true)));
        }
        if self.match_token(&[TokenKind::False]) {
            return Ok(Pattern::Literal(Expr::Bool(false)));
        }
        if self.match_token(&[TokenKind::Null]) {
            return Ok(Pattern::Literal(Expr::Null));
        }
        if self.match_token(&[TokenKind::Minus]) {
            match self.peek().kind {
                TokenKind::Integer(n) => {
                    let n = -n;
                    self.advance();
                    return Ok(Pattern::Literal(Expr::Integer(n)));
                }
                TokenKind::Float(f) => {
                    let f = -f;
                    self.advance();
                    return Ok(Pattern::Literal(Expr::Float(f)));
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "number after '-' in pattern".to_string(),
                        found: self.peek().clone(),
                    });
                }
            }
        }
        if let TokenKind::Integer(n) = self.peek().kind {
            self.advance();
            return Ok(Pattern::Literal(Expr::Integer(n)));
        }
        if let TokenKind::Float(f) = self.peek().kind {
            self.advance();
            return Ok(Pattern::Literal(Expr::Float(f)));
        }
        if let TokenKind::String(s) = &self.peek().kind {
            let s = s.clone();
            self.advance();
            return Ok(Pattern::Literal(Expr::String(s)));
        }

        // Identifier: Enum.Variant(fields), Enum.Variant, or bind variable
        if let TokenKind::Identifier(first) = &self.peek().kind {
            let first = first.clone();
            self.advance();

            // Enum.Variant pattern: Identifier followed by '.' on same line
            if self.peek().kind == TokenKind::Dot && self.peek().line == self.previous().line {
                self.advance(); // consume '.'
                let variant = if let TokenKind::Identifier(v) = &self.peek().kind {
                    let v = v.clone();
                    self.advance();
                    v
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "variant name after '.'".to_string(),
                        found: self.peek().clone(),
                    });
                };
                let fields = if self.match_token(&[TokenKind::LeftParen]) {
                    let mut f = Vec::new();
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        if let TokenKind::Identifier(field) = &self.peek().kind {
                            f.push(field.clone());
                            self.advance();
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    self.consume(TokenKind::RightParen, ")")?;
                    f
                } else {
                    Vec::new()
                };
                return Ok(Pattern::EnumVariant(first, Some(variant), fields));
            }

            // Plain identifier: bind variable
            return Ok(Pattern::Bind(first));
        }

        Err(ParseError::UnexpectedToken {
            expected: "pattern".to_string(),
            found: self.peek().clone(),
        })
    }

    // Parse match statement: match expr { pattern => stmt, ... }
    fn parse_match_statement(&mut self) -> Result<Stmt, ParseError> {
        let subject = self.expression()?;
        self.consume(TokenKind::LeftBrace, "{")?;

        let mut arms: Vec<(Pattern, Box<Stmt>)> = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(TokenKind::FatArrow, "=>")?;
            arms.push((pattern, Box::new(self.match_arm_body()?)));
            self.match_token(&[TokenKind::Comma]);
        }

        self.consume(TokenKind::RightBrace, "}")?;
        Ok(Stmt::Match { subject, arms })
    }

    fn match_arm_body(&mut self) -> Result<Stmt, ParseError> {
        if self.check(&TokenKind::LeftBrace) {
            self.advance(); // consume '{'
            self.block_statement()
        } else {
            self.statement()
        }
    }
}
