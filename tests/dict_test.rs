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

// Dict method tests
#[test]
fn test_dict_keys() {
    let result = run(r#"let d = {"a": 1, "b": 2, "c": 3}  d.keys()"#);
    assert!(result.is_ok(), "Failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("a"));
    assert!(output.contains("b"));
    assert!(output.contains("c"));
}

#[test]
fn test_dict_keys_empty() {
    let result = run(r#"let d = {}  len(d.keys())"#);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_dict_values() {
    let result = run(r#"let d = {"a": 1, "b": 2}  d.values()"#);
    assert!(result.is_ok(), "Failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_dict_values_empty() {
    let result = run(r#"let d = {}  len(d.values())"#);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_dict_contains_true() {
    let result = run(r#"let d = {"name": "Alice", "age": 30}  d.contains("name")"#);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_dict_contains_false() {
    let result = run(r#"let d = {"name": "Alice"}  d.contains("age")"#);
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_dict_contains_with_int_key() {
    let result = run(r#"let d = {1: "one", 2: "two"}  d.contains(1)"#);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_dict_size() {
    let result = run(r#"let d = {"x": 1, "y": 2}  d.size"#);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_dict_size_empty() {
    let result = run(r#"let d = {}  d.size"#);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_dict_keys_iteration() {
    let source = r#"
let d = {"a": 1, "b": 2, "c": 3}
let keys = d.keys()
len(keys)
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_dict_values_iteration() {
    let source = r#"
let d = {"x": 10, "y": 20}
let vals = d.values()
len(vals)
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_dict_keys_values_together() {
    let source = r#"
let d = {"a": 1, "b": 2}
let k = d.keys()
let v = d.values()
len(k) + len(v)
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "4");
}

#[test]
fn test_dict_methods_with_nested() {
    let source = r#"
let d = {"outer": {"inner": 42}}
d.contains("outer")
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_dict_size_vs_length() {
    let source = r#"
let d = {"a": 1, "b": 2}
d.size == d.length
"#;
    let result = run(source);
    assert_eq!(result.unwrap(), "true");
}

// Error cases
#[test]
fn test_dict_keys_with_args() {
    let result = run(r#"let d = {"a": 1}  d.keys(42)"#);
    assert!(result.is_err(), "Expected error for keys() with arguments");
}

#[test]
fn test_dict_values_with_args() {
    let result = run(r#"let d = {"a": 1}  d.values(42)"#);
    assert!(result.is_err(), "Expected error for values() with arguments");
}

#[test]
fn test_dict_contains_no_args() {
    let result = run(r#"let d = {"a": 1}  d.contains()"#);
    assert!(result.is_err(), "Expected error for contains() without arguments");
}
