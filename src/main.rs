use std::env;
use std::fs;
use std::process;

use aether_lang::checker;
use aether_lang::formatter;
use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser;
use aether_lang::parser::Parser;
use aether_lang::repl;
use aether_lang::test_runner;

fn main() {
    let args: Vec<String> = env::args().collect();

    // No arguments -> start REPL
    if args.len() == 1 {
        if let Err(e) = repl::run() {
            eprintln!("REPL error: {}", e);
            process::exit(1);
        }
        return;
    }

    let first = &args[1];
    if first == "--version" || first == "-V" {
        println!("aether {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if first == "--help" || first == "-h" {
        println!("Usage: aether [OPTIONS] [SUBCOMMAND] [script] [args...]");
        println!();
        println!("Options:");
        println!("  -V, --version          Print version and exit");
        println!("  -h, --help             Print this help and exit");
        println!();
        println!("Subcommands:");
        println!("  ast [--json] [--output <file>] <file>");
        println!(
            "                         Print the AST (default: indented tree; --json for JSON)"
        );
        println!("  fmt [--check] <file>   Format an Aether source file");
        println!(
            "    --check              Check formatting without writing; exit 1 if unformatted"
        );
        println!("  test [dir|file]        Discover and run *_test.ae files");
        println!("    (no arg)             Search current directory recursively");
        println!("  check <file>           Check for undefined variables without running");
        println!();
        println!("If no subcommand or script is given, starts the interactive REPL.");
        return;
    }

    if first == "ast" {
        let exit_code = parser::ast::run_ast(&args[2..]);
        process::exit(exit_code);
    }

    if first == "fmt" {
        let exit_code = formatter::run_fmt(&args[2..]);
        process::exit(exit_code);
    }

    if first == "test" {
        let exit_code = test_runner::run_test(&args[2..]);
        process::exit(exit_code);
    }

    if first == "check" {
        let exit_code = run_check(&args[2..]);
        process::exit(exit_code);
    }

    // First argument is the script; everything after it are script arguments
    let filename = &args[1];
    let script_args = &args[2..];
    if script_args.len() > 100 {
        eprintln!(
            "Error: too many arguments (max 100, got {})",
            script_args.len()
        );
        process::exit(1);
    }
    if let Err(e) = run_file(filename, script_args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Run `aether check <file>`. Returns exit code (0 = clean, 1 = diagnostics found or error).
fn run_check(args: &[String]) -> i32 {
    let path = match args.first() {
        Some(p) => p.as_str(),
        None => {
            eprintln!("check: no file specified");
            eprintln!("Usage: aether check <file>");
            return 1;
        }
    };

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("check: cannot read '{}': {}", path, e);
            return 1;
        }
    };

    let mut scanner = Scanner::new(&source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}:0: error: {}", path, e);
            return 1;
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}:0: error: {}", path, e);
            return 1;
        }
    };

    let diagnostics = checker::check(&program);
    if diagnostics.is_empty() {
        println!("{}: ok", path);
        0
    } else {
        for d in &diagnostics {
            eprintln!("{}:{}: {}", path, d.line, d.message);
        }
        1
    }
}

fn format_runtime_error(msg: String, line: usize) -> String {
    if line > 0 {
        format!("[line {}] {}", line, msg)
    } else {
        msg
    }
}

fn run_file(filename: &str, script_args: &[String]) -> Result<(), String> {
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

    // Expose script arguments as the global `args` array
    evaluator.set_script_args(script_args);

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
