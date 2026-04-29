//! Tests for string interpolation

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(source: &str) -> Result<String, String> {
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

#[test]
fn test_interp_variable() {
    let result = run(r#"let name = "Aether"  "Hello ${name}""#);
    assert_eq!(result.unwrap(), "Hello Aether");
}

#[test]
fn test_interp_number() {
    let result = run(r#"let n = 42  "Value: ${n}""#);
    assert_eq!(result.unwrap(), "Value: 42");
}

#[test]
fn test_interp_expression() {
    let result = run(r#""Result: ${1 + 2 + 3}""#);
    assert_eq!(result.unwrap(), "Result: 6");
}

#[test]
fn test_interp_multiple() {
    let result = run(r#"let a = 3  let b = 4  "${a} + ${b} = ${a + b}""#);
    assert_eq!(result.unwrap(), "3 + 4 = 7");
}

#[test]
fn test_interp_no_placeholder() {
    let result = run(r#""plain string""#);
    assert_eq!(result.unwrap(), "plain string");
}

#[test]
fn test_interp_bool() {
    let result = run(r#"let flag = true  "flag is ${flag}""#);
    assert_eq!(result.unwrap(), "flag is true");
}

#[test]
fn test_interp_nested_call() {
    let source = r#"
fn double(x) { return x * 2 }
"double(5) = ${double(5)}"
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "double(5) = 10");
}

#[test]
fn test_interp_at_start() {
    let result = run(r#"let x = 99  "${x} bottles""#);
    assert_eq!(result.unwrap(), "99 bottles");
}

#[test]
fn test_interp_at_end() {
    let result = run(r#"let x = 7  "number ${x}""#);
    assert_eq!(result.unwrap(), "number 7");
}
