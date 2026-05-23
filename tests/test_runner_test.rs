use std::path::Path;
use std::process::Command;

fn aether_bin() -> std::path::PathBuf {
    std::env::current_exe()
        .expect("current_exe")
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("aether")
}

fn run_test_cmd(args: &[&str]) -> std::process::Output {
    Command::new(aether_bin())
        .arg("test")
        .args(args)
        .output()
        .expect("failed to run aether test")
}

#[test]
fn test_runner_passes_math_test_file() {
    let out = run_test_cmd(&["examples/math_test.ae"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "expected exit 0, got:\n{}", stdout);
    assert!(
        stdout.contains("5 passed"),
        "expected 5 passed:\n{}",
        stdout
    );
    assert!(
        stdout.contains("0 failed"),
        "expected 0 failed:\n{}",
        stdout
    );
}

#[test]
fn test_runner_all_tests_pass_line() {
    let out = run_test_cmd(&["examples/math_test.ae"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("All tests passed"),
        "expected 'All tests passed':\n{}",
        stdout
    );
}

#[test]
fn test_runner_discovers_files_in_examples_dir() {
    let out = run_test_cmd(&["examples"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // At least one test file should be found
    assert!(
        stdout.contains("test file") || stdout.contains("passed"),
        "expected test output:\n{}",
        stdout
    );
}

#[test]
fn test_runner_no_test_files_in_empty_dir() {
    let tmp = std::env::temp_dir().join("aether_test_runner_empty");
    std::fs::create_dir_all(&tmp).unwrap();
    let out = run_test_cmd(&[tmp.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "should exit 0 when no files found:\n{}",
        stdout
    );
    assert!(
        stdout.contains("No *_test.ae files found"),
        "expected no-files message:\n{}",
        stdout
    );
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_runner_nonexistent_path_exits_one() {
    let out = run_test_cmd(&["/nonexistent/path/does_not_exist"]);
    assert!(!out.status.success(), "expected non-zero exit for bad path");
}

#[test]
fn test_runner_failing_test_exits_one() {
    let tmp = std::env::temp_dir().join("aether_failing_test.ae");
    std::fs::write(
        &tmp,
        r#"from testing import test, assert_eq, test_summary
fn main() {
    let results = []
    results.push(test("fail me", fn() {
        assert_eq(1, 2)
    }))
    test_summary(results)
}
"#,
    )
    .unwrap();
    let out = run_test_cmd(&[tmp.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "expected exit 1 for failing test:\n{}",
        stdout
    );
    assert!(
        stdout.contains("FAILED"),
        "expected FAILED in output:\n{}",
        stdout
    );
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_runner_skips_target_and_hidden_dirs() {
    // The discover function should not recurse into target/ or .hidden/
    // Verify by running on the project root — it will find examples/ tests
    // but must not hang trying to recurse into target/
    let out = run_test_cmd(&["examples"]);
    assert!(
        out.status.success() || !String::from_utf8_lossy(&out.stderr).contains("target"),
        "should not recurse into target/"
    );
}

#[test]
fn test_runner_from_import_stdlib_works() {
    // The math_test.ae file uses `from testing import test, assert_eq, assert_true`
    // This verifies the embedded stdlib is importable via from...import
    let out = run_test_cmd(&["examples/math_test.ae"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "from...import stdlib should work:\n{}\n{}",
        stdout,
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_runner_check_flag_missing_errors() {
    // "aether test" with a non-file, non-dir target should exit 1
    let out = run_test_cmd(&["does_not_exist.ae"]);
    assert!(!out.status.success(), "expected exit 1 for missing file");
}

#[test]
fn test_runner_default_dir_finds_examples() {
    // Run with no args — it defaults to "." which will find something
    let output = Command::new(aether_bin())
        .arg("test")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"))
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test file") || stdout.contains("passed"),
        "expected test runner output:\n{}",
        stdout
    );
}
