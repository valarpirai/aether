//! Tests for stdlib collections additions: chunk, partition, zip_longest, uniq_by, first, last

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

// first() tests
#[test]
fn test_first_normal() {
    assert_eq!(eval("first([10, 20, 30])").unwrap(), "10");
}

#[test]
fn test_first_single() {
    assert_eq!(eval("first([42])").unwrap(), "42");
}

#[test]
fn test_first_empty() {
    assert_eq!(eval("first([])").unwrap(), "null");
}

// last() tests
#[test]
fn test_last_normal() {
    assert_eq!(eval("last([10, 20, 30])").unwrap(), "30");
}

#[test]
fn test_last_single() {
    assert_eq!(eval("last([42])").unwrap(), "42");
}

#[test]
fn test_last_empty() {
    assert_eq!(eval("last([])").unwrap(), "null");
}

// chunk() tests
#[test]
fn test_chunk_even() {
    assert_eq!(eval("chunk([1, 2, 3, 4], 2)").unwrap(), "[[1, 2], [3, 4]]");
}

#[test]
fn test_chunk_remainder() {
    assert_eq!(
        eval("chunk([1, 2, 3, 4, 5], 2)").unwrap(),
        "[[1, 2], [3, 4], [5]]"
    );
}

#[test]
fn test_chunk_larger_than_array() {
    assert_eq!(eval("chunk([1, 2], 5)").unwrap(), "[[1, 2]]");
}

#[test]
fn test_chunk_size_one() {
    assert_eq!(eval("chunk([1, 2, 3], 1)").unwrap(), "[[1], [2], [3]]");
}

#[test]
fn test_chunk_empty() {
    assert_eq!(eval("chunk([], 3)").unwrap(), "[]");
}

// partition() tests
#[test]
fn test_partition_basic() {
    let result = eval(
        r#"
        fn is_even(x) { return x % 2 == 0 }
        partition([1, 2, 3, 4, 5], is_even)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[[2, 4], [1, 3, 5]]");
}

#[test]
fn test_partition_all_true() {
    let result = eval(
        r#"
        fn always(x) { return true }
        partition([1, 2, 3], always)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[[1, 2, 3], []]");
}

#[test]
fn test_partition_all_false() {
    let result = eval(
        r#"
        fn never(x) { return false }
        partition([1, 2, 3], never)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[[], [1, 2, 3]]");
}

#[test]
fn test_partition_empty() {
    let result = eval(
        r#"
        fn is_even(x) { return x % 2 == 0 }
        partition([], is_even)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[[], []]");
}

// zip_longest() tests
#[test]
fn test_zip_longest_equal() {
    assert_eq!(
        eval(r#"zip_longest([1, 2], ["a", "b"], null)"#).unwrap(),
        "[[1, a], [2, b]]"
    );
}

#[test]
fn test_zip_longest_first_shorter() {
    assert_eq!(
        eval(r#"zip_longest([1], ["a", "b"], 0)"#).unwrap(),
        "[[1, a], [0, b]]"
    );
}

#[test]
fn test_zip_longest_second_shorter() {
    assert_eq!(
        eval(r#"zip_longest([1, 2, 3], ["a"], null)"#).unwrap(),
        "[[1, a], [2, null], [3, null]]"
    );
}

#[test]
fn test_zip_longest_both_empty() {
    assert_eq!(eval(r#"zip_longest([], [], null)"#).unwrap(), "[]");
}

// uniq_by() tests
#[test]
fn test_uniq_by_removes_all_dupes() {
    let result = eval(
        r#"
        fn identity(x) { return x }
        uniq_by([1, 2, 1, 3, 2], identity)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_uniq_by_key_fn() {
    let result = eval(
        r#"
        fn first_char(s) { return s[0] }
        uniq_by(["apple", "ant", "banana", "avocado"], first_char)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[apple, banana]");
}

#[test]
fn test_uniq_by_empty() {
    let result = eval(
        r#"
        fn identity(x) { return x }
        uniq_by([], identity)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[]");
}

#[test]
fn test_uniq_by_no_dupes() {
    let result = eval(
        r#"
        fn identity(x) { return x }
        uniq_by([1, 2, 3], identity)
    "#,
    )
    .unwrap();
    assert_eq!(result, "[1, 2, 3]");
}
