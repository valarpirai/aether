//! Integration tests for the V2 plugin protocol (String, Vec, HashMap types).
//!
//! These load `examples/plugins/libexample_plugin_v2.dylib`, built from
//! `example_plugin_v2/`. Rebuild it with:
//!   (cd example_plugin_v2 && cargo build --release)
//!   cp target/release/libexample_plugin_v2.dylib examples/plugins/

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

/// Helper to evaluate code and get the value of the last expression.
fn eval_code(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut eval = Evaluator::new_without_stdlib();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        eval.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = eval.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        eval.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

const PLUGIN: &str = r#"load_plugin("examples/plugins/libexample_plugin_v2.dylib")"#;

// ============================================================================
// String parameters and returns
// ============================================================================

#[test]
fn test_v2_greet_string() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.greet("Alice")
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "Hello, Alice!");
}

#[test]
fn test_v2_to_upper() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.to_upper("hello")
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "HELLO");
}

#[test]
fn test_v2_mixed_string_and_int_args() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.repeat_string("ab", 3)
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "ababab");
}

#[test]
fn test_v2_empty_string() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.to_upper("")
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "");
}

// ============================================================================
// Array parameters and returns
// ============================================================================

#[test]
fn test_v2_sort_array() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sort_array([5, 2, 8, 1, 9])
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "[1, 2, 5, 8, 9]");
}

#[test]
fn test_v2_sum_array() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sum_array([5, 2, 8, 1, 9])
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "25");
}

#[test]
fn test_v2_reverse_array() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.reverse_array([1, 2, 3])
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "[3, 2, 1]");
}

#[test]
fn test_v2_empty_array() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sum_array([])
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "0");
}

// ============================================================================
// Dict (HashMap) parameters and returns
// ============================================================================

#[test]
fn test_v2_sum_dict_values() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sum_values({{"alice": 10, "bob": 20, "carol": 30}})
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "60");
}

#[test]
fn test_v2_increment_dict_values() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        let out = p.increment_values({{"a": 1, "b": 2}})
        [out["a"], out["b"]]
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "[2, 3]");
}

#[test]
fn test_v2_dict_returns_dict_type() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        type(p.increment_values({{"a": 1}}))
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "dict");
}

#[test]
fn test_v2_empty_dict() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sum_values({{}})
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "0");
}

// ============================================================================
// Composition and control flow
// ============================================================================

#[test]
fn test_v2_composition() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.to_upper(p.greet("bob"))
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "HELLO, BOB!");
}

#[test]
fn test_v2_in_loop() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        let total = 0
        for i in [1, 2, 3] {{
            total = total + p.sum_array([i, i])
        }}
        total
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "12");
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_v2_wrong_type_string_expected() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.greet(42)
    "#
    );
    let result = eval_code(&code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Plugin error"));
}

#[test]
fn test_v2_wrong_type_array_expected() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.sum_array("not an array")
    "#
    );
    let result = eval_code(&code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Plugin error"));
}

#[test]
fn test_v2_nonexistent_function() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.does_not_exist("x")
    "#
    );
    let result = eval_code(&code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

// ============================================================================
// Result return type — Ok returns the value, Err raises a catchable error
// ============================================================================

#[test]
fn test_v2_result_ok_returns_value() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.checked_div(10, 2)
    "#
    );
    assert_eq!(eval_code(&code).unwrap(), "5");
}

#[test]
fn test_v2_result_err_raises() {
    let code = format!(
        r#"
        let p = {PLUGIN}
        p.checked_div(10, 0)
    "#
    );
    let result = eval_code(&code);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("Plugin error"), "got: {msg}");
    assert!(msg.contains("division by zero"), "got: {msg}");
}
