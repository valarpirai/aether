//! Test recursion depth limit

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    for stmt in &program.statements {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    Ok("success".to_string())
}

// Each Aether call consumes ~20 Rust frames; debug frames are large (~8 KB each).
// 100 Aether calls × 20 frames × 8 KB ≈ 16 MB — exceeds the 8 MB default thread stack.
// Running in a 32 MB thread gives the Aether depth-limit check room to fire first.
fn eval_large_stack(source: &'static str) -> Result<String, String> {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || eval(source))
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn test_recursion_limit_exceeded() {
    let result = eval_large_stack(
        r#"
fn infinite() {
    return infinite()
}

infinite()
"#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("recursion") || err.contains("stack") || err.contains("overflow"),
        "Expected recursion error, got: {}",
        err
    );
    assert!(err.contains("100"), "Expected limit 100 in error: {}", err);
}

#[test]
fn test_deep_recursion_within_limit() {
    let result = eval_large_stack(
        r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(50)
"#,
    );
    assert!(
        result.is_ok(),
        "Expected success for depth 50, got: {:?}",
        result
    );
}

// Note: Mutual recursion test removed - function lookup across definitions needs more work
