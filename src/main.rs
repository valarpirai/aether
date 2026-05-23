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
        println!("  test [dir|file]        Discover and run *_test.ae files");
        println!("    (no arg)             Search current directory recursively");
        println!();
        println!("If no subcommand or script is given, starts the interactive REPL.");
        return;
    }

    if first == "fmt" {
        let exit_code = run_fmt(&args[2..]);
        process::exit(exit_code);
    }

    if first == "test" {
        let exit_code = run_test(&args[2..]);
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

// ── aether test ──────────────────────────────────────────────────────────────

struct FileResult {
    passed: usize,
    failed: usize,
    /// Lines starting with [FAIL] from the file's stdout
    fail_lines: Vec<String>,
    /// Lines from stderr (runtime errors, panics)
    error_lines: Vec<String>,
}

/// Run `aether test [dir|file]`. Returns exit code.
fn run_test(args: &[String]) -> i32 {
    use std::path::Path;

    let binary = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("test: cannot locate aether binary: {}", e);
            return 1;
        }
    };

    // Resolve target: explicit file, explicit dir, or default to "."
    let target = args.first().map(String::as_str).unwrap_or(".");
    let target_path = Path::new(target);

    let files: Vec<std::path::PathBuf> = if target_path.is_file() {
        vec![target_path.to_path_buf()]
    } else if target_path.is_dir() {
        discover_test_files(target_path)
    } else {
        eprintln!("test: '{}' is not a file or directory", target);
        return 1;
    };

    if files.is_empty() {
        println!("No *_test.ae files found in '{}'", target);
        return 0;
    }

    println!("Running {} test file(s) in '{}'\n", files.len(), target);

    let mut total_passed: usize = 0;
    let mut total_failed: usize = 0;

    for file in &files {
        let display = file.display().to_string();
        let result = run_one_test_file(&binary, file);

        let status = if result.failed == 0 && result.error_lines.is_empty() {
            "ok"
        } else {
            "FAILED"
        };

        println!(
            "{} ... {} ({} passed, {} failed)",
            display, status, result.passed, result.failed
        );

        for line in &result.fail_lines {
            println!("    {}", line);
        }
        for line in &result.error_lines {
            println!("    error: {}", line);
        }

        total_passed += result.passed;
        total_failed += result.failed;
    }

    println!();
    println!("{}", "─".repeat(50));
    if total_failed == 0 {
        println!(
            "All tests passed: {} passed, {} failed",
            total_passed, total_failed
        );
        0
    } else {
        println!("FAILED: {} passed, {} failed", total_passed, total_failed);
        1
    }
}

/// Walk `dir` recursively and collect files matching `*_test.ae` or `test_*.ae`.
/// Skips `target/` and hidden directories.
fn discover_test_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            found.extend(discover_test_files(&path));
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with("_test.ae") || (name.starts_with("test_") && name.ends_with(".ae")) {
                found.push(path);
            }
        }
    }
    found
}

/// Run a single test file as a subprocess, parse its output, return counts.
fn run_one_test_file(binary: &std::path::Path, file: &std::path::Path) -> FileResult {
    use std::process::Command;

    let output = match Command::new(binary).arg(file).output() {
        Ok(o) => o,
        Err(e) => {
            return FileResult {
                passed: 0,
                failed: 1,
                fail_lines: Vec::new(),
                error_lines: vec![e.to_string()],
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut fail_lines: Vec<String> = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("[PASS]") {
            passed += 1;
        } else if line.starts_with("[FAIL]") {
            failed += 1;
            fail_lines.push(line.to_string());
        }
    }

    // If the file uses no test framework, fall back to exit code
    let error_lines: Vec<String> = if passed == 0 && failed == 0 {
        if output.status.success() {
            passed = 1;
            Vec::new()
        } else {
            failed = 1;
            stderr
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
    } else {
        // Framework-based: include stderr only if non-empty
        stderr
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };

    FileResult {
        passed,
        failed,
        fail_lines,
        error_lines,
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
