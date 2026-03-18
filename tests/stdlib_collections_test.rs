//! Tests for standard library collections module
//! TDD: Written FIRST before implementing the module

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
    }

    Ok("null".to_string())
}

// map() tests
#[test]
fn test_map_double() {
    let result = eval(r#"
        fn double(x) { return x * 2 }
        map([1, 2, 3], double)
    "#);
    assert_eq!(result.unwrap(), "[2, 4, 6]");
}

#[test]
fn test_map_add_ten() {
    let result = eval(r#"
        fn add_ten(x) { return x + 10 }
        map([1, 2, 3, 4], add_ten)
    "#);
    assert_eq!(result.unwrap(), "[11, 12, 13, 14]");
}

#[test]
fn test_map_empty_array() {
    let result = eval(r#"
        fn double(x) { return x * 2 }
        map([], double)
    "#);
    assert_eq!(result.unwrap(), "[]");
}

#[test]
fn test_map_strings() {
    let result = eval(r#"
        fn to_upper(s) { return s.upper() }
        map(["hello", "world"], to_upper)
    "#);
    assert_eq!(result.unwrap(), "[HELLO, WORLD]");
}

// filter() tests
#[test]
fn test_filter_evens() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        filter([1, 2, 3, 4, 5, 6], is_even)
    "#);
    assert_eq!(result.unwrap(), "[2, 4, 6]");
}

#[test]
fn test_filter_greater_than() {
    let result = eval(r#"
        fn gt_three(x) { return x > 3 }
        filter([1, 5, 3, 8, 2], gt_three)
    "#);
    assert_eq!(result.unwrap(), "[5, 8]");
}

#[test]
fn test_filter_empty_result() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        filter([1, 3, 5], is_even)
    "#);
    assert_eq!(result.unwrap(), "[]");
}

#[test]
fn test_filter_all_match() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        filter([2, 4, 6], is_even)
    "#);
    assert_eq!(result.unwrap(), "[2, 4, 6]");
}

// reduce() tests
#[test]
fn test_reduce_sum() {
    let result = eval(r#"
        fn add(acc, x) { return acc + x }
        reduce([1, 2, 3, 4], add, 0)
    "#);
    assert_eq!(result.unwrap(), "10");
}

#[test]
fn test_reduce_product() {
    let result = eval(r#"
        fn multiply(acc, x) { return acc * x }
        reduce([1, 2, 3, 4], multiply, 1)
    "#);
    assert_eq!(result.unwrap(), "24");
}

#[test]
fn test_reduce_concat_strings() {
    let result = eval(r#"
        fn concat(acc, x) { return acc + x }
        reduce(["a", "b", "c"], concat, "")
    "#);
    assert_eq!(result.unwrap(), "abc");
}

#[test]
fn test_reduce_empty_array() {
    let result = eval(r#"
        fn add(acc, x) { return acc + x }
        reduce([], add, 42)
    "#);
    assert_eq!(result.unwrap(), "42");
}

// find() tests
#[test]
fn test_find_first_match() {
    let result = eval(r#"
        fn gt_two(x) { return x > 2 }
        find([1, 2, 3, 4, 5], gt_two)
    "#);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_find_no_match() {
    let result = eval(r#"
        fn gt_ten(x) { return x > 10 }
        find([1, 2, 3], gt_ten)
    "#);
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_find_string() {
    let result = eval(r#"
        fn is_banana(s) { return s == "banana" }
        find(["apple", "banana", "cherry"], is_banana)
    "#);
    assert_eq!(result.unwrap(), "banana");
}

// every() tests
#[test]
fn test_every_all_true() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        every([2, 4, 6, 8], is_even)
    "#);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_every_one_false() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        every([2, 4, 5, 8], is_even)
    "#);
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_every_empty_array() {
    let result = eval(r#"
        fn gt_ten(x) { return x > 10 }
        every([], gt_ten)
    "#);
    assert_eq!(result.unwrap(), "true"); // Vacuous truth
}

// some() tests
#[test]
fn test_some_has_match() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        some([1, 3, 4, 5], is_even)
    "#);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_some_no_match() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        some([1, 3, 5], is_even)
    "#);
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_some_empty_array() {
    let result = eval(r#"
        fn gt_ten(x) { return x > 10 }
        some([], gt_ten)
    "#);
    assert_eq!(result.unwrap(), "false");
}

// Composition tests
#[test]
fn test_map_filter_composition() {
    let result = eval(r#"
        fn double(x) { return x * 2 }
        fn gt_five(x) { return x > 5 }
        let doubled = map([1, 2, 3, 4, 5], double)
        filter(doubled, gt_five)
    "#);
    assert_eq!(result.unwrap(), "[6, 8, 10]");
}

#[test]
fn test_filter_reduce_composition() {
    let result = eval(r#"
        fn is_even(x) { return x % 2 == 0 }
        fn add(acc, x) { return acc + x }
        let evens = filter([1, 2, 3, 4, 5, 6], is_even)
        reduce(evens, add, 0)
    "#);
    assert_eq!(result.unwrap(), "12"); // 2 + 4 + 6
}

#[test]
fn test_map_reduce_composition() {
    let result = eval(r#"
        fn square(x) { return x * x }
        fn add(acc, x) { return acc + x }
        let squared = map([1, 2, 3, 4], square)
        reduce(squared, add, 0)
    "#);
    assert_eq!(result.unwrap(), "30"); // 1 + 4 + 9 + 16
}
