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

fn format_runtime_error(msg: String, line: usize) -> String {
    if line > 0 {
        format!("[line {}] {}", line, msg)
    } else {
        msg
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

    // Execute — use I/O thread pool if AETHER_IO_WORKERS is set
    let mut evaluator = if let Some(n) = std::env::var("AETHER_IO_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
    {
        Evaluator::new_with_pool(n)
    } else {
        Evaluator::new()
    };

    // Record the script file path for stack traces
    evaluator.current_file = Some(std::path::PathBuf::from(filename));

    // Override recursion depth limit if AETHER_CALL_DEPTH is set
    if let Some(depth) = std::env::var("AETHER_CALL_DEPTH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
    {
        evaluator.set_max_call_depth(depth);
    }

    evaluator
        .execute_program(&program.statements)
        .map_err(|e| format_runtime_error(e.to_string(), evaluator.current_line()))?;

    // Auto-call main()
    evaluator
        .call_main()
        .map_err(|e| format_runtime_error(e.to_string(), evaluator.current_line()))?;

    Ok(())
}
