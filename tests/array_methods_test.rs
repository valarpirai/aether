//! Tests for array methods
//! Written FIRST following TDD red-green-refactor

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut evaluator = Evaluator::new();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
    }

    Ok("null".to_string())
}

#[test]
fn test_array_push() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.push(4)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_array_push_returns_null() {
    let result = eval(
        r#"
        let arr = [1, 2]
        arr.push(3)
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_array_push_multiple() {
    let result = eval(
        r#"
        let arr = []
        arr.push(1)
        arr.push(2)
        arr.push(3)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_pop() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.pop()
    "#,
    );
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_array_pop_modifies_array() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.pop()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2]");
}

#[test]
fn test_array_pop_empty_array() {
    let result = eval(
        r#"
        let arr = []
        arr.pop()
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_array_push_pop_combo() {
    let result = eval(
        r#"
        let arr = [1, 2]
        arr.push(3)
        arr.push(4)
        arr.pop()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_method_chaining() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        let last = arr.pop()
        arr.push(last + 10)
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 13]");
}

#[test]
fn test_array_contains_true() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.contains(2)
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_array_contains_false() {
    let result = eval(
        r#"
        let arr = [1, 2, 3]
        arr.contains(99)
    "#,
    );
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_array_contains_string() {
    let result = eval(
        r#"
        let arr = ["a", "b", "c"]
        arr.contains("b")
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_array_contains_empty() {
    let result = eval(
        r#"
        let arr = []
        arr.contains(1)
    "#,
    );
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_array_sort_ints() {
    let result = eval(
        r#"
        let arr = [3, 1, 4, 1, 5, 9, 2, 6]
        arr.sort()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 1, 2, 3, 4, 5, 6, 9]");
}

#[test]
fn test_array_sort_strings() {
    let result = eval(
        r#"
        let arr = ["banana", "apple", "cherry"]
        arr.sort()
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[apple, banana, cherry]");
}

#[test]
fn test_array_sort_returns_null() {
    let result = eval(
        r#"
        let arr = [3, 1, 2]
        arr.sort()
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_array_sort_with_comparator_ascending() {
    let result = eval(
        r#"
        let arr = [3, 1, 4, 1, 5]
        arr.sort(fn(a, b) { return a - b })
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 1, 3, 4, 5]");
}

#[test]
fn test_array_sort_with_comparator_descending() {
    let result = eval(
        r#"
        let arr = [3, 1, 4, 1, 5]
        arr.sort(fn(a, b) { return b - a })
        arr
    "#,
    );
    assert_eq!(result.unwrap(), "[5, 4, 3, 1, 1]");
}

#[test]
fn test_array_concat_basic() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = [4, 5, 6]
        a.concat(b)
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3, 4, 5, 6]");
}

#[test]
fn test_array_concat_does_not_mutate() {
    let result = eval(
        r#"
        let a = [1, 2]
        let b = [3, 4]
        a.concat(b)
        a
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2]");
}

#[test]
fn test_array_concat_empty_left() {
    let result = eval(
        r#"
        let a = []
        let b = [1, 2, 3]
        a.concat(b)
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_concat_empty_right() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = []
        a.concat(b)
    "#,
    );
    assert_eq!(result.unwrap(), "[1, 2, 3]");
}

#[test]
fn test_array_concat_non_array_throws() {
    let result = eval(
        r#"
        let a = [1, 2]
        a.concat(42)
    "#,
    );
    assert!(
        result.is_err(),
        "concat with non-array should throw TypeError"
    );
}
