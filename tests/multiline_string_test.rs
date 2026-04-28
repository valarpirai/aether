//! Tests for triple-quoted multi-line strings

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;
    let val = evaluator
        .environment
        .get("result")
        .map_err(|e| e.to_string())?;
    Ok(format!("{}", val))
}

#[test]
fn test_triple_string_basic() {
    let src = r#"let result = """hello world""""#;
    assert_eq!(eval(src).unwrap(), "hello world");
}

#[test]
fn test_triple_string_multiline() {
    let src = "let result = \"\"\"\nline one\nline two\n\"\"\"";
    assert_eq!(eval(src).unwrap(), "line one\nline two\n");
}

#[test]
fn test_triple_string_leading_newline_stripped() {
    // The newline immediately after """ is stripped
    let src = "let result = \"\"\"\nhello\"\"\"";
    assert_eq!(eval(src).unwrap(), "hello");
}

#[test]
fn test_triple_string_no_leading_newline_kept() {
    // If content doesn't start with newline, nothing is stripped
    let src = r#"let result = """hello""""#;
    assert_eq!(eval(src).unwrap(), "hello");
}

#[test]
fn test_triple_string_preserves_inner_quotes() {
    // Single and double quotes inside are kept as-is
    let src = r#"let result = """say "hi" and 'bye'""""#;
    assert_eq!(eval(src).unwrap(), r#"say "hi" and 'bye'"#);
}

#[test]
fn test_triple_string_sql_query() {
    let src = "let result = \"\"\"\nSELECT *\nFROM users\nWHERE active = true\n\"\"\"";
    assert_eq!(
        eval(src).unwrap(),
        "SELECT *\nFROM users\nWHERE active = true\n"
    );
}

#[test]
fn test_triple_string_concatenation() {
    let src = "let a = \"\"\"foo\nbar\"\"\"\nlet result = a";
    assert_eq!(eval(src).unwrap(), "foo\nbar");
}

#[test]
fn test_triple_string_assigned_to_variable() {
    let src = "let result = \"\"\"\nhello\nworld\n\"\"\"";
    let val = eval(src).unwrap();
    assert!(val.contains("hello"));
    assert!(val.contains("world"));
}

#[test]
fn test_triple_string_len() {
    // "hi\n" is 3 chars
    let src = "let s = \"\"\"\nhi\n\"\"\"\nlet result = len(s)";
    assert_eq!(eval(src).unwrap(), "3");
}

#[test]
fn test_triple_string_empty() {
    let src = r#"let result = """""" "#;
    assert_eq!(eval(src).unwrap(), "");
}
