//! Tests for clock() and sleep() builtins

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

// clock() tests

#[test]
fn test_clock_returns_float() {
    let result = eval("type(clock())");
    assert_eq!(result.unwrap(), "float");
}

#[test]
fn test_clock_is_positive() {
    let result = eval("clock() > 0.0");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_clock_advances() {
    // Two successive calls should return non-decreasing values
    let result = eval(
        r#"
        let t1 = clock()
        let t2 = clock()
        t2 >= t1
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_clock_no_args() {
    assert!(eval("clock()").is_ok());
}

#[test]
fn test_clock_too_many_args_errors() {
    assert!(eval("clock(1)").is_err());
}

// sleep() tests

#[test]
fn test_sleep_returns_null() {
    let result = eval("sleep(0)");
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_sleep_float_duration() {
    let result = eval("sleep(0.0)");
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_sleep_advances_clock() {
    let result = eval(
        r#"
        let before = clock()
        sleep(0.05)
        let after = clock()
        after > before
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_sleep_wrong_type_errors() {
    assert!(eval(r#"sleep("long")"#).is_err());
}

#[test]
fn test_sleep_no_args_errors() {
    assert!(eval("sleep()").is_err());
}
