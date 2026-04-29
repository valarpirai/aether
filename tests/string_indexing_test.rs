//! Tests for string indexing feature

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    // Execute all but last statement
    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // Evaluate last statement if it's an expression
    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// Basic string indexing
#[test]
fn test_string_index_first_char() {
    let result = eval(
        r#"let s = "hello"
s[0]"#,
    )
    .unwrap();
    assert_eq!(result, "h");
}

#[test]
fn test_string_index_middle_char() {
    let result = eval(
        r#"let s = "hello"
s[2]"#,
    )
    .unwrap();
    assert_eq!(result, "l");
}

#[test]
fn test_string_index_last_char() {
    let result = eval(
        r#"let s = "hello"
s[4]"#,
    )
    .unwrap();
    assert_eq!(result, "o");
}

#[test]
fn test_string_index_literal() {
    let result = eval(r#""world"[1]"#).unwrap();
    assert_eq!(result, "o");
}

// UTF-8 support
#[test]
fn test_string_index_utf8() {
    let result = eval(
        r#"let s = "世界"
s[0]"#,
    )
    .unwrap();
    assert_eq!(result, "世");
}

#[test]
fn test_string_index_emoji() {
    let result = eval(
        r#"let s = "🎉🎊🎈"
s[1]"#,
    )
    .unwrap();
    assert_eq!(result, "🎊");
}

// Error cases
#[test]
fn test_string_index_negative() {
    let result = eval(
        r#"let s = "hello"
s[-1]"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("out of bounds") || err.contains("Index"));
}

#[test]
fn test_string_index_out_of_bounds() {
    let result = eval(
        r#"let s = "hello"
s[10]"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("out of bounds") || err.contains("Index"));
}

// Use cases
#[test]
fn test_string_index_multiple_access() {
    let result = eval(
        r#"
let s = "abc"
s[0] + s[1] + s[2]
"#,
    )
    .unwrap();
    assert_eq!(result, "abc");
}

#[test]
fn test_string_index_comparison() {
    let result = eval(
        r#"
let s = "hello"
s[0] == "h"
"#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_string_index_concatenation() {
    let result = eval(
        r#"
let s = "world"
let first = s[0]
let last = s[4]
first + last
"#,
    )
    .unwrap();
    assert_eq!(result, "wd");
}

// Edge cases
#[test]
fn test_empty_string_index() {
    let result = eval(
        r#"
let s = ""
s[0]
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_single_char_string_index() {
    let result = eval(
        r#"
let s = "x"
s[0]
"#,
    )
    .unwrap();
    assert_eq!(result, "x");
}

// Integration with other features
#[test]
fn test_string_index_with_variables() {
    let result = eval(
        r#"
let s = "testing"
let idx = 2
s[idx]
"#,
    )
    .unwrap();
    assert_eq!(result, "s");
}

#[test]
fn test_string_index_with_expression() {
    let result = eval(
        r#"
let s = "example"
s[1 + 1]
"#,
    )
    .unwrap();
    assert_eq!(result, "a");
}

#[test]
fn test_multiple_string_indexes() {
    let result = eval(
        r#"
let s = "code"
s[0] + s[1] + s[2] + s[3]
"#,
    )
    .unwrap();
    assert_eq!(result, "code");
}
