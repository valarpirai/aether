//! Tests for standard library math module
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

// abs() tests
#[test]
fn test_abs_positive() {
    assert_eq!(eval("abs(5)").unwrap(), "5");
    assert_eq!(eval("abs(42)").unwrap(), "42");
}

#[test]
fn test_abs_negative() {
    assert_eq!(eval("abs(-5)").unwrap(), "5");
    assert_eq!(eval("abs(-42)").unwrap(), "42");
}

#[test]
fn test_abs_zero() {
    assert_eq!(eval("abs(0)").unwrap(), "0");
}

#[test]
fn test_abs_float() {
    assert_eq!(eval("abs(-3.14)").unwrap(), "3.14");
    assert_eq!(eval("abs(2.5)").unwrap(), "2.5");
}

// min() tests
#[test]
fn test_min_two_args() {
    assert_eq!(eval("min(3, 7)").unwrap(), "3");
    assert_eq!(eval("min(10, 5)").unwrap(), "5");
}

#[test]
fn test_min_array() {
    assert_eq!(eval("min([3, 1, 4, 1, 5])").unwrap(), "1");
    assert_eq!(eval("min([10, 20, 5, 15])").unwrap(), "5");
}

#[test]
fn test_min_negative() {
    assert_eq!(eval("min(-5, -2)").unwrap(), "-5");
    assert_eq!(eval("min([-10, -5, -20])").unwrap(), "-20");
}

#[test]
fn test_min_single_element() {
    assert_eq!(eval("min([42])").unwrap(), "42");
}

// max() tests
#[test]
fn test_max_two_args() {
    assert_eq!(eval("max(3, 7)").unwrap(), "7");
    assert_eq!(eval("max(10, 5)").unwrap(), "10");
}

#[test]
fn test_max_array() {
    assert_eq!(eval("max([3, 1, 4, 1, 5])").unwrap(), "5");
    assert_eq!(eval("max([10, 20, 5, 15])").unwrap(), "20");
}

#[test]
fn test_max_negative() {
    assert_eq!(eval("max(-5, -2)").unwrap(), "-2");
    assert_eq!(eval("max([-10, -5, -20])").unwrap(), "-5");
}

#[test]
fn test_max_single_element() {
    assert_eq!(eval("max([42])").unwrap(), "42");
}

// sum() tests
#[test]
fn test_sum_positive() {
    assert_eq!(eval("sum([1, 2, 3, 4, 5])").unwrap(), "15");
}

#[test]
fn test_sum_mixed() {
    assert_eq!(eval("sum([10, -5, 3, -2])").unwrap(), "6");
}

#[test]
fn test_sum_empty() {
    assert_eq!(eval("sum([])").unwrap(), "0");
}

#[test]
fn test_sum_single() {
    assert_eq!(eval("sum([42])").unwrap(), "42");
}

// clamp() tests
#[test]
fn test_clamp_within_range() {
    assert_eq!(eval("clamp(5, 0, 10)").unwrap(), "5");
    assert_eq!(eval("clamp(7, 1, 100)").unwrap(), "7");
}

#[test]
fn test_clamp_below_min() {
    assert_eq!(eval("clamp(-5, 0, 10)").unwrap(), "0");
    assert_eq!(eval("clamp(3, 10, 20)").unwrap(), "10");
}

#[test]
fn test_clamp_above_max() {
    assert_eq!(eval("clamp(15, 0, 10)").unwrap(), "10");
    assert_eq!(eval("clamp(100, 1, 50)").unwrap(), "50");
}

#[test]
fn test_clamp_at_boundaries() {
    assert_eq!(eval("clamp(0, 0, 10)").unwrap(), "0");
    assert_eq!(eval("clamp(10, 0, 10)").unwrap(), "10");
}

// sign() tests
#[test]
fn test_sign_positive() {
    assert_eq!(eval("sign(5)").unwrap(), "1");
    assert_eq!(eval("sign(100)").unwrap(), "1");
}

#[test]
fn test_sign_negative() {
    assert_eq!(eval("sign(-5)").unwrap(), "-1");
    assert_eq!(eval("sign(-100)").unwrap(), "-1");
}

#[test]
fn test_sign_zero() {
    assert_eq!(eval("sign(0)").unwrap(), "0");
}

// Composition tests
#[test]
fn test_abs_max_composition() {
    assert_eq!(eval("max(abs(-10), abs(-5))").unwrap(), "10");
}

#[test]
fn test_sum_range() {
    assert_eq!(eval("sum(range(1, 6))").unwrap(), "15"); // 1+2+3+4+5
}

#[test]
fn test_clamp_calculation() {
    let result = eval(
        r#"
        let score = 150
        clamp(score, 0, 100)
    "#,
    );
    assert_eq!(result.unwrap(), "100");
}
