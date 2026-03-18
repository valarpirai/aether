//! Tests for the lexer module

use super::scanner::Scanner;
use super::token::{Token, TokenKind};

#[test]
fn test_token_creation() {
    let token = Token::new(TokenKind::Integer(42), "42".to_string(), 1, 1);
    assert_eq!(token.kind, TokenKind::Integer(42));
    assert_eq!(token.lexeme, "42");
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
}

#[test]
fn test_token_equality() {
    let token1 = Token::new(TokenKind::Plus, "+".to_string(), 1, 1);
    let token2 = Token::new(TokenKind::Plus, "+".to_string(), 1, 1);
    assert_eq!(token1, token2);
}

#[test]
fn test_tokenize_integer() {
    let mut scanner = Scanner::new("42");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens.len(), 2); // integer + EOF
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
}

#[test]
fn test_tokenize_float() {
    let mut scanner = Scanner::new("3.14");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Float(3.14));
}

#[test]
fn test_tokenize_string() {
    let mut scanner = Scanner::new("\"hello\"");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
}

#[test]
fn test_string_escapes() {
    let mut scanner = Scanner::new(r#""hello\nworld""#);
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("hello\nworld".to_string()));
}

#[test]
fn test_tokenize_keywords() {
    let mut scanner = Scanner::new("let fn return if else");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Fn);
    assert_eq!(tokens[2].kind, TokenKind::Return);
    assert_eq!(tokens[3].kind, TokenKind::If);
    assert_eq!(tokens[4].kind, TokenKind::Else);
}

#[test]
fn test_tokenize_operators() {
    let mut scanner = Scanner::new("+ - * / == != < > <= >=");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
    assert_eq!(tokens[4].kind, TokenKind::EqualEqual);
    assert_eq!(tokens[5].kind, TokenKind::NotEqual);
    assert_eq!(tokens[6].kind, TokenKind::Less);
    assert_eq!(tokens[7].kind, TokenKind::Greater);
    assert_eq!(tokens[8].kind, TokenKind::LessEqual);
    assert_eq!(tokens[9].kind, TokenKind::GreaterEqual);
}

#[test]
fn test_tokenize_identifier() {
    let mut scanner = Scanner::new("foo bar_baz");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("bar_baz".to_string()));
}

#[test]
fn test_single_line_comment() {
    let mut scanner = Scanner::new("42 // comment\n56");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Integer(56));
}

#[test]
fn test_multi_line_comment() {
    let mut scanner = Scanner::new("42 /* comment\nmore */ 56");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Integer(56));
}

#[test]
fn test_simple_expression() {
    let mut scanner = Scanner::new("let x = 10 + 20");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Integer(10));
    assert_eq!(tokens[4].kind, TokenKind::Plus);
    assert_eq!(tokens[5].kind, TokenKind::Integer(20));
}
