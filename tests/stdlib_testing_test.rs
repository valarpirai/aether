//! Tests for stdlib/testing.ae - the Aether testing framework
//! TDD: Written FIRST before implementing stdlib/testing.ae

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn run(source: &str) -> Result<String, String> {
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
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// assert_eq tests

#[test]
fn test_assert_eq_passes_when_equal() {
    let result = run(r#"assert_eq(1, 1)"#);
    assert!(result.is_ok(), "assert_eq(1, 1) should pass: {:?}", result);
}

#[test]
fn test_assert_eq_passes_string() {
    let result = run(r#"assert_eq("hello", "hello")"#);
    assert!(
        result.is_ok(),
        "assert_eq strings should pass: {:?}",
        result
    );
}

#[test]
fn test_assert_eq_fails_when_not_equal() {
    let result = run(r#"assert_eq(1, 2)"#);
    assert!(result.is_err(), "assert_eq(1, 2) should throw");
    let err = result.unwrap_err();
    assert!(
        err.contains("1") && err.contains("2"),
        "Error should mention values: {}",
        err
    );
}

#[test]
fn test_assert_eq_fails_with_descriptive_message() {
    let result = run(r#"assert_eq("foo", "bar")"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("foo") || err.contains("bar"),
        "Error should include values: {}",
        err
    );
}

// assert_true / assert_false tests

#[test]
fn test_assert_true_passes() {
    let result = run(r#"assert_true(true)"#);
    assert!(
        result.is_ok(),
        "assert_true(true) should pass: {:?}",
        result
    );
}

#[test]
fn test_assert_true_fails() {
    let result = run(r#"assert_true(false)"#);
    assert!(result.is_err(), "assert_true(false) should throw");
}

#[test]
fn test_assert_false_passes() {
    let result = run(r#"assert_false(false)"#);
    assert!(
        result.is_ok(),
        "assert_false(false) should pass: {:?}",
        result
    );
}

#[test]
fn test_assert_false_fails() {
    let result = run(r#"assert_false(true)"#);
    assert!(result.is_err(), "assert_false(true) should throw");
}

// assert_null / assert_not_null tests

#[test]
fn test_assert_null_passes() {
    let result = run(r#"assert_null(null)"#);
    assert!(
        result.is_ok(),
        "assert_null(null) should pass: {:?}",
        result
    );
}

#[test]
fn test_assert_null_fails() {
    let result = run(r#"assert_null(42)"#);
    assert!(result.is_err(), "assert_null(42) should throw");
}

#[test]
fn test_assert_not_null_passes() {
    let result = run(r#"assert_not_null(42)"#);
    assert!(
        result.is_ok(),
        "assert_not_null(42) should pass: {:?}",
        result
    );
}

#[test]
fn test_assert_not_null_fails() {
    let result = run(r#"assert_not_null(null)"#);
    assert!(result.is_err(), "assert_not_null(null) should throw");
}

// expect_error tests

#[test]
fn test_expect_error_passes_when_fn_throws() {
    let source = r#"
expect_error(fn() {
    throw "intentional"
})
"#;
    let result = run(source);
    assert!(
        result.is_ok(),
        "expect_error should pass when fn throws: {:?}",
        result
    );
}

#[test]
fn test_expect_error_fails_when_fn_does_not_throw() {
    let source = r#"
expect_error(fn() {
    let x = 1 + 1
})
"#;
    let result = run(source);
    assert!(
        result.is_err(),
        "expect_error should throw when fn does not throw"
    );
}

// test() runner tests

#[test]
fn test_runner_passes_on_success() {
    let source = r#"
test("addition works", fn() {
    assert_eq(1 + 1, 2)
})
"#;
    let result = run(source);
    assert!(
        result.is_ok(),
        "test() should not throw on passing test: {:?}",
        result
    );
}

#[test]
fn test_runner_does_not_throw_on_failure() {
    let source = r#"
test("this fails", fn() {
    assert_eq(1, 2)
})
"#;
    // test() catches the error and records it — it should NOT propagate
    let result = run(source);
    assert!(
        result.is_ok(),
        "test() should catch failures, not propagate: {:?}",
        result
    );
}

#[test]
fn test_summary_with_results() {
    let source = r#"
let results = []
results.push(test("passes", fn() { assert_eq(1, 1) }))
results.push(test("fails", fn() { assert_eq(1, 2) }))
test_summary(results)
"#;
    let result = run(source);
    assert!(result.is_ok(), "test_summary() should work: {:?}", result);
}

#[test]
fn test_returns_result_dict() {
    let source = r#"
let r = test("my test", fn() { assert_eq(2, 2) })
r["passed"]
"#;
    let result = run(source);
    assert!(
        result.is_ok(),
        "test() should return a result dict: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_returns_failure_dict() {
    let source = r#"
let r = test("failing", fn() { assert_eq(1, 2) })
r["passed"]
"#;
    let result = run(source);
    assert!(
        result.is_ok(),
        "test() should return failure dict without throwing: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "false");
}
