//! Tests for the format() builtin

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

// Happy path

#[test]
fn test_format_positional_no_spec() {
    let result = eval(r#"format("Hello, {}!", "Alice")"#);
    assert_eq!(result.unwrap(), "Hello, Alice!");
}

#[test]
fn test_format_multiple_positional() {
    let result = eval(r#"format("{} + {} = {}", 1, 2, 3)"#);
    assert_eq!(result.unwrap(), "1 + 2 = 3");
}

#[test]
fn test_format_float_precision() {
    let result = eval(r#"format("{:.2f}", 3.14159)"#);
    assert_eq!(result.unwrap(), "3.14");
}

#[test]
fn test_format_right_align_width() {
    let result = eval(r#"format("{:>10}", "hi")"#);
    assert_eq!(result.unwrap(), "        hi");
}

#[test]
fn test_format_fill_align_width_int() {
    let result = eval(r#"format("{:0>5d}", 42)"#);
    assert_eq!(result.unwrap(), "00042");
}

#[test]
fn test_format_left_align_width() {
    let result = eval(r#"format("{:<5}", "ab") + "|""#);
    assert_eq!(result.unwrap(), "ab   |");
}

#[test]
fn test_format_center_align() {
    let result = eval(r#"format("{:^6}", "hi") + "|""#);
    assert_eq!(result.unwrap(), "  hi  |");
}

#[test]
fn test_format_hex_type() {
    let result = eval(r#"format("{:x}", 255)"#);
    assert_eq!(result.unwrap(), "ff");
}

#[test]
fn test_format_octal_type() {
    let result = eval(r#"format("{:o}", 8)"#);
    assert_eq!(result.unwrap(), "10");
}

#[test]
fn test_format_binary_type() {
    let result = eval(r#"format("{:b}", 5)"#);
    assert_eq!(result.unwrap(), "101");
}

#[test]
fn test_format_escaped_braces() {
    let result = eval(r#"format("{{literal}}")"#);
    assert_eq!(result.unwrap(), "{literal}");
}

#[test]
fn test_format_no_placeholders() {
    let result = eval(r#"format("just text")"#);
    assert_eq!(result.unwrap(), "just text");
}

// Edge cases

#[test]
fn test_format_width_smaller_than_value_no_truncation() {
    let result = eval(r#"format("{:>2}", "hello")"#);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_format_zero_width_is_noop() {
    let result = eval(r#"format("{:.1f}", 3.14159)"#);
    assert_eq!(result.unwrap(), "3.1");
}

#[test]
fn test_format_unused_extra_args_allowed() {
    let result = eval(r#"format("{}", 1, 2, 3)"#);
    assert_eq!(result.unwrap(), "1");
}

// Error cases

#[test]
fn test_format_not_enough_args_errors() {
    assert!(eval(r#"format("{}")"#).is_err());
}

#[test]
fn test_format_unclosed_brace_errors() {
    assert!(eval(r#"format("{unclosed")"#).is_err());
}

#[test]
fn test_format_unmatched_closing_brace_errors() {
    assert!(eval(r#"format("unmatched}")"#).is_err());
}

#[test]
fn test_format_non_string_fmt_errors() {
    assert!(eval("format(42)").is_err());
}

#[test]
fn test_format_unknown_type_specifier_errors() {
    assert!(eval(r#"format("{:z}", 1)"#).is_err());
}

#[test]
fn test_format_no_args_errors() {
    assert!(eval("format()").is_err());
}

#[test]
fn test_format_float_type_wrong_value_errors() {
    assert!(eval(r#"format("{:.2f}", "not a number")"#).is_err());
}
