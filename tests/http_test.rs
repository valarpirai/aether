//! Tests for http_get() and http_post() builtins

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
#[ignore]
fn test_http_get_returns_non_empty_string() {
    let result = eval(
        r#"
        let body = http_get("https://httpbin.org/get")
        len(body) > 0
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
#[ignore]
fn test_http_get_type_is_string() {
    let result = eval(r#"type(http_get("https://httpbin.org/get"))"#);
    assert_eq!(result.unwrap(), "string");
}

#[test]
#[ignore]
fn test_http_post_returns_non_empty_string() {
    let result = eval(
        r#"
        let body = http_post("https://httpbin.org/post", "hello")
        len(body) > 0
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
#[ignore]
fn test_http_post_type_is_string() {
    let result = eval(r#"type(http_post("https://httpbin.org/post", "hello"))"#);
    assert_eq!(result.unwrap(), "string");
}

#[test]
#[ignore]
fn test_http_get_invalid_url_throws_error() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get("not-a-valid-url://???")
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}
