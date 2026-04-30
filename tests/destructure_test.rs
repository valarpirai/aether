//! Tests for destructuring let statements

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
            return Ok(format!(
                "{}",
                evaluator.eval_expr(expr).map_err(|e| e.to_string())?
            ));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }
    Ok(String::new())
}

// --- Array destructuring ---

#[test]
fn test_array_first_element() {
    assert_eq!(eval("let [a, b, c] = [1, 2, 3]  a").unwrap(), "1");
}

#[test]
fn test_array_middle_element() {
    assert_eq!(eval("let [a, b, c] = [10, 20, 30]  b").unwrap(), "20");
}

#[test]
fn test_array_shorter_than_pattern() {
    assert_eq!(eval("let [a, b] = [42]  b").unwrap(), "null");
}

#[test]
fn test_array_default_value_used() {
    assert_eq!(eval("let [a, b = 99] = [1]  b").unwrap(), "99");
}

#[test]
fn test_array_default_value_not_used() {
    assert_eq!(eval("let [a, b = 99] = [1, 2]  b").unwrap(), "2");
}

#[test]
fn test_array_rest_head() {
    assert_eq!(
        eval("let [head, ...tail] = [1, 2, 3, 4]  head").unwrap(),
        "1"
    );
}

#[test]
fn test_array_rest_tail_length() {
    assert_eq!(
        eval("let [head, ...tail] = [1, 2, 3, 4]  len(tail)").unwrap(),
        "3"
    );
}

#[test]
fn test_array_rest_empty() {
    assert_eq!(eval("let [a, ...rest] = [1]  len(rest)").unwrap(), "0");
}

#[test]
fn test_array_underscore_skips() {
    assert_eq!(eval("let [_, b] = [10, 20]  b").unwrap(), "20");
}

#[test]
fn test_array_type_error() {
    assert!(eval("let [a] = 42").is_err());
}

// --- Dict destructuring ---

#[test]
fn test_dict_basic() {
    assert_eq!(eval(r#"let {x, y} = {"x": 1, "y": 2}  x"#).unwrap(), "1");
}

#[test]
fn test_dict_both_keys() {
    assert_eq!(eval(r#"let {x, y} = {"x": 10, "y": 20}  y"#).unwrap(), "20");
}

#[test]
fn test_dict_missing_key_is_null() {
    assert_eq!(eval(r#"let {x, z} = {"x": 1}  z"#).unwrap(), "null");
}

#[test]
fn test_dict_rename() {
    assert_eq!(
        eval(r#"let {port: p} = {"port": 5432}  p"#).unwrap(),
        "5432"
    );
}

#[test]
fn test_dict_default_used() {
    assert_eq!(eval(r#"let {x = 0} = {}  x"#).unwrap(), "0");
}

#[test]
fn test_dict_default_not_used() {
    assert_eq!(eval(r#"let {x = 0} = {"x": 7}  x"#).unwrap(), "7");
}

#[test]
fn test_dict_rename_with_default() {
    assert_eq!(eval(r#"let {timeout: t = 30} = {}  t"#).unwrap(), "30");
}

#[test]
fn test_dict_type_error() {
    assert!(eval("let {x} = 42").is_err());
}

// --- Inside functions ---

#[test]
fn test_destructure_in_function() {
    assert_eq!(
        eval(
            r#"
            fn swap(pair) {
                let [a, b] = pair
                return [b, a]
            }
            let result = swap([1, 2])
            result[0]
        "#
        )
        .unwrap(),
        "2"
    );
}

#[test]
fn test_dict_destructure_in_function() {
    assert_eq!(
        eval(
            r#"
            fn connect(cfg) {
                let {host, port: p = 5432} = cfg
                return host
            }
            connect({"host": "localhost"})
        "#
        )
        .unwrap(),
        "localhost"
    );
}
