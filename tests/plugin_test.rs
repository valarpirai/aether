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
        // Execute non-expression statements
        eval.exec_stmt(last).map_err(|e| e.to_string())?;
    }

    Ok("null".to_string())
}

#[test]
fn test_load_plugin() {
    let code = r#"
        let plugin = load_plugin("examples/plugins/libexample_plugin.dylib")
        type(plugin)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "plugin");
}

#[test]
fn test_plugin_add() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.add(40, 2)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_plugin_multiply() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.multiply(6, 7)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_plugin_power() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.power(2, 10)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "1024");
}

#[test]
fn test_plugin_is_even() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        [math.is_even(4), math.is_even(7), math.is_even(100)]
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "[1, 0, 1]");
}

#[test]
fn test_plugin_composition() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.add(math.multiply(3, 4), math.power(2, 3))
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "20");
}

#[test]
fn test_plugin_wrong_arity() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.add(1, 2, 3)
    "#;
    let result = eval_code(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Plugin function failed"));
}

#[test]
fn test_plugin_wrong_type() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.add("hello", 2)
    "#;
    let result = eval_code(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Type error: expected int, got string"));
}

#[test]
fn test_plugin_nonexistent_function() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        math.nonexistent(1)
    "#;
    let result = eval_code(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Method 'nonexistent' does not exist"));
}

#[test]
fn test_plugin_load_error() {
    let code = r#"
        load_plugin("nonexistent.dylib")
    "#;
    let result = eval_code(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("load plugin"));
}

#[test]
fn test_plugin_in_function() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")

        fn compute(a, b) {
            return math.add(math.multiply(a, 2), math.power(b, 2))
        }

        compute(5, 3)
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "19"); // 5*2 + 3^2 = 10 + 9 = 19
}

#[test]
fn test_plugin_in_loop() {
    let code = r#"
        let math = load_plugin("examples/plugins/libexample_plugin.dylib")
        let sum = 0
        for i in [1, 2, 3, 4, 5] {
            sum = math.add(sum, math.multiply(i, 2))
        }
        sum
    "#;
    let result = eval_code(code).unwrap();
    assert_eq!(result, "30"); // (1+2+3+4+5)*2 = 30
}
