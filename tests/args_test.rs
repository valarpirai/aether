use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval_with_args(source: &str, script_args: &[&str]) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    let owned: Vec<String> = script_args.iter().map(|s| s.to_string()).collect();
    evaluator.set_script_args(&owned);
    let stmts = &program.statements;
    for stmt in &stmts[..stmts.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    if let Some(last) = stmts.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let val = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", val));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }
    Ok(String::new())
}

// args is an empty array when no script arguments are passed
#[test]
fn test_args_empty_by_default() {
    let result = eval_with_args("len(args)", &[]);
    assert_eq!(result.unwrap(), "0");
}

// args contains all passed strings in order
#[test]
fn test_args_contains_passed_values() {
    let result = eval_with_args("args[0]", &["hello"]);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_args_multiple_values() {
    let result = eval_with_args("len(args)", &["a", "b", "c"]);
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_args_index_access() {
    let result = eval_with_args("args[1]", &["first", "second", "third"]);
    assert_eq!(result.unwrap(), "second");
}

// args elements are strings
#[test]
fn test_args_elements_are_strings() {
    let result = eval_with_args(r#"type(args[0])"#, &["42"]);
    assert_eq!(result.unwrap(), "string");
}
