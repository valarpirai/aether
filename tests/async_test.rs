//! Tests for async/await language feature (Phase 1: Promise-based)

use aether_lang::interpreter::value::{PromiseState, Value};
use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(source: &str) -> Result<Value, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    let stmts = &program.statements;
    if stmts.is_empty() {
        return Ok(Value::Null);
    }
    for stmt in &stmts[..stmts.len() - 1] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    let last = stmts.last().unwrap();
    if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
        return evaluator.eval_expr(expr).map_err(|e| e.to_string());
    }
    evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

fn run_get(source: &str, var: &str) -> Result<Value, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;
    evaluator.environment.get(var).map_err(|e| e.to_string())
}

#[test]
fn test_async_fn_returns_promise() {
    let src = r#"
async fn get_val() { return 42 }
let p = get_val()
"#;
    let p = run_get(src, "p").unwrap();
    assert!(
        matches!(p, Value::Promise(_)),
        "Expected Promise, got {:?}",
        p
    );
}

#[test]
fn test_promise_is_pending_before_await() {
    let src = r#"
async fn get_val() { return 42 }
let p = get_val()
"#;
    let p = run_get(src, "p").unwrap();
    if let Value::Promise(state) = p {
        assert!(matches!(&*state.borrow(), PromiseState::Pending { .. }));
    } else {
        panic!("Expected Promise");
    }
}

#[test]
fn test_await_resolves_to_return_value() {
    let src = r#"
async fn get_val() { return 42 }
await get_val()
"#;
    assert_eq!(run(src).unwrap(), Value::Int(42));
}

#[test]
fn test_await_non_promise_is_identity_int() {
    assert_eq!(run("await 42").unwrap(), Value::Int(42));
}

#[test]
fn test_await_non_promise_is_identity_string() {
    assert_eq!(run(r#"await "hello""#).unwrap(), Value::string("hello"));
}

#[test]
fn test_await_non_promise_is_identity_null() {
    assert_eq!(run("await null").unwrap(), Value::Null);
}

#[test]
fn test_async_fn_with_args() {
    let src = r#"
async fn double(x) { return x * 2 }
await double(21)
"#;
    assert_eq!(run(src).unwrap(), Value::Int(42));
}

#[test]
fn test_sequential_async_await() {
    let src = r#"
async fn double(x) { return x * 2 }
async fn quadruple(x) {
    let a = await double(x)
    return await double(a)
}
await quadruple(5)
"#;
    assert_eq!(run(src).unwrap(), Value::Int(20));
}

#[test]
fn test_async_function_expr() {
    let src = r#"
let f = async fn(x) { return x + 1 }
await f(10)
"#;
    assert_eq!(run(src).unwrap(), Value::Int(11));
}

#[test]
fn test_await_promise_twice_returns_cached() {
    let src = r#"
async fn get_val() { return 42 }
let p = get_val()
let a = await p
let b = await p
b
"#;
    assert_eq!(run(src).unwrap(), Value::Int(42));
}

#[test]
fn test_async_type_name_function() {
    let src = r#"
async fn f() { return 1 }
type(f)
"#;
    assert_eq!(run(src).unwrap(), Value::string("async_function"));
}

#[test]
fn test_async_type_name_promise() {
    // type() is a builtin so we need stdlib — use a different check
    let src2 = r#"
async fn f() { return 1 }
let p = f()
"#;
    let p = run_get(src2, "p").unwrap();
    assert_eq!(p.type_name(), "promise");
}

#[test]
fn test_async_function_optional_params() {
    let src = r#"
async fn greet(name) {
    if (name == null) { return "Hello" }
    return "Hello " + name
}
await greet()
"#;
    assert_eq!(run(src).unwrap(), Value::string("Hello"));
}

#[test]
fn test_await_in_regular_fn() {
    let src = r#"
async fn get_num() { return 10 }
fn double_async_result() {
    let n = await get_num()
    return n * 2
}
double_async_result()
"#;
    assert_eq!(run(src).unwrap(), Value::Int(20));
}

#[test]
fn test_nested_async_functions() {
    let src = r#"
async fn add(a, b) { return a + b }
async fn sum_three(a, b, c) {
    let ab = await add(a, b)
    return await add(ab, c)
}
await sum_three(1, 2, 3)
"#;
    assert_eq!(run(src).unwrap(), Value::Int(6));
}

#[test]
fn test_async_fn_with_string_return() {
    let src = r#"
async fn greet(name) { return "Hello " + name }
await greet("Aether")
"#;
    assert_eq!(run(src).unwrap(), Value::string("Hello Aether"));
}

#[test]
fn test_async_args_evaluated_eagerly() {
    let src = r#"
async fn f(x) { return x }
f(undefined_var)
"#;
    assert!(run(src).is_err(), "Expected error for undefined_var");
}

#[test]
fn test_promise_resolved_state_after_await() {
    let src = r#"
async fn get_val() { return 99 }
let p = get_val()
let _ = await p
"#;
    let p = run_get(src, "p").unwrap();
    if let Value::Promise(state) = p {
        assert!(
            matches!(&*state.borrow(), PromiseState::Resolved(_)),
            "Promise should be Resolved after await"
        );
    } else {
        panic!("Expected Promise");
    }
}

#[test]
fn test_async_fn_display() {
    let src = r#"
async fn my_fn(a, b) { return a + b }
"#;
    let f = run_get(src, "my_fn").unwrap();
    assert_eq!(format!("{}", f), "<async fn(2)>");
}

#[test]
fn test_promise_pending_display() {
    let src = r#"
async fn f() { return 1 }
let p = f()
"#;
    let p = run_get(src, "p").unwrap();
    assert_eq!(format!("{}", p), "<promise:pending>");
}

#[test]
fn test_promise_resolved_display() {
    let src = r#"
async fn f() { return 42 }
let p = f()
let _ = await p
"#;
    let p = run_get(src, "p").unwrap();
    assert_eq!(format!("{}", p), "<promise:42>");
}

#[test]
fn test_promise_all_parallel() {
    // Functional: Promise.all with two I/O-backed sleeps returns both results.
    let src = r#"
async fn do_sleep(n) { sleep(n) }
let p1 = do_sleep(0.01)
let p2 = do_sleep(0.01)
Promise.all([p1, p2])
"#;
    let mut scanner = aether_lang::lexer::Scanner::new(src);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = aether_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut evaluator = aether_lang::interpreter::Evaluator::new_with_pool(2);
    for stmt in &program.statements[..program.statements.len() - 1] {
        evaluator.exec_stmt(stmt).unwrap();
    }
    let last = program.statements.last().unwrap();
    let result = if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
        evaluator.eval_expr(expr).unwrap()
    } else {
        panic!("expected expr");
    };
    match result {
        aether_lang::interpreter::value::Value::Array(arr) => {
            assert_eq!(arr.len(), 2, "Promise.all should return 2 results");
        }
        other => panic!("Expected array, got {:?}", other),
    }
}

#[test]
fn test_promise_all_is_parallel_not_sequential() {
    // Timing proof: two 50ms sleeps should complete in ~50ms (parallel),
    // not ~100ms (sequential).
    let src = r#"
async fn do_sleep(n) { sleep(n) }
let p1 = do_sleep(0.05)
let p2 = do_sleep(0.05)
Promise.all([p1, p2])
"#;
    let mut scanner = aether_lang::lexer::Scanner::new(src);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = aether_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut evaluator = aether_lang::interpreter::Evaluator::new_with_pool(2);
    for stmt in &program.statements[..program.statements.len() - 1] {
        evaluator.exec_stmt(stmt).unwrap();
    }
    let last = program.statements.last().unwrap();
    let start = std::time::Instant::now();
    if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
        evaluator.eval_expr(expr).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 90,
        "Promise.all with 2×50ms sleeps should complete in <90ms (parallel), took {}ms",
        elapsed.as_millis()
    );
}

// ---- Promise.race tests ----

fn run_with_pool(source: &str, workers: usize) -> Result<Value, String> {
    let mut scanner = aether_lang::lexer::Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = aether_lang::parser::Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = aether_lang::interpreter::Evaluator::new_with_pool(workers);
    let stmts = &program.statements;
    for stmt in &stmts[..stmts.len() - 1] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    let last = stmts.last().unwrap();
    if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
        return evaluator.eval_expr(expr).map_err(|e| e.to_string());
    }
    evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

#[test]
fn test_promise_race_returns_first_resolved() {
    // p1 sleeps 10ms, p2 sleeps 100ms — race should return p1's result (null from sleep).
    let src = r#"
async fn do_sleep(n) { sleep(n) }
let p1 = do_sleep(0.01)
let p2 = do_sleep(0.1)
Promise.race([p1, p2])
"#;
    let result = run_with_pool(src, 2).unwrap();
    assert_eq!(result, Value::Null, "sleep resolves to null");
}

#[test]
fn test_promise_race_already_resolved() {
    // If a promise is already resolved, race returns it immediately.
    let src = r#"
async fn fast() { return 42 }
let p1 = fast()
let _ = await p1
async fn slow() { sleep(1.0) }
let p2 = slow()
Promise.race([p1, p2])
"#;
    let result = run_with_pool(src, 2).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_promise_race_is_faster_than_slowest() {
    // race on [50ms, 200ms] should complete in ~50ms, not ~200ms.
    let src = r#"
async fn do_sleep(n) { sleep(n) }
let p1 = do_sleep(0.05)
let p2 = do_sleep(0.2)
Promise.race([p1, p2])
"#;
    let start = std::time::Instant::now();
    run_with_pool(src, 2).unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 120,
        "Promise.race should complete in ~50ms not ~200ms, took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_promise_race_empty_array_errors() {
    let src = r#"Promise.race([])"#;
    assert!(run_with_pool(src, 1).is_err());
}
