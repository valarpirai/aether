//! Tests for random() and rand_int(n) builtins

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

// random() tests

#[test]
fn test_random_returns_float() {
    let result = eval("type(random())");
    assert_eq!(result.unwrap(), "float");
}

#[test]
fn test_random_in_range() {
    let result = eval("let r = random()\nr >= 0.0 && r < 1.0");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_random_varies() {
    // Extremely unlikely two calls collide; sanity check it isn't a constant
    let result = eval("random() != random() || random() != random()");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_random_too_many_args_errors() {
    assert!(eval("random(1)").is_err());
}

// rand_int(n) tests

#[test]
fn test_rand_int_returns_int() {
    let result = eval("type(rand_int(6))");
    assert_eq!(result.unwrap(), "int");
}

#[test]
fn test_rand_int_in_range() {
    let result = eval("let r = rand_int(6)\nr >= 0 && r < 6");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_rand_int_one() {
    // rand_int(1) must always return 0
    let result = eval("rand_int(1)");
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_rand_int_zero_errors() {
    assert!(eval("rand_int(0)").is_err());
}

#[test]
fn test_rand_int_negative_errors() {
    assert!(eval("rand_int(-5)").is_err());
}

#[test]
fn test_rand_int_wrong_type_errors() {
    assert!(eval(r#"rand_int("six")"#).is_err());
}

#[test]
fn test_rand_int_no_args_errors() {
    assert!(eval("rand_int()").is_err());
}
