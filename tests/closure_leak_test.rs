//! Minimal test to reproduce closure memory leak

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
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

// Test 1: Function expression WITHOUT closure (should work fine)
#[test]
fn test_no_closure() {
    let source = r#"
let add = fn(a, b) { return a + b }
add(3, 5)
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "8");
}

// Test 2: Function expression WITH closure (causes leak)
#[test]
fn test_simple_closure() {
    let source = r#"
let x = 10
let f = fn(y) { return x + y }
f(5)
"#;
    println!("Testing simple closure...");
    let result = eval(source);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "15");
}

// Test 3: Multiple calls (shows memory accumulation)
#[test]
fn test_closure_multiple_calls() {
    let source = r#"
let x = 10
let f = fn(y) { return x + y }
f(1)
f(2)
f(3)
"#;
    println!("Testing multiple closure calls...");
    let result = eval(source);
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "13");
}

// Test 4: Closure in loop (extreme memory test)
#[test]
#[ignore] // Ignore by default - causes OOM
fn test_closure_in_loop() {
    let source = r#"
let x = 10
let f = fn(y) { return x + y }
let total = 0
let i = 0
while (i < 100) {
    total = total + f(i)
    i = i + 1
}
total
"#;
    let result = eval(source);
    assert!(result.is_ok());
}
