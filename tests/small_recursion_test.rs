//! Test recursion with small limit

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval_large_stack(source: &'static str) -> Result<(), String> {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
            let mut parser = Parser::new(tokens);
            let program = parser.parse().map_err(|e| e.to_string())?;
            let mut evaluator = Evaluator::new_without_stdlib();
            for stmt in &program.statements {
                evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
            }
            Ok(())
        })
        .unwrap()
        .join()
        .unwrap()
}

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
    let result = eval_large_stack(
        r#"
fn countdown(n) {
    if (n <= 0) {
        return 0
    }
    return countdown(n - 1)
}

countdown(150)
"#,
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("recursion") || err.contains("stack") || err.contains("overflow"),
        "Expected recursion/stack error, got: {}",
        err
    );
}
