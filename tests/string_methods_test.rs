//! Tests for string methods
//! Written FIRST following TDD red-green-refactor

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut evaluator = Evaluator::new();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
    }

    Ok("null".to_string())
}

#[test]
fn test_string_upper() {
    assert_eq!(eval(r#""hello".upper()"#).unwrap(), "HELLO");
    assert_eq!(eval(r#""Hello World".upper()"#).unwrap(), "HELLO WORLD");
}

#[test]
fn test_string_lower() {
    assert_eq!(eval(r#""HELLO".lower()"#).unwrap(), "hello");
    assert_eq!(eval(r#""Hello World".lower()"#).unwrap(), "hello world");
}

#[test]
fn test_string_trim() {
    assert_eq!(eval(r#""  hello  ".trim()"#).unwrap(), "hello");
    assert_eq!(eval(r#""no spaces".trim()"#).unwrap(), "no spaces");
    assert_eq!(eval(r#""  \t\n  ".trim()"#).unwrap(), "");
}

#[test]
fn test_string_split() {
    assert_eq!(eval(r#""a,b,c".split(",")"#).unwrap(), "[a, b, c]");
    assert_eq!(
        eval(r#""hello world".split(" ")"#).unwrap(),
        "[hello, world]"
    );
}

#[test]
fn test_string_split_empty() {
    assert_eq!(eval(r#""".split(",")"#).unwrap(), "[]");
}

#[test]
fn test_string_method_chaining() {
    let result = eval(
        r#"
        let text = "  Hello World  "
        text.trim().lower()
    "#,
    );
    assert_eq!(result.unwrap(), "hello world");
}

#[test]
fn test_string_methods_with_variables() {
    let result = eval(
        r#"
        let text = "hello"
        text.upper()
    "#,
    );
    assert_eq!(result.unwrap(), "HELLO");
}

#[test]
fn test_string_split_result_is_array() {
    let result = eval(
        r#"
        let parts = "a,b,c".split(",")
        parts.length
    "#,
    );
    assert_eq!(result.unwrap(), "3");
}
