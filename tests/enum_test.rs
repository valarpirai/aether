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

// --- Declaration ---

#[test]
fn test_enum_decl_binds_name() {
    let result = eval("enum Color { Red Green Blue }\ntype(Color)");
    assert_eq!(result.unwrap(), "enum");
}

// --- Unit variants ---

#[test]
fn test_unit_variant_type() {
    let result = eval("enum Dir { North South East West }\ntype(Dir.North)");
    assert_eq!(result.unwrap(), "Dir.North");
}

#[test]
fn test_unit_variant_to_string() {
    let result = eval("enum Dir { North South }\nstr(Dir.South)");
    assert_eq!(result.unwrap(), "Dir.South");
}

#[test]
fn test_unit_variant_equality_same() {
    let result = eval("enum Color { Red Green }\nColor.Red == Color.Red");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_unit_variant_equality_different() {
    let result = eval("enum Color { Red Green }\nColor.Red == Color.Green");
    assert_eq!(result.unwrap(), "false");
}

// --- Tuple variants ---

#[test]
fn test_tuple_variant_type() {
    let result = eval("enum Shape { Circle(radius) Point }\ntype(Shape.Circle(5))");
    assert_eq!(result.unwrap(), "Shape.Circle");
}

#[test]
fn test_tuple_variant_field_single() {
    let result = eval("enum Shape { Circle(radius) }\nShape.Circle(7).radius");
    assert_eq!(result.unwrap(), "7");
}

#[test]
fn test_tuple_variant_field_first() {
    let result = eval("enum Shape { Rect(width, height) }\nShape.Rect(3, 4).width");
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_tuple_variant_field_second() {
    let result = eval("enum Shape { Rect(width, height) }\nShape.Rect(3, 4).height");
    assert_eq!(result.unwrap(), "4");
}

#[test]
fn test_tuple_variant_to_string_single_field() {
    let result = eval("enum Shape { Circle(radius) }\nstr(Shape.Circle(5))");
    assert_eq!(result.unwrap(), "Shape.Circle(5)");
}

#[test]
fn test_tuple_variant_to_string_multi_field() {
    let result = eval("enum Shape { Rect(width, height) }\nstr(Shape.Rect(3, 4))");
    assert_eq!(result.unwrap(), "Shape.Rect(3, 4)");
}

#[test]
fn test_tuple_variant_equality_same() {
    let result = eval("enum Shape { Circle(radius) }\nShape.Circle(5) == Shape.Circle(5)");
    assert_eq!(result.unwrap(), "true");
}

#[test]
fn test_tuple_variant_equality_different_fields() {
    let result = eval("enum Shape { Circle(radius) }\nShape.Circle(5) == Shape.Circle(9)");
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_tuple_variant_equality_different_variants() {
    let result = eval("enum Shape { Circle(radius) Point }\nShape.Circle(5) == Shape.Point");
    assert_eq!(result.unwrap(), "false");
}

// --- Mixed variants ---

#[test]
fn test_mixed_unit_and_tuple_types() {
    let result = eval(
        r#"
enum Expr { Num(value) Nil }
let a = Expr.Num(42)
let b = Expr.Nil
str(type(a)) + " " + str(type(b))
"#,
    );
    assert_eq!(result.unwrap(), "Expr.Num Expr.Nil");
}

#[test]
fn test_mixed_variant_field_access() {
    let result = eval(
        r#"
enum Expr { Num(value) Add(left, right) Nil }
Expr.Num(42).value
"#,
    );
    assert_eq!(result.unwrap(), "42");
}

// --- type() dispatch ---

#[test]
fn test_type_check_dispatch() {
    let result = eval(
        r#"
enum Shape { Circle(radius) Rect(width, height) Point }
fn area(s) {
    if (type(s) == "Shape.Circle") {
        return s.radius * s.radius
    } else if (type(s) == "Shape.Rect") {
        return s.width * s.height
    } else {
        return 0
    }
}
str(area(Shape.Circle(3))) + " " + str(area(Shape.Rect(4, 5))) + " " + str(area(Shape.Point))
"#,
    );
    assert_eq!(result.unwrap(), "9 20 0");
}

// --- Arity errors ---

#[test]
fn test_variant_arity_too_many_args() {
    let result = eval(
        r#"
enum Shape { Circle(radius) }
let outcome = "no_error"
try {
    Shape.Circle(1, 2)
} catch(e) {
    outcome = "caught"
}
outcome
"#,
    );
    assert_eq!(result.unwrap(), "caught");
}

#[test]
fn test_variant_arity_too_few_args() {
    let result = eval(
        r#"
enum Shape { Rect(width, height) }
let outcome = "no_error"
try {
    Shape.Rect(1)
} catch(e) {
    outcome = "caught"
}
outcome
"#,
    );
    assert_eq!(result.unwrap(), "caught");
}

#[test]
fn test_unknown_variant_error() {
    let result = eval(
        r#"
enum Color { Red }
let outcome = "no_error"
try {
    Color.Purple
} catch(e) {
    outcome = "caught"
}
outcome
"#,
    );
    assert_eq!(result.unwrap(), "caught");
}
