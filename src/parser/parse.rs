//! Recursive descent parser for Aether

use crate::lexer::{Token, TokenKind};
use super::ast::*;
use std::fmt;

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
                write!(f, "Expected {}, found '{}' at line {}", expected, found.lexeme, found.line)
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

    // Parse declarations (let statements, function declarations)
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenKind::Let]) {
            return self.let_declaration();
        }
        if self.match_token(&[TokenKind::Fn]) {
            return self.function_declaration();
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

        // Parse parameter list
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

        // Parse function body (must be a block)
        self.consume(TokenKind::LeftBrace, "{")?;
        let body = self.block_statement()?;

        Ok(Stmt::Function(name, params, Box::new(body)))
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
            return Ok(Stmt::Break);
        }
        if self.match_token(&[TokenKind::Continue]) {
            return Ok(Stmt::Continue);
        }
        if self.match_token(&[TokenKind::LeftBrace]) {
            return self.block_statement();
        }
        self.expression_statement()
    }

    // Parse block statement: { stmt* }
    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "}")?;
        Ok(Stmt::Block(statements))
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
        self.logical_or()
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
            } else if self.match_token(&[TokenKind::LeftBracket]) {
                // Array indexing
                let index = self.expression()?;
                self.consume(TokenKind::RightBracket, "]")?;
                expr = Expr::Index(Box::new(expr), Box::new(index));
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

    // Parse primary expressions: literals, identifiers, grouped expressions, arrays
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance().clone();

        match &token.kind {
            TokenKind::Integer(n) => Ok(Expr::Integer(*n)),
            TokenKind::Float(f) => Ok(Expr::Float(*f)),
            TokenKind::String(s) => Ok(Expr::String(s.clone())),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Null => Ok(Expr::Null),
            TokenKind::Identifier(name) => Ok(Expr::Identifier(name.clone())),
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, ")")?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                // Array literal
                let mut elements = Vec::new();

                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBracket, "]")?;
                Ok(Expr::Array(elements))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: token,
            }),
        }
    }
}
