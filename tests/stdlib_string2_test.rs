//! Tests for stdlib string additions: contains, index_of, replace, count, pad_left, pad_right,
//! strip_prefix, strip_suffix, is_alpha, is_digit, is_space

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

// contains()
#[test]
fn test_contains_found() {
    assert_eq!(eval(r#"contains("hello world", "world")"#).unwrap(), "true");
}

#[test]
fn test_contains_not_found() {
    assert_eq!(eval(r#"contains("hello", "xyz")"#).unwrap(), "false");
}

#[test]
fn test_contains_empty_needle() {
    assert_eq!(eval(r#"contains("hello", "")"#).unwrap(), "true");
}

#[test]
fn test_contains_empty_haystack() {
    assert_eq!(eval(r#"contains("", "a")"#).unwrap(), "false");
}

// index_of()
#[test]
fn test_index_of_found() {
    assert_eq!(eval(r#"index_of("hello", "ll")"#).unwrap(), "2");
}

#[test]
fn test_index_of_not_found() {
    assert_eq!(eval(r#"index_of("hello", "xyz")"#).unwrap(), "-1");
}

#[test]
fn test_index_of_start() {
    assert_eq!(eval(r#"index_of("hello", "he")"#).unwrap(), "0");
}

#[test]
fn test_index_of_empty_needle() {
    assert_eq!(eval(r#"index_of("hello", "")"#).unwrap(), "0");
}

// replace()
#[test]
fn test_replace_basic() {
    assert_eq!(
        eval(r#"replace("hello world", "world", "aether")"#).unwrap(),
        "hello aether"
    );
}

#[test]
fn test_replace_all_occurrences() {
    assert_eq!(eval(r#"replace("aabbaa", "aa", "x")"#).unwrap(), "xbbx");
}

#[test]
fn test_replace_no_match() {
    assert_eq!(eval(r#"replace("hello", "xyz", "abc")"#).unwrap(), "hello");
}

#[test]
fn test_replace_empty_string() {
    assert_eq!(eval(r#"replace("", "a", "b")"#).unwrap(), "");
}

// count()
#[test]
fn test_count_occurrences() {
    assert_eq!(eval(r#"count("banana", "a")"#).unwrap(), "3");
}

#[test]
fn test_count_no_match() {
    assert_eq!(eval(r#"count("hello", "x")"#).unwrap(), "0");
}

#[test]
fn test_count_overlapping() {
    assert_eq!(eval(r#"count("aaa", "a")"#).unwrap(), "3");
}

#[test]
fn test_count_empty_string() {
    assert_eq!(eval(r#"count("", "a")"#).unwrap(), "0");
}

// pad_left()
#[test]
fn test_pad_left_basic() {
    assert_eq!(eval(r#"pad_left("42", 5, "0")"#).unwrap(), "00042");
}

#[test]
fn test_pad_left_no_padding_needed() {
    assert_eq!(eval(r#"pad_left("hello", 3, " ")"#).unwrap(), "hello");
}

#[test]
fn test_pad_left_exact_width() {
    assert_eq!(eval(r#"pad_left("hi", 2, " ")"#).unwrap(), "hi");
}

#[test]
fn test_pad_left_space() {
    assert_eq!(eval(r#"pad_left("x", 4, " ")"#).unwrap(), "   x");
}

// pad_right()
#[test]
fn test_pad_right_basic() {
    assert_eq!(eval(r#"pad_right("hi", 5, ".")"#).unwrap(), "hi...");
}

#[test]
fn test_pad_right_no_padding_needed() {
    assert_eq!(eval(r#"pad_right("hello", 3, " ")"#).unwrap(), "hello");
}

#[test]
fn test_pad_right_space() {
    assert_eq!(eval(r#"pad_right("x", 4, " ")"#).unwrap(), "x   ");
}

// strip_prefix()
#[test]
fn test_strip_prefix_found() {
    assert_eq!(
        eval(r#"strip_prefix("hello world", "hello ")"#).unwrap(),
        "world"
    );
}

#[test]
fn test_strip_prefix_not_found() {
    assert_eq!(
        eval(r#"strip_prefix("hello world", "xyz")"#).unwrap(),
        "hello world"
    );
}

#[test]
fn test_strip_prefix_empty() {
    assert_eq!(eval(r#"strip_prefix("hello", "")"#).unwrap(), "hello");
}

// strip_suffix()
#[test]
fn test_strip_suffix_found() {
    assert_eq!(eval(r#"strip_suffix("hello.ae", ".ae")"#).unwrap(), "hello");
}

#[test]
fn test_strip_suffix_not_found() {
    assert_eq!(
        eval(r#"strip_suffix("hello.ae", ".rs")"#).unwrap(),
        "hello.ae"
    );
}

#[test]
fn test_strip_suffix_empty() {
    assert_eq!(eval(r#"strip_suffix("hello", "")"#).unwrap(), "hello");
}

// is_alpha()
#[test]
fn test_is_alpha_true() {
    assert_eq!(eval(r#"is_alpha("hello")"#).unwrap(), "true");
    assert_eq!(eval(r#"is_alpha("ABC")"#).unwrap(), "true");
}

#[test]
fn test_is_alpha_false() {
    assert_eq!(eval(r#"is_alpha("hello1")"#).unwrap(), "false");
    assert_eq!(eval(r#"is_alpha("hi there")"#).unwrap(), "false");
}

#[test]
fn test_is_alpha_empty() {
    assert_eq!(eval(r#"is_alpha("")"#).unwrap(), "false");
}

// is_digit()
#[test]
fn test_is_digit_true() {
    assert_eq!(eval(r#"is_digit("12345")"#).unwrap(), "true");
    assert_eq!(eval(r#"is_digit("0")"#).unwrap(), "true");
}

#[test]
fn test_is_digit_false() {
    assert_eq!(eval(r#"is_digit("12.3")"#).unwrap(), "false");
    assert_eq!(eval(r#"is_digit("12a")"#).unwrap(), "false");
}

#[test]
fn test_is_digit_empty() {
    assert_eq!(eval(r#"is_digit("")"#).unwrap(), "false");
}

// is_space()
#[test]
fn test_is_space_true() {
    assert_eq!(eval(r#"is_space("   ")"#).unwrap(), "true");
    assert_eq!(eval(r#"is_space(" ")"#).unwrap(), "true");
}

#[test]
fn test_is_space_false() {
    assert_eq!(eval(r#"is_space("  a  ")"#).unwrap(), "false");
}

#[test]
fn test_is_space_empty() {
    assert_eq!(eval(r#"is_space("")"#).unwrap(), "false");
}
