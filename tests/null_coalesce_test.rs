use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(source: &str) -> Result<String, String> {
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

// ── Null coalescing (??) ─────────────────────────────────────────────────────

#[test]
fn test_null_coalesce_left_is_null() {
    assert_eq!(run("null ?? 42").unwrap(), "42");
}

#[test]
fn test_null_coalesce_left_is_not_null() {
    assert_eq!(run("7 ?? 42").unwrap(), "7");
}

#[test]
fn test_null_coalesce_left_is_zero() {
    // 0 is not null, so 0 should be returned
    assert_eq!(run("0 ?? 42").unwrap(), "0");
}

#[test]
fn test_null_coalesce_left_is_false() {
    // false is not null, so false is returned
    assert_eq!(run("false ?? true").unwrap(), "false");
}

#[test]
fn test_null_coalesce_left_is_empty_string() {
    assert_eq!(run(r#""" ?? "default""#).unwrap(), "");
}

#[test]
fn test_null_coalesce_chained() {
    assert_eq!(run("null ?? null ?? 99").unwrap(), "99");
}

#[test]
fn test_null_coalesce_chained_first_wins() {
    assert_eq!(run("1 ?? 2 ?? 3").unwrap(), "1");
}

#[test]
fn test_null_coalesce_with_variable() {
    let src = r#"
let x = null
x ?? "fallback"
"#;
    assert_eq!(run(src).unwrap(), "fallback");
}

#[test]
fn test_null_coalesce_short_circuits() {
    // Right side must NOT be evaluated when left is non-null;
    // undefined_fn() would error if called.
    let src = r#"
let x = 5
x ?? undefined_fn()
"#;
    assert_eq!(run(src).unwrap(), "5");
}

#[test]
fn test_null_coalesce_right_is_expression() {
    let src = r#"
let x = null
x ?? (1 + 2)
"#;
    assert_eq!(run(src).unwrap(), "3");
}

// ── Optional member access (?.) ───────────────────────────────────────────────

#[test]
fn test_optional_member_null_returns_null() {
    assert_eq!(run("let x = null\nx?.length").unwrap(), "null");
}

#[test]
fn test_optional_member_on_string() {
    assert_eq!(
        run(r#"let s = "hello"
s?.length"#)
        .unwrap(),
        "5"
    );
}

#[test]
fn test_optional_member_on_array() {
    assert_eq!(run("let a = [1, 2, 3]\na?.length").unwrap(), "3");
}

#[test]
fn test_optional_member_chained_with_coalesce() {
    let src = r#"
let x = null
x?.length ?? 0
"#;
    assert_eq!(run(src).unwrap(), "0");
}

// ── Optional method call (?.) ─────────────────────────────────────────────────

#[test]
fn test_optional_call_null_returns_null() {
    assert_eq!(
        run(r#"let s = null
s?.upper()"#)
        .unwrap(),
        "null"
    );
}

#[test]
fn test_optional_call_on_string() {
    assert_eq!(
        run(r#"let s = "hello"
s?.upper()"#)
        .unwrap(),
        "HELLO"
    );
}

#[test]
fn test_optional_call_with_args() {
    assert_eq!(
        run(r#"let s = "hello world"
s?.split(" ")"#)
        .unwrap(),
        "[hello, world]"
    );
}

#[test]
fn test_optional_call_null_skips_body() {
    assert_eq!(
        run(r#"let s = null
s?.upper()"#)
        .unwrap(),
        "null"
    );
}

#[test]
fn test_optional_chaining_combined_null() {
    let src = r#"
let name = null
name?.upper() ?? "UNKNOWN"
"#;
    assert_eq!(run(src).unwrap(), "UNKNOWN");
}

#[test]
fn test_optional_chaining_combined_non_null() {
    let src = r#"
let name = "alice"
name?.upper() ?? "UNKNOWN"
"#;
    assert_eq!(run(src).unwrap(), "ALICE");
}

#[test]
fn test_null_coalesce_with_array() {
    let src = r#"
let arr = null
arr ?? [1, 2, 3]
"#;
    assert_eq!(run(src).unwrap(), "[1, 2, 3]");
}

#[test]
fn test_optional_call_on_array_contains() {
    let src = r#"
let a = [1, 2, 3]
a?.contains(2)
"#;
    assert_eq!(run(src).unwrap(), "true");
}

#[test]
fn test_optional_call_null_contains() {
    let src = r#"
let a = null
a?.contains(2)
"#;
    assert_eq!(run(src).unwrap(), "null");
}
