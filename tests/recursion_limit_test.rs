//! Test recursion depth limit

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
    assert!(err.contains("recursion"), "Expected recursion error, got: {}", err);
    assert!(err.contains("1000"), "Expected limit 1000 in error: {}", err);
}

#[test]
fn test_deep_recursion_within_limit() {
    let source = r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(100)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Expected success for depth 100, got: {:?}", result);
}

#[test]
fn test_mutual_recursion_limit() {
    let source = r#"
fn even(n) {
    if (n == 0) {
        return true
    }
    return odd(n - 1)
}

fn odd(n) {
    if (n == 0) {
        return false
    }
    return even(n - 1)
}

even(2000)
"#;
    let result = eval(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("recursion"), "Expected recursion error for mutual recursion");
}
