//! Tests for array spread operator: [...arr1, ...arr2]

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

#[test]
fn test_spread_two_arrays() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = [4, 5, 6]
        [...a, ...b]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4, 5, 6]");
}

#[test]
fn test_spread_with_leading_elements() {
    let result = eval(
        r#"
        let rest = [2, 3, 4]
        [1, ...rest]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_spread_with_trailing_elements() {
    let result = eval(
        r#"
        let head = [1, 2, 3]
        [...head, 4]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_spread_mixed_positions() {
    let result = eval(
        r#"
        let mid = [2, 3]
        [1, ...mid, 4, 5]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4, 5]");
}

#[test]
fn test_spread_empty_array() {
    let result = eval(
        r#"
        let empty = []
        [1, ...empty, 2]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2]");
}

#[test]
fn test_spread_literal_array() {
    let result = eval("[...[1, 2, 3]]");
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_spread_three_arrays() {
    let result = eval(
        r#"
        let a = [1, 2]
        let b = [3, 4]
        let c = [5, 6]
        [...a, ...b, ...c]
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4, 5, 6]");
}

#[test]
fn test_spread_does_not_mutate_original() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = [...a, 4]
        a
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_spread_non_array_error() {
    let result = eval(r#"[...42]"#);
    assert!(result.is_err(), "spreading non-array should error");
}
