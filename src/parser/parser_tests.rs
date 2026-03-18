//! Tests for the parser module

use super::ast::*;
use super::parse::Parser;
use crate::lexer::Scanner;

// AST tests
#[test]
fn test_expr_integer() {
    let expr = Expr::Integer(42);
    assert_eq!(expr, Expr::Integer(42));
}

#[test]
fn test_expr_binary() {
    let left = Box::new(Expr::Integer(1));
    let right = Box::new(Expr::Integer(2));
    let expr = Expr::Binary(left, BinaryOp::Add, right);

    if let Expr::Binary(_, op, _) = expr {
        assert_eq!(op, BinaryOp::Add);
    } else {
        panic!("Expected Binary expression");
    }
}

#[test]
fn test_stmt_let() {
    let stmt = Stmt::Let("x".to_string(), Expr::Integer(42));
    if let Stmt::Let(name, _) = stmt {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Let statement");
    }
}

// Parser tests
#[test]
fn test_parser_creation() {
    let mut scanner = Scanner::new("42");
    let tokens = scanner.scan_tokens().unwrap();
    let _parser = Parser::new(tokens);
    // Parser created successfully
}

#[test]
fn test_parse_integer() {
    let mut scanner = Scanner::new("42");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Expr(Expr::Integer(n)) => assert_eq!(*n, 42),
        _ => panic!("Expected integer expression"),
    }
}

#[test]
fn test_parse_float() {
    let mut scanner = Scanner::new("3.14");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Float(f)) => assert_eq!(*f, 3.14),
        _ => panic!("Expected float expression"),
    }
}

#[test]
fn test_parse_string() {
    let mut scanner = Scanner::new("\"hello\"");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::String(s)) => assert_eq!(s, "hello"),
        _ => panic!("Expected string expression"),
    }
}

#[test]
fn test_parse_bool() {
    let mut scanner = Scanner::new("true");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Bool(b)) => assert_eq!(*b, true),
        _ => panic!("Expected bool expression"),
    }
}

#[test]
fn test_parse_identifier() {
    let mut scanner = Scanner::new("foo");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Identifier(name)) => assert_eq!(name, "foo"),
        _ => panic!("Expected identifier expression"),
    }
}

#[test]
fn test_parse_negate() {
    let mut scanner = Scanner::new("-42");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Unary(UnaryOp::Negate, operand)) => match **operand {
            Expr::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer"),
        },
        _ => panic!("Expected unary negate expression"),
    }
}

#[test]
fn test_parse_not() {
    let mut scanner = Scanner::new("!true");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Unary(UnaryOp::Not, operand)) => match **operand {
            Expr::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected bool"),
        },
        _ => panic!("Expected unary not expression"),
    }
}

#[test]
fn test_parse_addition() {
    let mut scanner = Scanner::new("1 + 2");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::Add),
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_multiplication() {
    let mut scanner = Scanner::new("3 * 4");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::Multiply),
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_precedence() {
    let mut scanner = Scanner::new("1 + 2 * 3");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    // Should parse as: 1 + (2 * 3)
    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(left, op1, right)) => {
            assert_eq!(*op1, BinaryOp::Add);
            assert!(matches!(**left, Expr::Integer(1)));
            // Right side should be 2 * 3
            match &**right {
                Expr::Binary(_, op2, _) => assert_eq!(*op2, BinaryOp::Multiply),
                _ => panic!("Expected multiplication on right side"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_comparison() {
    let mut scanner = Scanner::new("1 < 2");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::Less),
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_equality() {
    let mut scanner = Scanner::new("1 == 2");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::Equal),
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_logical_and() {
    let mut scanner = Scanner::new("true && false");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::And),
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_parse_logical_or() {
    let mut scanner = Scanner::new("true || false");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Binary(_, op, _)) => assert_eq!(*op, BinaryOp::Or),
        _ => panic!("Expected binary expression"),
    }
}

// Statement tests
#[test]
fn test_parse_let_declaration() {
    let mut scanner = Scanner::new("let x = 42");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let(name, expr) => {
            assert_eq!(name, "x");
            assert!(matches!(expr, Expr::Integer(42)));
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_let_with_expression() {
    let mut scanner = Scanner::new("let y = 1 + 2");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Let(name, expr) => {
            assert_eq!(name, "y");
            assert!(matches!(expr, Expr::Binary(_, _, _)));
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_block() {
    let mut scanner = Scanner::new("{ 42 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Block(statements) => {
            assert_eq!(statements.len(), 1);
        }
        _ => panic!("Expected block statement"),
    }
}

#[test]
fn test_parse_if_statement() {
    let mut scanner = Scanner::new("if (true) { 42 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::If(condition, _, _) => {
            assert!(matches!(condition, Expr::Bool(true)));
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_else_statement() {
    let mut scanner = Scanner::new("if (true) { 1 } else { 2 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::If(_, _, else_branch) => {
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected if-else statement"),
    }
}

#[test]
fn test_parse_while_loop() {
    let mut scanner = Scanner::new("while (true) { 42 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::While(condition, _) => {
            assert!(matches!(condition, Expr::Bool(true)));
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_for_loop() {
    let mut scanner = Scanner::new("for i in items { 42 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::For(var, iterable, _) => {
            assert_eq!(var, "i");
            assert!(matches!(iterable, Expr::Identifier(_)));
        }
        _ => panic!("Expected for statement"),
    }
}

// Function declaration tests
#[test]
fn test_parse_function_declaration() {
    let mut scanner = Scanner::new("fn add(a, b) { return a + b }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Function(name, params, body) => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert!(matches!(**body, Stmt::Block(_)));
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_parse_function_no_params() {
    let mut scanner = Scanner::new("fn greet() { return 42 }");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Function(name, params, _) => {
            assert_eq!(name, "greet");
            assert_eq!(params.len(), 0);
        }
        _ => panic!("Expected function declaration"),
    }
}

// Function call tests
#[test]
fn test_parse_function_call_no_args() {
    let mut scanner = Scanner::new("foo()");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Call(func, args)) => {
            assert!(matches!(**func, Expr::Identifier(_)));
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let mut scanner = Scanner::new("add(1, 2)");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Call(func, args)) => {
            assert!(matches!(**func, Expr::Identifier(_)));
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], Expr::Integer(1)));
            assert!(matches!(args[1], Expr::Integer(2)));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parse_nested_function_calls() {
    let mut scanner = Scanner::new("outer(inner(42))");
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::Expr(Expr::Call(func, args)) => {
            assert!(matches!(**func, Expr::Identifier(_)));
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expr::Call(_, _)));
        }
        _ => panic!("Expected function call"),
    }
}
