//! Tests for member access (obj.property)
//! These tests are written FIRST following TDD red-green-refactor

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

/// Helper to evaluate expression
fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut evaluator = Evaluator::new();

    // Execute all but last statement
    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // Evaluate last statement if it's an expression
    if let Some(last) = program.statements.last() {
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
    }

    Ok("null".to_string())
}

#[test]
fn test_array_length_property() {
    let result = eval(r#"
        let arr = [1, 2, 3, 4, 5]
        arr.length
    "#);
    assert_eq!(result.unwrap(), "5");
}

#[test]
fn test_string_length_property() {
    let result = eval(r#"
        let text = "hello"
        text.length
    "#);
    assert_eq!(result.unwrap(), "5");
}

#[test]
fn test_empty_array_length() {
    let result = eval(r#"
        let arr = []
        arr.length
    "#);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_empty_string_length() {
    let result = eval(r#"
        let text = ""
        text.length
    "#);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_direct_literal_member_access() {
    assert_eq!(eval(r#"[1, 2, 3].length"#).unwrap(), "3");
    assert_eq!(eval(r#""test".length"#).unwrap(), "4");
}

#[test]
fn test_member_access_in_expression() {
    let result = eval(r#"
        let arr = [10, 20, 30]
        arr.length + 5
    "#);
    assert_eq!(result.unwrap(), "8");
}

#[test]
fn test_undefined_property_error() {
    let result = eval(r#"
        let arr = [1, 2, 3]
        arr.notexist
    "#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    eprintln!("Error message: {}", err);
    assert!(err.contains("Property") || err.contains("property") || err.contains("member"));
}

#[test]
fn test_member_access_on_non_object() {
    let result = eval(r#"
        let num = 42
        num.length
    "#);
    assert!(result.is_err());
}
