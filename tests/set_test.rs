//! Tests for Set type

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

// Basic set creation
#[test]
fn test_set_from_array() {
    let source = r#"
set([1, 2, 3])
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    let output = result.unwrap();
    // Set output is sorted for display
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("3"));
}

#[test]
fn test_set_removes_duplicates() {
    let source = r#"
let s = set([1, 2, 2, 3, 3, 3])
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_empty_set() {
    let source = r#"
let s = set([])
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_set_with_strings() {
    let source = r#"
let s = set(["apple", "banana", "apple"])
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

// Set methods
#[test]
fn test_set_add() {
    let source = r#"
let s = set([1, 2])
s.add(3)
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_set_add_duplicate() {
    let source = r#"
let s = set([1, 2])
s.add(2)
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_set_remove() {
    let source = r#"
let s = set([1, 2, 3])
s.remove(2)
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_set_remove_nonexistent() {
    let source = r#"
let s = set([1, 2])
s.remove(99)
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_set_contains_true() {
    let source = r#"
let s = set([1, 2, 3])
s.contains(2)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_set_contains_false() {
    let source = r#"
let s = set([1, 2, 3])
s.contains(99)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_set_clear() {
    let source = r#"
let s = set([1, 2, 3])
s.clear()
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "0");
}

#[test]
fn test_set_to_array() {
    let source = r#"
let s = set([3, 1, 2])
let arr = s.to_array()
len(arr)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "3");
}

// Set operations
#[test]
fn test_set_union() {
    let source = r#"
let s1 = set([1, 2, 3])
let s2 = set([3, 4, 5])
let s3 = s1.union(s2)
s3.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "5");
}

#[test]
fn test_set_intersection() {
    let source = r#"
let s1 = set([1, 2, 3, 4])
let s2 = set([3, 4, 5, 6])
let s3 = s1.intersection(s2)
s3.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_set_difference() {
    let source = r#"
let s1 = set([1, 2, 3, 4])
let s2 = set([3, 4])
let s3 = s1.difference(s2)
s3.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_set_is_subset_true() {
    let source = r#"
let s1 = set([1, 2])
let s2 = set([1, 2, 3, 4])
s1.is_subset(s2)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_set_is_subset_false() {
    let source = r#"
let s1 = set([1, 2, 5])
let s2 = set([1, 2, 3, 4])
s1.is_subset(s2)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "false");
}

// Error cases
#[test]
fn test_set_non_hashable_array() {
    let source = r#"
set([[1, 2], [3, 4]])
"#;
    let result = eval(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not hashable") || err.contains("hashable"));
}

#[test]
fn test_set_add_non_hashable() {
    let source = r#"
let s = set([1, 2])
s.add([3, 4])
"#;
    let result = eval(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not hashable") || err.contains("hashable"));
}

#[test]
fn test_set_from_non_array() {
    let source = r#"
set(42)
"#;
    let result = eval(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("array") || err.contains("TypeError"));
}

// Integration tests
#[test]
fn test_set_truthiness_empty() {
    let source = r#"
let s = set([])
s.size == 0
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_set_truthiness_non_empty() {
    let source = r#"
let s = set([1])
s.size > 0
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_set_with_mixed_types() {
    let source = r#"
let s = set([1, "hello", true, null])
s.size
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "4");
}

#[test]
fn test_set_chained_operations() {
    let source = r#"
let s = set([1, 2])
s.add(3)
s.add(4)
s.remove(2)
s.contains(3)
"#;
    let result = eval(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "true");
}
