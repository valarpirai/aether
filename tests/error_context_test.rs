//! Tests for runtime error context: line numbers, stack traces, error objects

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

// ── e.message ──────────────────────────────────────────────────────────────

#[test]
fn test_error_message_from_throw() {
    let src = r#"
let msg = ""
try {
    throw "something went wrong"
} catch(e) {
    msg = e.message
}
msg
"#;
    let result = run(src).unwrap();
    assert_eq!(result, "something went wrong");
}

#[test]
fn test_error_message_from_runtime_error() {
    let src = r#"
let msg = ""
try {
    let x = 1 / 0
} catch(e) {
    msg = e.message
}
msg
"#;
    let result = run(src).unwrap();
    assert!(result.contains("Division by zero"), "got: {}", result);
}

#[test]
fn test_error_message_from_undefined_variable() {
    let src = r#"
let msg = ""
try {
    let x = nonexistent
} catch(e) {
    msg = e.message
}
msg
"#;
    let result = run(src).unwrap();
    assert!(result.contains("nonexistent"), "got: {}", result);
}

// ── e.stack_trace ───────────────────────────────────────────────────────────

#[test]
fn test_error_stack_trace_contains_function_name() {
    let src = r#"
fn boom() {
    throw "oops"
}
let trace = ""
try {
    boom()
} catch(e) {
    trace = e.stack_trace
}
trace
"#;
    let result = run(src).unwrap();
    assert!(result.contains("boom"), "trace should mention 'boom', got: {}", result);
}

#[test]
fn test_error_stack_trace_contains_line() {
    let src = r#"
fn boom() {
    throw "oops"
}
let trace = ""
try {
    boom()
} catch(e) {
    trace = e.stack_trace
}
trace
"#;
    let result = run(src).unwrap();
    assert!(result.contains("line"), "trace should contain 'line', got: {}", result);
}

#[test]
fn test_error_stack_trace_nested_calls() {
    let src = r#"
fn inner() {
    throw "deep"
}
fn outer() {
    inner()
}
let trace = ""
try {
    outer()
} catch(e) {
    trace = e.stack_trace
}
trace
"#;
    let result = run(src).unwrap();
    assert!(result.contains("inner"), "trace should mention 'inner', got: {}", result);
    assert!(result.contains("outer"), "trace should mention 'outer', got: {}", result);
}

// ── e as string (backward compat) ───────────────────────────────────────────

#[test]
fn test_error_display_is_message() {
    let src = r#"
let msg = ""
try {
    throw "hello error"
} catch(e) {
    msg = e
}
msg
"#;
    // e displays as its message — backward compatible with old string behaviour
    let result = run(src).unwrap();
    assert_eq!(result, "hello error");
}

#[test]
fn test_error_type_name() {
    let src = r#"
let t = ""
try {
    throw "x"
} catch(e) {
    t = type(e)
}
t
"#;
    let result = run(src).unwrap();
    assert_eq!(result, "error");
}

// ── error_line tracking ─────────────────────────────────────────────────────

#[test]
fn test_line_tracking_updates_current_line() {
    let src = r#"
fn f() {
    let a = 1
    let b = undefined_var
    let c = 3
}
let line_info = ""
try {
    f()
} catch(e) {
    line_info = e.message
}
line_info
"#;
    let result = run(src).unwrap();
    assert!(result.contains("undefined_var"), "got: {}", result);
}

// ── nested try/catch with stack ─────────────────────────────────────────────

#[test]
fn test_nested_catch_has_independent_error_objects() {
    let src = r#"
let msg1 = ""
let msg2 = ""
try {
    try {
        throw "inner error"
    } catch(e1) {
        msg1 = e1.message
        throw "outer error"
    }
} catch(e2) {
    msg2 = e2.message
}
msg1
"#;
    let result = run(src).unwrap();
    assert_eq!(result, "inner error");
}

#[test]
fn test_nested_catch_outer_message() {
    let src = r#"
let msg2 = ""
try {
    try {
        throw "inner error"
    } catch(e1) {
        throw "outer error"
    }
} catch(e2) {
    msg2 = e2.message
}
msg2
"#;
    let result = run(src).unwrap();
    assert_eq!(result, "outer error");
}
