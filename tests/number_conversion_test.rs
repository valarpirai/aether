//! Tests for hex(), oct(), bin() and int(s, base)

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

// --- hex() ---

#[test]
fn test_hex_positive() {
    assert_eq!(eval("hex(255)").unwrap(), "0xff");
}

#[test]
fn test_hex_zero() {
    assert_eq!(eval("hex(0)").unwrap(), "0x0");
}

#[test]
fn test_hex_negative() {
    assert_eq!(eval("hex(-1)").unwrap(), "-0x1");
}

#[test]
fn test_hex_large() {
    assert_eq!(eval("hex(65535)").unwrap(), "0xffff");
}

#[test]
fn test_hex_type_error() {
    assert!(eval("hex(3.14)").is_err());
}

#[test]
fn test_hex_returns_string() {
    assert_eq!(eval("type(hex(10))").unwrap(), "string");
}

// --- oct() ---

#[test]
fn test_oct_positive() {
    assert_eq!(eval("oct(8)").unwrap(), "0o10");
}

#[test]
fn test_oct_zero() {
    assert_eq!(eval("oct(0)").unwrap(), "0o0");
}

#[test]
fn test_oct_negative() {
    assert_eq!(eval("oct(-8)").unwrap(), "-0o10");
}

#[test]
fn test_oct_returns_string() {
    assert_eq!(eval("type(oct(8))").unwrap(), "string");
}

#[test]
fn test_oct_type_error() {
    assert!(eval("oct(\"x\")").is_err());
}

// --- bin() ---

#[test]
fn test_bin_positive() {
    assert_eq!(eval("bin(5)").unwrap(), "0b101");
}

#[test]
fn test_bin_zero() {
    assert_eq!(eval("bin(0)").unwrap(), "0b0");
}

#[test]
fn test_bin_negative() {
    assert_eq!(eval("bin(-5)").unwrap(), "-0b101");
}

#[test]
fn test_bin_returns_string() {
    assert_eq!(eval("type(bin(5))").unwrap(), "string");
}

#[test]
fn test_bin_type_error() {
    assert!(eval("bin(1.5)").is_err());
}

// --- int(s, base) ---

#[test]
fn test_int_from_hex_string() {
    assert_eq!(eval("int(\"ff\", 16)").unwrap(), "255");
}

#[test]
fn test_int_from_hex_prefix() {
    assert_eq!(eval("int(\"0xff\", 16)").unwrap(), "255");
}

#[test]
fn test_int_from_bin_string() {
    assert_eq!(eval("int(\"101\", 2)").unwrap(), "5");
}

#[test]
fn test_int_from_bin_prefix() {
    assert_eq!(eval("int(\"0b101\", 2)").unwrap(), "5");
}

#[test]
fn test_int_from_oct_string() {
    assert_eq!(eval("int(\"10\", 8)").unwrap(), "8");
}

#[test]
fn test_int_from_oct_prefix() {
    assert_eq!(eval("int(\"0o10\", 8)").unwrap(), "8");
}

#[test]
fn test_int_base10_explicit() {
    assert_eq!(eval("int(\"42\", 10)").unwrap(), "42");
}

#[test]
fn test_int_single_arg_unchanged() {
    assert_eq!(eval("int(\"42\")").unwrap(), "42");
}

#[test]
fn test_int_base_invalid_string_errors() {
    assert!(eval("int(\"xyz\", 16)").is_err());
}

#[test]
fn test_int_roundtrip_hex() {
    assert_eq!(eval("int(hex(255), 16)").unwrap(), "255");
}

#[test]
fn test_int_roundtrip_bin() {
    assert_eq!(eval("int(bin(42), 2)").unwrap(), "42");
}

#[test]
fn test_int_roundtrip_oct() {
    assert_eq!(eval("int(oct(64), 8)").unwrap(), "64");
}

// --- base64_encode / base64_decode ---

#[test]
fn test_base64_encode_basic() {
    assert_eq!(eval("base64_encode(\"hello\")").unwrap(), "aGVsbG8=");
}

#[test]
fn test_base64_encode_empty() {
    assert_eq!(eval("base64_encode(\"\")").unwrap(), "");
}

#[test]
fn test_base64_decode_basic() {
    assert_eq!(eval("base64_decode(\"aGVsbG8=\")").unwrap(), "hello");
}

#[test]
fn test_base64_roundtrip() {
    assert_eq!(
        eval("base64_decode(base64_encode(\"Aether lang\"))").unwrap(),
        "Aether lang"
    );
}

#[test]
fn test_base64_decode_invalid_errors() {
    assert!(eval("base64_decode(\"!!!\")").is_err());
}

#[test]
fn test_base64_encode_type_error() {
    assert!(eval("base64_encode(42)").is_err());
}

#[test]
fn test_base64_decode_type_error() {
    assert!(eval("base64_decode(42)").is_err());
}
