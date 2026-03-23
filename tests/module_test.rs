//! Tests for module system

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

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
        if let aether::parser::ast::Stmt::Expr(expr) = last {
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
