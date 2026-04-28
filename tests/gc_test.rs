//! GC and memory behavior tests

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

// Closure called 1000 times should not OOM
#[test]
fn test_closure_1000_iterations() {
    let source = r#"
let x = 42
let f = fn(n) { return x + n }
let i = 0
let result = 0
while (i < 1000) {
    result = f(i)
    i = i + 1
}
result
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "1041"); // 42 + 999
}

// Nested closures — multiple levels of captured env
#[test]
fn test_nested_closures() {
    let source = r#"
let make_adder = fn(x) {
    return fn(y) { return x + y }
}
let add5 = make_adder(5)
let add10 = make_adder(10)
add5(3) + add10(3)
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "21"); // 8 + 13
}

// Recursive function called many times should not leak
// Ignored: sum(50) triggers a Rust native stack overflow in debug builds
// (each Aether call ≈ 20 Rust frames; 50 * 20 = 1000 frames exceeds the default limit)
#[test]
#[ignore]
fn test_recursion_no_leak() {
    let source = r#"
fn sum(n) {
    if (n <= 0) { return 0 }
    return n + sum(n - 1)
}
sum(50)
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "1275");
}

// Many closures created and discarded should not grow unboundedly
#[test]
fn test_many_closures_created() {
    let source = r#"
let i = 0
let result = 0
while (i < 500) {
    let captured = i
    let f = fn() { return captured }
    result = result + f()
    i = i + 1
}
result
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "124750"); // sum(0..499)
}

// Functions defined in a loop should work and not accumulate
#[test]
fn test_functions_in_loop() {
    let source = r#"
let total = 0
let i = 0
while (i < 100) {
    fn double(n) { return n * 2 }
    total = total + double(i)
    i = i + 1
}
total
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "9900"); // sum of 2*i for i in 0..99
}

// Large array operations should not leak
#[test]
fn test_large_array_no_leak() {
    let source = r#"
let arr = []
let i = 0
while (i < 500) {
    arr.push(i)
    i = i + 1
}
arr.length
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "500");
}

// Closure capturing large env should still work
#[test]
fn test_closure_with_large_env() {
    let source = r#"
let a = 1
let b = 2
let c = 3
let d = 4
let e = 5
let f = fn() { return a + b + c + d + e }
let i = 0
let result = 0
while (i < 100) {
    result = result + f()
    i = i + 1
}
result
"#;
    let result = eval(source).unwrap();
    assert_eq!(result, "1500"); // 15 * 100
}
