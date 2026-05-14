//! Tests for array/dict reference semantics, deep equality, id(), and copy()

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

// --- Reference semantics ---

#[test]
fn test_array_assignment_is_reference() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        b.push(4)
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2, 3, 4]");
}

#[test]
fn test_array_mutation_visible_through_all_aliases() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        let c = b
        c.push(99)
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2, 3, 99]");
}

#[test]
fn test_dict_assignment_is_reference() {
    let result = eval(
        r#"
        let a = {"x": 1}
        let b = a
        b["y"] = 2
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "{x: 1, y: 2}");
}

#[test]
fn test_array_pop_visible_through_alias() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        b.pop()
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2]");
}

#[test]
fn test_array_index_assign_visible_through_alias() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        b[0] = 99
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[99, 2, 3]");
}

// --- id() ---

#[test]
fn test_id_same_array_ref() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        id(a) == id(b)
    "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_id_distinct_arrays() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = [1, 2, 3]
        id(a) == id(b)
    "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_id_returns_int() {
    let result = eval("type(id([1,2,3]))").unwrap();
    assert_eq!(result, "int");
}

#[test]
fn test_id_same_dict_ref() {
    let result = eval(
        r#"
        let a = {"k": 1}
        let b = a
        id(a) == id(b)
    "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

// --- Deep equality ==  ---

#[test]
fn test_array_deep_equal_same_values() {
    assert_eq!(eval("[1, 2, 3] == [1, 2, 3]").unwrap(), "true");
}

#[test]
fn test_array_deep_equal_different_values() {
    assert_eq!(eval("[1, 2, 3] == [1, 2, 4]").unwrap(), "false");
}

#[test]
fn test_array_deep_equal_different_lengths() {
    assert_eq!(eval("[1, 2] == [1, 2, 3]").unwrap(), "false");
}

#[test]
fn test_array_nested_deep_equal() {
    assert_eq!(
        eval("[[1, 2], [3, 4]] == [[1, 2], [3, 4]]").unwrap(),
        "true"
    );
    assert_eq!(
        eval("[[1, 2], [3, 4]] == [[1, 2], [3, 5]]").unwrap(),
        "false"
    );
}

#[test]
fn test_dict_deep_equal() {
    assert_eq!(
        eval(
            r#"
            let a = {"a": 1, "b": 2}
            let b = {"a": 1, "b": 2}
            a == b
        "#
        )
        .unwrap(),
        "true"
    );
    assert_eq!(
        eval(
            r#"
            let a = {"a": 1, "b": 2}
            let b = {"a": 1, "b": 3}
            a == b
        "#
        )
        .unwrap(),
        "false"
    );
}

#[test]
fn test_same_ref_equal() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = a
        a == b
    "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}

// --- copy() ---

#[test]
fn test_copy_creates_independent_array() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = copy(a)
        b.push(99)
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_copy_different_id() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = copy(a)
        id(a) == id(b)
    "#,
    )
    .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_copy_deep_clones_nested() {
    let result = eval(
        r#"
        let a = [[1, 2], [3, 4]]
        let b = copy(a)
        b[0].push(99)
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "[[1, 2], [3, 4]]");
}

#[test]
fn test_copy_dict_independent() {
    let result = eval(
        r#"
        let a = {"x": 1}
        let b = copy(a)
        b["y"] = 2
        a
    "#,
    )
    .unwrap();
    assert_eq!(result, "{x: 1}");
}

#[test]
fn test_copy_equal_values() {
    let result = eval(
        r#"
        let a = [1, 2, 3]
        let b = copy(a)
        a == b
    "#,
    )
    .unwrap();
    assert_eq!(result, "true");
}
