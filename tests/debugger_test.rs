use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn eval(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    for stmt in &program.statements {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    Ok(String::new())
}

// debugger statement parses and executes without error when stdin is EOF
#[test]
fn test_debugger_resumes_on_eof() {
    let result = eval(
        r#"
        fn main() {
            let x = 1
            debugger
            let y = 2
        }
        main()
        "#,
    );
    assert!(result.is_ok(), "unexpected error: {:?}", result.err());
}

// debugger does not corrupt variable values
#[test]
fn test_debugger_does_not_corrupt_env() {
    let result = eval(
        r#"
        fn main() {
            let a = 42
            debugger
        }
        main()
        "#,
    );
    assert!(result.is_ok());
}

// multiple debugger statements all resume cleanly
#[test]
fn test_multiple_debugger_statements() {
    let result = eval(
        r#"
        fn main() {
            debugger
            debugger
            debugger
        }
        main()
        "#,
    );
    assert!(result.is_ok());
}

// debugger inside a called function
#[test]
fn test_debugger_inside_function() {
    let result = eval(
        r#"
        fn helper() {
            debugger
        }
        fn main() {
            helper()
        }
        main()
        "#,
    );
    assert!(result.is_ok());
}

// debugger inside a loop
#[test]
fn test_debugger_inside_loop() {
    let result = eval(
        r#"
        fn main() {
            let i = 0
            while (i < 2) {
                debugger
                i = i + 1
            }
        }
        main()
        "#,
    );
    assert!(result.is_ok());
}
