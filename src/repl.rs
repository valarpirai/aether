//! REPL (Read-Eval-Print Loop) for interactive Aether sessions

use crate::interpreter::Evaluator;
use crate::lexer::Scanner;
use crate::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the interactive REPL
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Aether v0.1.0");
    println!("Type '_help' for commands, Ctrl+D to exit\n");

    let mut rl = DefaultEditor::new()?;
    let mut evaluator = Evaluator::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(&line);

                // Handle special commands
                if line.trim().starts_with('_') {
                    handle_command(line.trim(), &evaluator);
                    continue;
                }

                // Execute code
                match execute_line(&line, &mut evaluator) {
                    Ok(Some(result)) => println!("{}", result),
                    Ok(None) => {} // Statement executed, no output
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

/// Execute a line of code
fn execute_line(
    source: &str,
    evaluator: &mut Evaluator,
) -> Result<Option<String>, String> {
    // Tokenize
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    // Execute statements
    if program.statements.is_empty() {
        return Ok(None);
    }

    // Check if last statement is an expression
    let (stmts, maybe_expr) = if program.statements.len() == 1 {
        if let crate::parser::ast::Stmt::Expr(expr) = &program.statements[0] {
            (&[] as &[crate::parser::ast::Stmt], Some(expr))
        } else {
            (&program.statements[..], None)
        }
    } else {
        (&program.statements[..], None)
    };

    // Execute all statements
    for stmt in stmts {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }

    // If last was an expression, evaluate and return
    if let Some(expr) = maybe_expr {
        let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
        Ok(Some(format!("{}", value)))
    } else {
        Ok(None)
    }
}

/// Handle special REPL commands
fn handle_command(cmd: &str, evaluator: &Evaluator) {
    match cmd {
        "_help" => {
            println!("Special commands:");
            println!("  _help    - Show this help");
            println!("  _env     - Show current environment");
            println!("  _tokens  - Show tokens (not yet implemented)");
            println!("  _ast     - Show AST (not yet implemented)");
            println!("  _exit    - Exit REPL (or use Ctrl+D)");
        }
        "_exit" => {
            std::process::exit(0);
        }
        "_env" => {
            println!("Current environment: {:?}", evaluator.environment);
        }
        "_tokens" => {
            println!("Token inspection not yet implemented");
        }
        "_ast" => {
            println!("AST inspection not yet implemented");
        }
        _ => {
            println!("Unknown command: {}", cmd);
            println!("Type '_help' for available commands");
        }
    }
}
