use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FileResult {
    pub passed: usize,
    pub failed: usize,
    pub fail_lines: Vec<String>,
    pub error_lines: Vec<String>,
}

/// Run `aether test [dir|file]`. Returns exit code.
pub fn run_test(args: &[String]) -> i32 {
    let binary = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("test: cannot locate aether binary: {}", e);
            return 1;
        }
    };

    let target = args.first().map(String::as_str).unwrap_or(".");
    let target_path = Path::new(target);

    let files: Vec<PathBuf> = if target_path.is_file() {
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
pub fn discover_test_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
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
pub fn run_one_test_file(binary: &Path, file: &Path) -> FileResult {
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
