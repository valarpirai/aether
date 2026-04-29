//! Tests for array and string slice syntax: arr[start:end]

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// --- Array slices ---

#[test]
fn test_array_slice_start_end() {
    let result = eval("[1, 2, 3, 4, 5][1:3]");
    assert_eq!(result.unwrap(), "[2, 3]");
}

#[test]
fn test_array_slice_from_start() {
    let result = eval("[1, 2, 3, 4, 5][:3]");
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_slice_to_end() {
    let result = eval("[1, 2, 3, 4, 5][2:]");
    assert_eq!(result.unwrap(), "[3, 4, 5]");
}

#[test]
fn test_array_slice_full_copy() {
    let result = eval("[1, 2, 3][:]");
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_slice_empty_result() {
    let result = eval("[1, 2, 3][2:1]");
    assert_eq!(result.unwrap(), "[]");
}

#[test]
fn test_array_slice_negative_start() {
    let result = eval("[1, 2, 3, 4, 5][-2:]");
    assert_eq!(result.unwrap(), "[4, 5]");
}

#[test]
fn test_array_slice_negative_end() {
    let result = eval("[1, 2, 3, 4, 5][:-1]");
    assert_eq!(result.unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_array_slice_out_of_bounds_clamped() {
    let result = eval("[1, 2, 3][0:100]");
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_slice_via_variable() {
    let result = eval(
        r#"
        let arr = [10, 20, 30, 40, 50]
        arr[1:4]
    "#,
    );
    assert_eq!(result.unwrap(), "[20, 30, 40]");
}

// --- String slices ---

#[test]
fn test_string_slice_start_end() {
    let result = eval(r#""hello"[1:3]"#);
    assert_eq!(result.unwrap(), "el");
}

#[test]
fn test_string_slice_from_start() {
    let result = eval(r#""hello"[:3]"#);
    assert_eq!(result.unwrap(), "hel");
}

#[test]
fn test_string_slice_to_end() {
    let result = eval(r#""hello"[2:]"#);
    assert_eq!(result.unwrap(), "llo");
}

#[test]
fn test_string_slice_full_copy() {
    let result = eval(r#""hello"[:]"#);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_string_slice_negative_start() {
    let result = eval(r#""hello"[-3:]"#);
    assert_eq!(result.unwrap(), "llo");
}

#[test]
fn test_string_slice_empty_result() {
    let result = eval(r#""hello"[3:1]"#);
    assert_eq!(result.unwrap(), "");
}
