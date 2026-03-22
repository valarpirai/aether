//! Test recursion with small limit

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

#[test]
fn test_recursion_with_small_limit() {
    let source = r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(5)
"#;

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut evaluator = Evaluator::new_without_stdlib();

    // Execute and check it works
    for stmt in &program.statements {
        evaluator.exec_stmt(stmt).unwrap();
    }

    println!("Test passed: countdown(5) succeeded");
}

#[test]
fn test_recursion_hits_limit() {
    let source = r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(150)
"#;

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let mut evaluator = Evaluator::new_without_stdlib();

    // Execute and expect error
    let mut got_error = false;
    for stmt in &program.statements {
        if let Err(e) = evaluator.exec_stmt(stmt) {
            let err_msg = e.to_string();
            println!("Got error: {}", err_msg);
            assert!(err_msg.contains("recursion") || err_msg.contains("stack") || err_msg.contains("overflow"),
                   "Expected recursion/stack error, got: {}", err_msg);
            got_error = true;
            break;
        }
    }

    assert!(got_error, "Expected recursion limit error but none was raised");
}
