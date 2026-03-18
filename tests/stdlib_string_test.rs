//! Tests for standard library string module
//! TDD: Written FIRST before implementing the module

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

// join() tests
#[test]
fn test_join_with_space() {
    assert_eq!(eval(r#"join(["hello", "world"], " ")"#).unwrap(), "hello world");
}

#[test]
fn test_join_with_comma() {
    assert_eq!(eval(r#"join(["a", "b", "c"], ", ")"#).unwrap(), "a, b, c");
}

#[test]
fn test_join_empty_separator() {
    assert_eq!(eval(r#"join(["hello", "world"], "")"#).unwrap(), "helloworld");
}

#[test]
fn test_join_empty_array() {
    assert_eq!(eval(r#"join([], ", ")"#).unwrap(), "");
}

#[test]
fn test_join_single_element() {
    assert_eq!(eval(r#"join(["alone"], ", ")"#).unwrap(), "alone");
}

// repeat() tests
#[test]
fn test_repeat_normal() {
    assert_eq!(eval(r#"repeat("ha", 3)"#).unwrap(), "hahaha");
    assert_eq!(eval(r#"repeat("*", 5)"#).unwrap(), "*****");
}

#[test]
fn test_repeat_once() {
    assert_eq!(eval(r#"repeat("hello", 1)"#).unwrap(), "hello");
}

#[test]
fn test_repeat_zero() {
    assert_eq!(eval(r#"repeat("x", 0)"#).unwrap(), "");
}

#[test]
fn test_repeat_empty_string() {
    assert_eq!(eval(r#"repeat("", 5)"#).unwrap(), "");
}

// reverse() tests
#[test]
fn test_reverse_normal() {
    assert_eq!(eval(r#"reverse("hello")"#).unwrap(), "olleh");
    assert_eq!(eval(r#"reverse("world")"#).unwrap(), "dlrow");
}

#[test]
fn test_reverse_single_char() {
    assert_eq!(eval(r#"reverse("a")"#).unwrap(), "a");
}

#[test]
fn test_reverse_empty() {
    assert_eq!(eval(r#"reverse("")"#).unwrap(), "");
}

#[test]
fn test_reverse_palindrome() {
    assert_eq!(eval(r#"reverse("racecar")"#).unwrap(), "racecar");
}

// starts_with() tests
#[test]
fn test_starts_with_true() {
    assert_eq!(eval(r#"starts_with("hello world", "hello")"#).unwrap(), "true");
    assert_eq!(eval(r#"starts_with("test", "te")"#).unwrap(), "true");
}

#[test]
fn test_starts_with_false() {
    assert_eq!(eval(r#"starts_with("hello", "world")"#).unwrap(), "false");
    assert_eq!(eval(r#"starts_with("test", "st")"#).unwrap(), "false");
}

#[test]
fn test_starts_with_empty_prefix() {
    assert_eq!(eval(r#"starts_with("hello", "")"#).unwrap(), "true");
}

#[test]
fn test_starts_with_same_string() {
    assert_eq!(eval(r#"starts_with("test", "test")"#).unwrap(), "true");
}

// ends_with() tests
#[test]
fn test_ends_with_true() {
    assert_eq!(eval(r#"ends_with("hello world", "world")"#).unwrap(), "true");
    assert_eq!(eval(r#"ends_with("test", "st")"#).unwrap(), "true");
}

#[test]
fn test_ends_with_false() {
    assert_eq!(eval(r#"ends_with("hello", "world")"#).unwrap(), "false");
    assert_eq!(eval(r#"ends_with("test", "te")"#).unwrap(), "false");
}

#[test]
fn test_ends_with_empty_suffix() {
    assert_eq!(eval(r#"ends_with("hello", "")"#).unwrap(), "true");
}

#[test]
fn test_ends_with_same_string() {
    assert_eq!(eval(r#"ends_with("test", "test")"#).unwrap(), "true");
}

// Composition tests
#[test]
fn test_join_split_roundtrip() {
    let result = eval(r#"
        let words = ["hello", "world"]
        let sentence = join(words, " ")
        sentence
    "#);
    assert_eq!(result.unwrap(), "hello world");
}

#[test]
fn test_repeat_join() {
    let result = eval(r#"
        join([repeat("*", 3), repeat("-", 3)], " ")
    "#);
    assert_eq!(result.unwrap(), "*** ---");
}

#[test]
fn test_reverse_uppercase() {
    let result = eval(r#"
        reverse("hello".upper())
    "#);
    assert_eq!(result.unwrap(), "OLLEH");
}
