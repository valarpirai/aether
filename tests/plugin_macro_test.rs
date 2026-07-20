use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

/// Helper to evaluate code and get result
fn eval_code(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    let mut eval = Evaluator::new_without_stdlib();

    // Execute all but last statement
    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        eval.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // Evaluate last statement if it's an expression
    if let Some(last) = program.statements.last() {
        if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
            let value = eval.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        eval.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

#[test]
fn test_macro_plugin_load() {
    let code = r#"
        let plugin = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        type(plugin)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "plugin");
}

#[test]
fn test_macro_plugin_add() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        math.add(15, 27)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_macro_plugin_multiply() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        math.multiply(12, 12)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "144");
}

#[test]
fn test_macro_plugin_power() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        math.power(3, 4)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "81");
}

#[test]
fn test_macro_plugin_is_even() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        [math.is_even(42), math.is_even(17)]
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "[1, 0]");
}

#[test]
fn test_macro_plugin_factorial() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        [math.factorial(5), math.factorial(10)]
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "[120, 3628800]");
}

#[test]
fn test_macro_plugin_gcd() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        [math.gcd(48, 18), math.gcd(100, 35), math.gcd(17, 19)]
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "[6, 5, 1]");
}

#[test]
fn test_macro_plugin_composition() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        math.multiply(math.factorial(5), math.power(2, 3))
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "960"); // 120 * 8
}

#[test]
fn test_macro_plugin_complex_expression() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        math.gcd(math.multiply(12, 15), math.multiply(12, 20))
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "60");
}

#[test]
fn test_macro_plugin_in_loop() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin_macro.dylib")
        let sum = 0
        for i in [1, 2, 3, 4, 5] {
            sum = math.add(sum, math.factorial(i))
        }
        sum
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "153"); // 1 + 2 + 6 + 24 + 120
}
