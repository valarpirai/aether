//! Tests for match statement

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

#[test]
fn test_match_integer_literal() {
    let result = eval(
        r#"
        let x = 2
        let out = "none"
        match x {
            1 => out = "one",
            2 => out = "two",
            3 => out = "three",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "two");
}

#[test]
fn test_match_wildcard() {
    let result = eval(
        r#"
        let x = 99
        let out = "none"
        match x {
            1 => out = "one",
            _ => out = "other",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "other");
}

#[test]
fn test_match_bind_variable() {
    let result = eval(
        r#"
        let x = 42
        let captured = 0
        match x {
            n => captured = n,
        }
        captured
    "#,
    );
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_match_string_literal() {
    let result = eval(
        r#"
        let s = "hello"
        let out = "no"
        match s {
            "world" => out = "world",
            "hello" => out = "hello",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_match_bool_literal() {
    let result = eval(
        r#"
        let b = true
        let out = "no"
        match b {
            false => out = "false",
            true  => out = "true",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_match_null_literal() {
    let result = eval(
        r#"
        let v = null
        let out = "not null"
        match v {
            null => out = "null",
            _    => out = "other",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_match_or_pattern() {
    let result = eval(
        r#"
        let x = 3
        let out = "none"
        match x {
            1 | 2 | 3 => out = "small",
            _ => out = "large",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "small");
}

#[test]
fn test_match_negative_literal() {
    let result = eval(
        r#"
        let x = -5
        let out = "pos"
        match x {
            -5 => out = "neg five",
            _  => out = "other",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "neg five");
}

#[test]
fn test_match_no_arm_matches() {
    let result = eval(
        r#"
        let x = 10
        let out = "initial"
        match x {
            1 => out = "one",
            2 => out = "two",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "initial");
}

#[test]
fn test_match_block_body() {
    let result = eval(
        r#"
        let x = 5
        let out = 0
        match x {
            5 => {
                let tmp = x * 2
                out = tmp
            },
            _ => out = -1,
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "10");
}

#[test]
fn test_match_enum_variant() {
    let result = eval(
        r#"
        enum Shape { Circle(radius) Square(side) }
        let s = Shape.Circle(7)
        let out = 0
        match s {
            Shape.Circle(r) => out = r,
            Shape.Square(w) => out = w,
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "7");
}

#[test]
fn test_match_enum_variant_unit() {
    let result = eval(
        r#"
        enum Dir { North South East West }
        let d = Dir.North
        let out = "unknown"
        match d {
            Dir.North => out = "north",
            Dir.South => out = "south",
            _ => out = "ew",
        }
        out
    "#,
    );
    assert_eq!(result.unwrap(), "north");
}

#[test]
fn test_match_return_from_function() {
    let result = eval(
        r#"
        fn describe(n) {
            match n {
                0 => return "zero",
                1 => return "one",
                _ => return "many",
            }
        }
        describe(1)
    "#,
    );
    assert_eq!(result.unwrap(), "one");
}
