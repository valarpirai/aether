use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    let stmts = &program.statements;
    for stmt in &stmts[..stmts.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    if let Some(last) = stmts.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            return Ok(format!("{}", evaluator.eval_expr(expr).map_err(|e| e.to_string())?));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }
    Ok(String::new())
}

// --- Power operator ---

#[test]
fn test_power_int() {
    assert_eq!(eval("2 ** 10").unwrap(), "1024");
}

#[test]
fn test_power_zero_exp() {
    assert_eq!(eval("99 ** 0").unwrap(), "1");
}

#[test]
fn test_power_one_exp() {
    assert_eq!(eval("7 ** 1").unwrap(), "7");
}

#[test]
fn test_power_float_base() {
    assert_eq!(eval("4.0 ** 0.5").unwrap(), "2");
}

#[test]
fn test_power_negative_exp_gives_float() {
    assert_eq!(eval("2 ** -1").unwrap(), "0.5");
}

#[test]
fn test_power_right_associative() {
    // 2 ** 3 ** 2 == 2 ** 9 == 512
    assert_eq!(eval("2 ** 3 ** 2").unwrap(), "512");
}

#[test]
fn test_power_type_error() {
    assert!(eval(r#""a" ** 2"#).is_err());
}

// --- Bitwise AND ---

#[test]
fn test_bitwise_and() {
    // 12 (1100) & 10 (1010) == 8 (1000)
    assert_eq!(eval("12 & 10").unwrap(), "8");
}

#[test]
fn test_bitwise_and_mask() {
    assert_eq!(eval("255 & 15").unwrap(), "15");
}

#[test]
fn test_bitwise_and_type_error() {
    assert!(eval("1.5 & 2").is_err());
}

// --- Bitwise OR ---

#[test]
fn test_bitwise_or() {
    // 12 (1100) | 3 (0011) == 15 (1111)
    assert_eq!(eval("12 | 3").unwrap(), "15");
}

#[test]
fn test_bitwise_or_flags() {
    assert_eq!(eval("4 | 2 | 1").unwrap(), "7");
}

// --- Bitwise XOR ---

#[test]
fn test_bitwise_xor() {
    // 15 (1111) ^ 10 (1010) == 5 (0101)
    assert_eq!(eval("15 ^ 10").unwrap(), "5");
}

#[test]
fn test_bitwise_xor_self() {
    assert_eq!(eval("42 ^ 42").unwrap(), "0");
}

// --- Bitwise NOT ---

#[test]
fn test_bitwise_not_zero() {
    assert_eq!(eval("~0").unwrap(), "-1");
}

#[test]
fn test_bitwise_not_minus_one() {
    assert_eq!(eval("~(-1)").unwrap(), "0");
}

#[test]
fn test_bitwise_not_type_error() {
    assert!(eval("~1.5").is_err());
}

// --- Shift left ---

#[test]
fn test_shift_left() {
    assert_eq!(eval("1 << 4").unwrap(), "16");
}

#[test]
fn test_shift_left_multiply() {
    assert_eq!(eval("3 << 2").unwrap(), "12");
}

#[test]
fn test_shift_left_out_of_range() {
    assert!(eval("1 << 64").is_err());
}

// --- Shift right ---

#[test]
fn test_shift_right() {
    assert_eq!(eval("16 >> 2").unwrap(), "4");
}

#[test]
fn test_shift_right_divide() {
    assert_eq!(eval("100 >> 2").unwrap(), "25");
}

// --- Ternary ---

#[test]
fn test_ternary_true_branch() {
    assert_eq!(eval("true ? 1 : 2").unwrap(), "1");
}

#[test]
fn test_ternary_false_branch() {
    assert_eq!(eval("false ? 1 : 2").unwrap(), "2");
}

#[test]
fn test_ternary_with_expression() {
    assert_eq!(eval("3 > 2 ? \"yes\" : \"no\"").unwrap(), "yes");
}

#[test]
fn test_ternary_nested() {
    // right-associative: true ? 1 : (false ? 2 : 3) == 1
    assert_eq!(eval("true ? 1 : false ? 2 : 3").unwrap(), "1");
}

#[test]
fn test_ternary_null_coerce() {
    // null is falsy
    assert_eq!(eval("null ? \"yes\" : \"no\"").unwrap(), "no");
}

#[test]
fn test_ternary_lazy_else() {
    // else branch should not be evaluated when condition is true
    assert_eq!(eval("1 > 0 ? 42 : 1/0").unwrap(), "42");
}

// --- Operator precedence ---

#[test]
fn test_power_higher_than_multiply() {
    // 2 * 3 ** 2 == 2 * 9 == 18
    assert_eq!(eval("2 * 3 ** 2").unwrap(), "18");
}

#[test]
fn test_shift_lower_than_addition() {
    // (1 + 1) << 2 == 2 << 2 == 8
    assert_eq!(eval("(1 + 1) << 2").unwrap(), "8");
}

#[test]
fn test_bitwise_lower_than_comparison() {
    // bitwise AND binds tighter than ==, but looser than comparison
    assert_eq!(eval("(3 & 1) == 1").unwrap(), "true");
}
