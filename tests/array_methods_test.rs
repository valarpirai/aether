//! Tests for array methods
//! Written FIRST following TDD red-green-refactor

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

#[test]
fn test_array_push() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.push(4)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_array_push_returns_null() {
    let result = eval(
        r#"
        let arr = [1, 2]
        arr.push(3)
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_array_push_multiple() {
    let result = eval(
        r#"
        let arr = []
        arr.push(1)
        arr.push(2)
        arr.push(3)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_pop() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.pop()
    "#,
    );
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_array_pop_modifies_array() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.pop()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2]");
}

#[test]
fn test_array_pop_empty_array() {
    let result = eval(
        r#"
        let arr = []
        arr.pop()
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_array_push_pop_combo() {
    let result = eval(
        r#"
        let arr = [1, 2]
        arr.push(3)
        arr.push(4)
        arr.pop()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_method_chaining() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        let last = arr.pop()
        arr.push(last + 10)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 13]");
}
