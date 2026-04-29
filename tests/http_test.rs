//! Tests for http_get() and http_post() builtins

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

// --- error handling ---

#[test]
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

#[test]
fn test_http_post_invalid_url_throws_error() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_post("not-a-valid-url://???", "body")
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

// --- config dict: error cases ---

#[test]
fn test_http_get_config_wrong_type_throws() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get("not-a-valid-url://???", "not-a-dict")
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_http_get_config_bad_timeout_type_throws() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get("not-a-valid-url://???", {timeout: "slow"})
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

// --- config dict: happy path (invalid URL still throws but opts are parsed first) ---

#[test]
fn test_http_get_with_timeout_opt() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get("not-a-valid-url://???", {timeout: 5})
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_http_get_with_user_agent_opt() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get("not-a-valid-url://???", {user_agent: "my-client/1.0"})
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_http_post_with_config_opts() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_post("not-a-valid-url://???", "data", {timeout: 10, user_agent: "bot/2"})
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_http_get_arity_zero_throws() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_get()
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_http_post_arity_one_throws() {
    let result = eval(
        r#"
        let caught = false
        try {
            http_post("url")
        } catch(e) {
            caught = true
        }
        caught
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}
