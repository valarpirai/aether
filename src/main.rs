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
        println!("  check [file|dir]       Check for undefined variables without running (default: current dir)");
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

/// Run `aether check [file|dir]`. Returns exit code (0 = clean, 1 = issues found).
/// Accepts a single `.ae` file or a directory to scan recursively.
/// Defaults to the current directory when no argument is given.
fn run_check(args: &[String]) -> i32 {
    let target = args.first().map(|s| s.as_str()).unwrap_or(".");
    let path = std::path::Path::new(target);

    let files = if path.is_dir() {
        collect_ae_files(path)
    } else if path.extension().and_then(|e| e.to_str()) == Some("ae") {
        vec![path.to_path_buf()]
    } else {
        eprintln!("check: '{}' is not an .ae file or directory", target);
        return 1;
    };

    if files.is_empty() {
        println!("check: no .ae files found in '{}'", target);
        return 0;
    }

    let mut exit_code = 0;
    for file in &files {
        if check_one_file(file) != 0 {
            exit_code = 1;
        }
    }
    exit_code
}

/// Recursively collect all `.ae` files under `dir`, skipping `target/` and hidden dirs.
fn collect_ae_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            found.extend(collect_ae_files(&p));
        } else if p.extension().and_then(|e| e.to_str()) == Some("ae") {
            found.push(p);
        }
    }
    found
}

/// Check a single `.ae` file. Prints results and returns 0 (ok) or 1 (issues).
fn check_one_file(path: &std::path::Path) -> i32 {
    let display = path.display();

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: error: {}", display, e);
            return 1;
        }
    };

    let mut scanner = Scanner::new(&source);
    let tokens = match scanner.scan_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}:0: error: {}", display, e);
            return 1;
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}:0: error: {}", display, e);
            return 1;
        }
    };

    let has_main = program.statements.iter().any(
        |s| matches!(s, aether_lang::parser::ast::Stmt::Function(name, _, _) if name == "main"),
    );
    if !has_main {
        eprintln!("{}:0: warning: no main() function defined", display);
    }

    let diagnostics = checker::check(&program);
    if diagnostics.is_empty() {
        if has_main {
            println!("{}: ok", display);
        }
        0
    } else {
        for d in &diagnostics {
            eprintln!("{}:{}: {}", display, d.line, d.message);
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

    // Require main() — check before executing any code so we fail cleanly
    // without running top-level side effects first.
    let has_main = program.statements.iter().any(
        |s| matches!(s, aether_lang::parser::ast::Stmt::Function(name, _, _) if name == "main"),
    );
    if !has_main {
        return Err(format!(
            "{}: no main() function defined. Every Aether program must have a main() function.",
            filename
        ));
    }

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
