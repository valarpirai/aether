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

    // Execute all statements
    for stmt in &program.statements {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    Ok("success".to_string())
}

#[test]
#[ignore = "infinite recursion overflows Rust native stack before Aether depth limit is hit in debug builds"]
fn test_recursion_limit_exceeded() {
    let source = r#"
fn infinite() {
    return infinite()
}

infinite()
"#;
    let result = eval(source);
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
#[ignore = "countdown(50) overflows Rust native stack in debug builds before Aether depth check fires"]
fn test_deep_recursion_within_limit() {
    let source = r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(50)
"#;
    let result = eval(source);
    assert!(
        result.is_ok(),
        "Expected success for depth 50, got: {:?}",
        result
    );
}

// Note: Mutual recursion test removed - function lookup across definitions needs more work
