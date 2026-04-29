//! Tests for module system

use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval_in_dir(source: &str, dir: &str) -> Result<String, String> {
    // Change to test directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();

    let result = eval(source);

    // Change back
    std::env::set_current_dir(original_dir).unwrap();

    result
}

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    // Execute all but last statement
    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // Evaluate last statement if it's an expression
    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

// Basic import tests
#[test]
fn test_from_import_single_function() {
    let source = r#"
from math_utils import double

double(5)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "10");
}

#[test]
fn test_from_import_multiple_functions() {
    let source = r#"
from math_utils import double, triple

double(5) + triple(3)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "19");
}

#[test]
fn test_from_import_string_function() {
    let source = r#"
from greetings import hello

hello()
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Hello!");
}

// Error cases
#[test]
fn test_module_not_found() {
    let source = r#"
from nonexistent import func
"#;
    let result = eval(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found") || err.contains("Module"));
}

#[test]
fn test_function_not_in_module() {
    let source = r#"
from math_utils import nonexistent

nonexistent()
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_err());
}

// Module caching
#[test]
fn test_import_same_module_twice() {
    let source = r#"
from math_utils import double
from math_utils import triple

double(2) + triple(2)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "10");
}

// import module as namespace
#[test]
fn test_import_module_namespace() {
    let source = r#"
import math_utils

math_utils.double(5)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "10");
}

#[test]
fn test_import_module_namespace_multiple_members() {
    let source = r#"
import math_utils

math_utils.double(3) + math_utils.triple(2)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "12");
}

#[test]
fn test_import_module_as_alias() {
    let source = r#"
import math_utils as mu

mu.square(4)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "16");
}

#[test]
fn test_import_module_member_not_found() {
    let source = r#"
import math_utils

math_utils.nonexistent
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("nonexistent"), "Unexpected error: {}", err);
}

// from module import func as alias
#[test]
fn test_from_import_with_alias() {
    let source = r#"
from math_utils import double as twice

twice(7)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "14");
}

#[test]
fn test_from_import_multiple_with_aliases() {
    let source = r#"
from math_utils import double as twice, triple as thrice

twice(3) + thrice(2)
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "12");
}

// Circular dependency detection
#[test]
fn test_circular_dependency_detected() {
    let source = r#"
import circular_a
"#;
    let result = eval_in_dir(source, "tests/test_modules");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Circular") || err.contains("circular"),
        "Expected circular dependency error, got: {}",
        err
    );
}
