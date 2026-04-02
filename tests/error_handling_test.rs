//! Tests for try/catch error handling and throw

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// throw tests

#[test]
fn test_throw_string() {
    let result = run(r#"throw "something went wrong""#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("something went wrong"));
}

#[test]
fn test_throw_propagates() {
    let source = r#"
fn risky() {
    throw "bad input"
}
risky()
"#;
    let result = run(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("bad input"));
}

// try/catch tests

#[test]
fn test_try_no_error() {
    let source = r#"
let x = 0
try {
    x = 42
} catch(e) {
    x = -1
}
x
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_catch_thrown_error() {
    let source = r#"
let result = "none"
try {
    throw "oops"
    result = "unreachable"
} catch(e) {
    result = "caught"
}
result
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "caught");
}

#[test]
fn test_catch_error_message() {
    let source = r#"
let msg = ""
try {
    throw "hello error"
} catch(e) {
    msg = e
}
msg
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "hello error");
}

#[test]
fn test_catch_runtime_error() {
    let source = r#"
let caught = false
try {
    let x = 1 / 0
} catch(e) {
    caught = true
}
caught
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_catch_undefined_variable() {
    let source = r#"
let caught = false
try {
    let y = nonexistent_var
} catch(e) {
    caught = true
}
caught
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_try_catch_in_function() {
    let source = r#"
fn safe_divide(a, b) {
    try {
        return a / b
    } catch(e) {
        return -1
    }
}
safe_divide(10, 0)
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "-1");
}

#[test]
fn test_nested_try_catch() {
    let source = r#"
let result = ""
try {
    try {
        throw "inner"
    } catch(e) {
        result = "inner caught"
        throw "outer"
    }
} catch(e) {
    result = result + " outer caught"
}
result
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "inner caught outer caught");
}

#[test]
fn test_catch_does_not_run_when_no_error() {
    let source = r#"
let x = 1
try {
    x = 2
} catch(e) {
    x = 99
}
x
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}
