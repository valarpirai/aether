//! Tests for Phase 2: I/O thread pool async builtins

use aether::interpreter::value::{PromiseState, Value};
use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn run_with_pool(source: &str, workers: usize) -> Result<Value, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_with_pool(workers);

    let stmts = &program.statements;
    if stmts.is_empty() {
        return Ok(Value::Null);
    }
    for stmt in &stmts[..stmts.len() - 1] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    let last = stmts.last().unwrap();
    if let aether::parser::ast::Stmt::Expr(expr) = last {
        return evaluator.eval_expr(expr).map_err(|e| e.to_string());
    }
    evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

fn run_with_pool_get(source: &str, workers: usize, var: &str) -> Result<Value, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_with_pool(workers);
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;
    evaluator.environment.get(var).map_err(|e| e.to_string())
}

#[test]
fn test_io_pool_creates_with_workers() {
    // Should not panic — pool creation is the test
    let _evaluator = Evaluator::new_with_pool(2);
}

#[test]
fn test_set_workers_builtin() {
    let src = r#"
set_workers(2)
"#;
    // Should run without error
    assert!(run_with_pool(src, 1).is_ok());
}

#[test]
fn test_sleep_with_pool_returns_promise() {
    let src = r#"
set_workers(2)
let p = sleep(0.001)
"#;
    let p = run_with_pool_get(src, 2, "p").unwrap();
    assert!(
        matches!(p, Value::Promise(_)),
        "Expected Promise for sleep() with pool active, got {:?}",
        p
    );
}

#[test]
fn test_sleep_with_pool_is_pending_before_await() {
    let src = r#"
let p = sleep(0.001)
"#;
    let p = run_with_pool_get(src, 2, "p").unwrap();
    if let Value::Promise(state) = p {
        assert!(
            matches!(&*state.borrow(), PromiseState::IoWaiting(_)),
            "Expected IoWaiting state"
        );
    } else {
        panic!("Expected Promise");
    }
}

#[test]
fn test_await_sleep_with_pool() {
    let src = "await sleep(0.001)";
    let result = run_with_pool(src, 2).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_sleep_without_pool_is_blocking() {
    // Without pool, sleep() should execute synchronously and return Null directly
    let src = "sleep(0.001)";
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut evaluator = Evaluator::new_without_stdlib();
    // register only builtins (no stdlib)
    let last = program.statements.last().unwrap();
    if let aether::parser::ast::Stmt::Expr(expr) = last {
        let result = evaluator.eval_expr(expr).unwrap();
        assert_eq!(result, Value::Null);
    }
}

#[test]
fn test_read_file_with_pool_returns_promise() {
    let src = r#"
let p = read_file("Cargo.toml")
"#;
    let p = run_with_pool_get(src, 2, "p").unwrap();
    assert!(
        matches!(p, Value::Promise(_)),
        "Expected Promise for read_file() with pool active"
    );
}

#[test]
fn test_await_read_file_with_pool() {
    let src = r#"await read_file("Cargo.toml")"#;
    let result = run_with_pool(src, 2).unwrap();
    // Should be a string containing package info
    if let Value::String(s) = result {
        assert!(
            s.contains("aether"),
            "Expected file content to contain 'aether'"
        );
    } else {
        panic!("Expected String result from read_file");
    }
}

#[test]
fn test_promise_all_two_sleeps() {
    let src = r#"
let p1 = sleep(0.001)
let p2 = sleep(0.001)
let results = await Promise.all([p1, p2])
results
"#;
    let results = run_with_pool(src, 2).unwrap();
    if let Value::Array(arr) = results {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::Null);
        assert_eq!(arr[1], Value::Null);
    } else {
        panic!("Expected array from Promise.all, got {:?}", results);
    }
}

#[test]
fn test_promise_all_empty_array() {
    let src = r#"await Promise.all([])"#;
    let results = run_with_pool(src, 2).unwrap();
    if let Value::Array(arr) = results {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected empty array");
    }
}

#[test]
fn test_promise_all_with_regular_promises() {
    // Promise.all should also work with Pending (non-I/O) promises
    let src = r#"
async fn double(x) { return x * 2 }
let p1 = double(5)
let p2 = double(10)
let results = await Promise.all([p1, p2])
results
"#;
    let results = run_with_pool(src, 2).unwrap();
    if let Value::Array(arr) = results {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::Int(10));
        assert_eq!(arr[1], Value::Int(20));
    } else {
        panic!("Expected array from Promise.all, got {:?}", results);
    }
}

#[test]
fn test_set_workers_replaces_pool() {
    // After set_workers(4), subsequent I/O calls should use new pool
    let src = r#"
set_workers(4)
let p = sleep(0.001)
await p
"#;
    let result = run_with_pool(src, 1).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_promise_display_io_waiting() {
    let src = r#"
let p = sleep(0.001)
"#;
    let p = run_with_pool_get(src, 2, "p").unwrap();
    assert_eq!(format!("{}", p), "<promise:pending>");
}

#[test]
fn test_promise_display_resolved_after_io_await() {
    let src = r#"
async fn get_num() { return 42 }
let p = get_num()
let _ = await p
"#;
    let p = run_with_pool_get(src, 2, "p").unwrap();
    assert_eq!(format!("{}", p), "<promise:42>");
}
