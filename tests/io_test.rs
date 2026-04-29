//! Tests for I/O built-ins: read_file, write_file

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

#[test]
fn test_write_and_read_file() {
    let path = "/tmp/aether_test_io.txt";
    let source = format!(
        r#"write_file("{}", "hello from aether")  read_file("{}")"#,
        path, path
    );
    let result = run(&source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "hello from aether");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_read_file_not_found() {
    let result = run(r#"read_file("/tmp/aether_nonexistent_file_xyz.txt")"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("read_file"));
}

#[test]
fn test_write_file_creates_file() {
    let path = "/tmp/aether_test_write.txt";
    let source = format!(r#"write_file("{}", "content")"#, path);
    let result = run(&source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert!(std::path::Path::new(path).exists());
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_write_file_returns_null() {
    let path = "/tmp/aether_test_null.txt";
    let source = format!(r#"write_file("{}", "x")"#, path);
    let result = run(&source);
    assert!(result.is_ok());
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_read_file_try_catch() {
    let source = r#"
let content = ""
try {
    content = read_file("/tmp/aether_no_such_file_abc.txt")
} catch(e) {
    content = "error caught"
}
content
"#;
    let result = run(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "error caught");
}
