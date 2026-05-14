//! Tests for stdlib math additions: factorial, trunc, pi/e/tau constants,
//! degrees, radians, hypot, exp, sin, cos, tan

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

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
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
    }
    Ok("null".to_string())
}

fn eval_float(source: &str) -> f64 {
    eval(source).unwrap().parse::<f64>().unwrap()
}

// factorial()
#[test]
fn test_factorial_zero() {
    assert_eq!(eval("factorial(0)").unwrap(), "1");
}

#[test]
fn test_factorial_one() {
    assert_eq!(eval("factorial(1)").unwrap(), "1");
}

#[test]
fn test_factorial_five() {
    assert_eq!(eval("factorial(5)").unwrap(), "120");
}

#[test]
fn test_factorial_ten() {
    assert_eq!(eval("factorial(10)").unwrap(), "3628800");
}

// trunc()
#[test]
fn test_trunc_positive() {
    assert_eq!(eval("trunc(3.9)").unwrap(), "3");
}

#[test]
fn test_trunc_negative() {
    assert_eq!(eval("trunc(-3.9)").unwrap(), "-3");
}

#[test]
fn test_trunc_integer() {
    assert_eq!(eval("trunc(5.0)").unwrap(), "5");
}

#[test]
fn test_trunc_zero() {
    assert_eq!(eval("trunc(0.0)").unwrap(), "0");
}

// pi / e / tau constants
#[test]
fn test_pi_value() {
    let v = eval_float("pi");
    assert!((v - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_e_value() {
    let v = eval_float("e");
    assert!((v - std::f64::consts::E).abs() < 1e-10);
}

#[test]
fn test_tau_value() {
    let v = eval_float("tau");
    assert!((v - std::f64::consts::TAU).abs() < 1e-10);
}

// degrees() / radians()
#[test]
fn test_degrees_from_pi() {
    let v = eval_float("degrees(pi)");
    assert!((v - 180.0).abs() < 1e-8);
}

#[test]
fn test_degrees_zero() {
    let v = eval_float("degrees(0.0)");
    assert!(v.abs() < 1e-10);
}

#[test]
fn test_radians_from_180() {
    let v = eval_float("radians(180.0)");
    assert!((v - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_degrees_radians_roundtrip() {
    let v = eval_float("degrees(radians(90.0))");
    assert!((v - 90.0).abs() < 1e-8);
}

// hypot()
#[test]
fn test_hypot_3_4() {
    let v = eval_float("hypot(3.0, 4.0)");
    assert!((v - 5.0).abs() < 1e-8);
}

#[test]
fn test_hypot_zero() {
    let v = eval_float("hypot(0.0, 0.0)");
    assert!(v.abs() < 1e-10);
}

#[test]
fn test_hypot_integers() {
    let v = eval_float("hypot(5, 12)");
    assert!((v - 13.0).abs() < 1e-8);
}

// exp()
#[test]
fn test_exp_zero() {
    let v = eval_float("exp(0.0)");
    assert!((v - 1.0).abs() < 1e-8);
}

#[test]
fn test_exp_one() {
    let v = eval_float("exp(1.0)");
    assert!((v - std::f64::consts::E).abs() < 1e-6);
}

#[test]
fn test_exp_two() {
    let v = eval_float("exp(2.0)");
    assert!((v - std::f64::consts::E.powi(2)).abs() < 1e-6);
}

// sin()
#[test]
fn test_sin_zero() {
    let v = eval_float("sin(0.0)");
    assert!(v.abs() < 1e-8);
}

#[test]
fn test_sin_pi_over_2() {
    let v = eval_float("sin(pi / 2.0)");
    assert!((v - 1.0).abs() < 1e-6);
}

#[test]
fn test_sin_pi() {
    let v = eval_float("sin(pi)");
    assert!(v.abs() < 1e-6);
}

// cos()
#[test]
fn test_cos_zero() {
    let v = eval_float("cos(0.0)");
    assert!((v - 1.0).abs() < 1e-8);
}

#[test]
fn test_cos_pi() {
    let v = eval_float("cos(pi)");
    assert!((v + 1.0).abs() < 1e-6);
}

#[test]
fn test_cos_pi_over_2() {
    let v = eval_float("cos(pi / 2.0)");
    assert!(v.abs() < 1e-6);
}

// tan()
#[test]
fn test_tan_zero() {
    let v = eval_float("tan(0.0)");
    assert!(v.abs() < 1e-8);
}

#[test]
fn test_tan_pi_over_4() {
    let v = eval_float("tan(pi / 4.0)");
    assert!((v - 1.0).abs() < 1e-6);
}

#[test]
fn test_tan_pi() {
    let v = eval_float("tan(pi)");
    assert!(v.abs() < 1e-6);
}

// sin² + cos² = 1 (Pythagorean identity)
#[test]
fn test_sin_cos_identity() {
    let v = eval_float("sin(1.0) * sin(1.0) + cos(1.0) * cos(1.0)");
    assert!((v - 1.0).abs() < 1e-6);
}
