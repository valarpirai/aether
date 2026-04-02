//! Tests for standard library functionality
//! TDD: Written FIRST before implementing stdlib loader

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

// Core stdlib tests
#[test]
fn test_range_single_arg() {
    assert_eq!(eval("range(5)").unwrap(), "[0, 1, 2, 3, 4]");
    assert_eq!(eval("range(3)").unwrap(), "[0, 1, 2]");
    assert_eq!(eval("range(0)").unwrap(), "[]");
}

#[test]
fn test_range_two_args() {
    assert_eq!(eval("range(2, 7)").unwrap(), "[2, 3, 4, 5, 6]");
    assert_eq!(eval("range(5, 10)").unwrap(), "[5, 6, 7, 8, 9]");
    assert_eq!(eval("range(0, 3)").unwrap(), "[0, 1, 2]");
}

#[test]
fn test_range_empty() {
    assert_eq!(eval("range(0, 0)").unwrap(), "[]");
    assert_eq!(eval("range(5, 5)").unwrap(), "[]");
}

#[test]
fn test_range_in_loop() {
    let result = eval(
        r#"
        let sum = 0
        for i in range(5) {
            sum = sum + i
        }
        sum
    "#,
    );
    assert_eq!(result.unwrap(), "10"); // 0 + 1 + 2 + 3 + 4
}

#[test]
fn test_enumerate() {
    let result = eval(
        r#"
        let arr = ["a", "b", "c"]
        enumerate(arr)
    "#,
    );
    assert_eq!(result.unwrap(), "[[0, a], [1, b], [2, c]]");
}

#[test]
fn test_enumerate_empty() {
    assert_eq!(eval("enumerate([])").unwrap(), "[]");
}

#[test]
fn test_enumerate_with_numbers() {
    let result = eval(
        r#"
        let nums = [10, 20, 30]
        enumerate(nums)
    "#,
    );
    assert_eq!(result.unwrap(), "[[0, 10], [1, 20], [2, 30]]");
}

#[test]
fn test_range_and_map_pattern() {
    let result = eval(
        r#"
        let doubled = []
        for i in range(3) {
            doubled.push(i * 2)
        }
        doubled
    "#,
    );
    assert_eq!(result.unwrap(), "[0, 2, 4]");
}

#[test]
fn test_stdlib_available_immediately() {
    // Test that stdlib functions are available without explicit import
    assert_eq!(eval("range(3)").unwrap(), "[0, 1, 2]");
}
