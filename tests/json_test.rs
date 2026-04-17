//! Tests for json_parse() and json_stringify() builtins

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

// --- json_parse ---

#[test]
fn test_json_parse_null() {
    assert_eq!(eval(r#"json_parse("null")"#).unwrap(), "null");
}

#[test]
fn test_json_parse_bool_true() {
    assert_eq!(eval(r#"json_parse("true")"#).unwrap(), "true");
}

#[test]
fn test_json_parse_bool_false() {
    assert_eq!(eval(r#"json_parse("false")"#).unwrap(), "false");
}

#[test]
fn test_json_parse_integer() {
    assert_eq!(eval(r#"json_parse("42")"#).unwrap(), "42");
}

#[test]
fn test_json_parse_negative_integer() {
    assert_eq!(eval(r#"json_parse("-7")"#).unwrap(), "-7");
}

#[test]
fn test_json_parse_float() {
    assert_eq!(eval(r#"json_parse("3.14")"#).unwrap(), "3.14");
}

#[test]
fn test_json_parse_string() {
    assert_eq!(eval(r#"json_parse("\"hello\"")"#).unwrap(), "hello");
}

#[test]
fn test_json_parse_array() {
    assert_eq!(eval(r#"json_parse("[1,2,3]")"#).unwrap(), "[1, 2, 3]");
}

#[test]
fn test_json_parse_nested_array() {
    assert_eq!(
        eval(r#"json_parse("[[1,2],[3,4]]")"#).unwrap(),
        "[[1, 2], [3, 4]]"
    );
}

#[test]
fn test_json_parse_object() {
    let result = eval(
        r#"
        let d = json_parse("{\"name\":\"Alice\",\"age\":30}")
        d["name"]
    "#,
    );
    assert_eq!(result.unwrap(), "Alice");
}

#[test]
fn test_json_parse_object_int_value() {
    let result = eval(
        r#"
        let d = json_parse("{\"x\":42}")
        d["x"]
    "#,
    );
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_json_parse_whitespace() {
    assert_eq!(eval(r#"json_parse("  42  ")"#).unwrap(), "42");
}

#[test]
fn test_json_parse_invalid_errors() {
    assert!(eval(r#"json_parse("not json")"#).is_err());
}

// --- json_stringify ---

#[test]
fn test_json_stringify_null() {
    assert_eq!(eval("json_stringify(null)").unwrap(), "null");
}

#[test]
fn test_json_stringify_bool_true() {
    assert_eq!(eval("json_stringify(true)").unwrap(), "true");
}

#[test]
fn test_json_stringify_bool_false() {
    assert_eq!(eval("json_stringify(false)").unwrap(), "false");
}

#[test]
fn test_json_stringify_integer() {
    assert_eq!(eval("json_stringify(42)").unwrap(), "42");
}

#[test]
fn test_json_stringify_float() {
    assert_eq!(eval("json_stringify(3.14)").unwrap(), "3.14");
}

#[test]
fn test_json_stringify_string() {
    assert_eq!(eval(r#"json_stringify("hello")"#).unwrap(), r#""hello""#);
}

#[test]
fn test_json_stringify_string_with_quotes() {
    assert_eq!(
        eval(r#"json_stringify("say \"hi\"")"#).unwrap(),
        r#""say \"hi\"""#
    );
}

#[test]
fn test_json_stringify_array() {
    assert_eq!(eval("json_stringify([1, 2, 3])").unwrap(), "[1,2,3]");
}

#[test]
fn test_json_stringify_nested_array() {
    assert_eq!(
        eval("json_stringify([[1,2],[3,4]])").unwrap(),
        "[[1,2],[3,4]]"
    );
}

#[test]
fn test_json_stringify_function_errors() {
    let result = eval(
        r#"
        fn f() { return 1 }
        json_stringify(f)
    "#,
    );
    assert!(result.is_err());
}

// --- round-trip ---

#[test]
fn test_json_round_trip_array() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        let s = json_stringify(arr)
        json_parse(s)
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_json_round_trip_string() {
    let result = eval(
        r#"
        let s = json_stringify("hello world")
        json_parse(s)
    "#,
    );
    assert_eq!(result.unwrap(), "hello world");
}
