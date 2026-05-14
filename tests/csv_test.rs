//! Tests for csv_parse() and csv_stringify()

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

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
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }
    Ok("null".to_string())
}

// --- csv_parse ---

#[test]
fn test_csv_parse_single_row() {
    assert_eq!(eval(r#"csv_parse("a,b,c")"#).unwrap(), "[[a, b, c]]");
}

#[test]
fn test_csv_parse_multiple_rows() {
    assert_eq!(
        eval("csv_parse(\"a,b\\n1,2\")").unwrap(),
        "[[a, b], [1, 2]]"
    );
}

#[test]
fn test_csv_parse_quoted_field_with_comma() {
    assert_eq!(
        eval(r#"csv_parse("\"hello, world\",b")"#).unwrap(),
        "[[hello, world, b]]"
    );
}

#[test]
fn test_csv_parse_escaped_quote() {
    assert_eq!(
        eval(r#"csv_parse("\"say \"\"hi\"\"\",b")"#).unwrap(),
        r#"[[say "hi", b]]"#
    );
}

#[test]
fn test_csv_parse_empty_fields() {
    assert_eq!(eval(r#"csv_parse("a,,c")"#).unwrap(), "[[a, , c]]");
}

#[test]
fn test_csv_parse_crlf() {
    assert_eq!(
        eval("csv_parse(\"a,b\\r\\n1,2\")").unwrap(),
        "[[a, b], [1, 2]]"
    );
}

#[test]
fn test_csv_parse_custom_delimiter() {
    assert_eq!(eval(r#"csv_parse("a;b;c", ";")"#).unwrap(), "[[a, b, c]]");
}

#[test]
fn test_csv_parse_returns_array_of_arrays() {
    assert_eq!(eval(r#"type(csv_parse("a,b"))"#).unwrap(), "array");
}

#[test]
fn test_csv_parse_row_count() {
    assert_eq!(eval("csv_parse(\"a\\nb\\nc\").length").unwrap(), "3");
}

#[test]
fn test_csv_parse_field_access() {
    assert_eq!(
        eval(r#"csv_parse("name,age\nAlice,30")[1][0]"#).unwrap(),
        "Alice"
    );
}

#[test]
fn test_csv_parse_type_error() {
    assert!(eval("csv_parse(42)").is_err());
}

// --- csv_stringify ---

#[test]
fn test_csv_stringify_single_row() {
    assert_eq!(
        eval(r#"csv_stringify([["a", "b", "c"]])"#).unwrap(),
        "a,b,c"
    );
}

#[test]
fn test_csv_stringify_multiple_rows() {
    assert_eq!(
        eval(r#"csv_stringify([["a", "b"], ["1", "2"]])"#).unwrap(),
        "a,b\n1,2"
    );
}

#[test]
fn test_csv_stringify_quotes_field_with_comma() {
    assert_eq!(
        eval(r#"csv_stringify([["hello, world", "b"]])"#).unwrap(),
        r#""hello, world",b"#
    );
}

#[test]
fn test_csv_stringify_escapes_quotes() {
    assert_eq!(
        eval(r#"csv_stringify([["say \"hi\"", "b"]])"#).unwrap(),
        r#""say ""hi""",b"#
    );
}

#[test]
fn test_csv_stringify_int_fields() {
    assert_eq!(
        eval(r#"csv_stringify([["name", "age"], ["Alice", 30]])"#).unwrap(),
        "name,age\nAlice,30"
    );
}

#[test]
fn test_csv_stringify_custom_delimiter() {
    assert_eq!(
        eval(r#"csv_stringify([["a", "b", "c"]], ";")"#).unwrap(),
        "a;b;c"
    );
}

#[test]
fn test_csv_stringify_type_error() {
    assert!(eval("csv_stringify(\"not an array\")").is_err());
}

// --- roundtrip ---

#[test]
fn test_csv_roundtrip() {
    let result = eval(
        r#"
        let data = [["name", "age"], ["Alice", "30"], ["Bob", "25"]]
        let text = csv_stringify(data)
        let parsed = csv_parse(text)
        parsed.equals(data)
        "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_csv_roundtrip_with_commas_in_fields() {
    let result = eval(
        r#"
        let data = [["city", "note"], ["New York", "big, busy"]]
        let text = csv_stringify(data)
        let parsed = csv_parse(text)
        parsed[1][1]
        "#,
    )
    .unwrap();
    assert_eq!(result, "big, busy");
}
