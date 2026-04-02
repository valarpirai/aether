//! Tests for dictionary literals and access

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

#[test]
fn test_empty_dict() {
    let result = run("let d = {}  d");
    assert_eq!(result.unwrap(), "{}");
}

#[test]
fn test_dict_string_keys() {
    let result = run(r#"let d = {"a": 1, "b": 2}  d"#);
    assert_eq!(result.unwrap(), "{a: 1, b: 2}");
}

#[test]
fn test_dict_index_access() {
    let result = run(r#"let d = {"name": "Aether", "version": 1}  d["name"]"#);
    assert_eq!(result.unwrap(), "Aether");
}

#[test]
fn test_dict_member_access() {
    let result = run(r#"let d = {"name": "Aether"}  d.name"#);
    assert_eq!(result.unwrap(), "Aether");
}

#[test]
fn test_dict_int_value() {
    let result = run(r#"let d = {"x": 42}  d["x"]"#);
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_dict_length() {
    let result = run(r#"let d = {"a": 1, "b": 2, "c": 3}  d.length"#);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_dict_key_not_found() {
    let result = run(r#"let d = {"a": 1}  d["z"]"#);
    assert!(result.is_err());
}

#[test]
fn test_dict_nested() {
    let result = run(r#"let d = {"inner": {"val": 99}}  d["inner"]["val"]"#);
    assert_eq!(result.unwrap(), "99");
}

#[test]
fn test_dict_in_function() {
    let source = r#"
fn make_person(name, age) {
    return {"name": name, "age": age}
}
let p = make_person("Alice", 30)
p.name
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "Alice");
}

#[test]
fn test_dict_variable_value() {
    let result = run(r#"let x = 10  let d = {"val": x}  d["val"]"#);
    assert_eq!(result.unwrap(), "10");
}
