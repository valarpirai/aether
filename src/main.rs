use std::env;
use std::fs;
use std::process;

use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;
use aether::repl;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Usage: aether [script.ae]");
        process::exit(1);
    }

    // No arguments -> start REPL
    if args.len() == 1 {
        if let Err(e) = repl::run() {
            eprintln!("REPL error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Argument provided -> run file
    let filename = &args[1];
    if let Err(e) = run_file(filename) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_file(filename: &str) -> Result<(), String> {
    // Read file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file '{}': {}", filename, e))?;

    // Tokenize
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    // Execute
    let mut evaluator = Evaluator::new();
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;

    Ok(())
}
