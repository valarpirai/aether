use std::env;
use std::fs;
use std::process;

use aether_lang::formatter;
use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;
use aether_lang::repl;

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
        println!("  fmt [--check] <file>   Format an Aether source file");
        println!(
            "    --check              Check formatting without writing; exit 1 if unformatted"
        );
        println!();
        println!("If no subcommand or script is given, starts the interactive REPL.");
        return;
    }

    if first == "fmt" {
        let exit_code = run_fmt(&args[2..]);
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

/// Run `aether fmt [--check] <file>`. Returns exit code (0 = ok, 1 = error / unformatted).
fn run_fmt(args: &[String]) -> i32 {
    let mut check = false;
    let mut file: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            f if !f.starts_with('-') => file = Some(f),
            other => {
                eprintln!("fmt: unknown option '{}'", other);
                return 1;
            }
        }
    }

    let path = match file {
        Some(p) => p,
        None => {
            eprintln!("fmt: no file specified");
            eprintln!("Usage: aether fmt [--check] <file>");
            return 1;
        }
    };

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fmt: cannot read '{}': {}", path, e);
            return 1;
        }
    };

    let formatted = match formatter::format_source(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fmt: {}: {}", path, e);
            return 1;
        }
    };

    if check {
        if formatted == source {
            0
        } else {
            eprintln!("fmt: '{}' is not formatted", path);
            1
        }
    } else {
        if let Err(e) = fs::write(path, &formatted) {
            eprintln!("fmt: cannot write '{}': {}", path, e);
            return 1;
        }
        0
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
