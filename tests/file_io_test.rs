use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(src: &str) -> Result<Evaluator, String> {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;
    Ok(evaluator)
}

fn run_with_result(src: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(src);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    evaluator
        .execute_program(&program.statements)
        .map_err(|e| e.to_string())?;
    let val = evaluator
        .environment
        .get("result")
        .map_err(|e| e.to_string())?;
    Ok(format!("{}", val))
}

fn with_temp_file(content: &str, f: impl FnOnce(&str)) {
    let path = format!("/tmp/aether_test_{}.txt", std::process::id());
    std::fs::write(&path, content).unwrap();
    f(&path);
    let _ = std::fs::remove_file(&path);
}

// --- read_lines ---

#[test]
fn test_read_lines_returns_array() {
    with_temp_file("alpha\nbeta\ngamma\n", |path| {
        let src = format!(r#"let result = read_lines("{}")"#, path);
        let result = run_with_result(&src).unwrap();
        assert_eq!(result, "[alpha, beta, gamma]");
    });
}

#[test]
fn test_read_lines_empty_file() {
    with_temp_file("", |path| {
        let src = format!(r#"let result = read_lines("{}")"#, path);
        let result = run_with_result(&src).unwrap();
        assert_eq!(result, "[]");
    });
}

#[test]
fn test_read_lines_count() {
    with_temp_file("a\nb\nc\n", |path| {
        let src = format!(r#"let result = len(read_lines("{}"))"#, path);
        let result = run_with_result(&src).unwrap();
        assert_eq!(result, "3");
    });
}

// --- append_file ---

#[test]
fn test_append_file_creates_and_appends() {
    let path = format!("/tmp/aether_append_{}.txt", std::process::id());
    let _ = std::fs::remove_file(&path);

    let src1 = format!(r#"append_file("{}", "hello\n")"#, path);
    let src2 = format!(r#"append_file("{}", "world\n")"#, path);
    run(&src1).unwrap();
    run(&src2).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "hello\nworld\n");
    let _ = std::fs::remove_file(&path);
}

// --- file_exists ---

#[test]
fn test_file_exists_true() {
    with_temp_file("data", |path| {
        let src = format!(r#"let result = file_exists("{}")"#, path);
        assert_eq!(run_with_result(&src).unwrap(), "true");
    });
}

#[test]
fn test_file_exists_false() {
    let src = r#"let result = file_exists("/tmp/definitely_does_not_exist_aether.txt")"#;
    assert_eq!(run_with_result(src).unwrap(), "false");
}

// --- is_file / is_dir ---

#[test]
fn test_is_file() {
    with_temp_file("content", |path| {
        let src = format!(r#"let result = is_file("{}")"#, path);
        assert_eq!(run_with_result(&src).unwrap(), "true");
    });
}

#[test]
fn test_is_dir() {
    let src = r#"let result = is_dir("/tmp")"#;
    assert_eq!(run_with_result(src).unwrap(), "true");
}

#[test]
fn test_is_file_on_dir_returns_false() {
    let src = r#"let result = is_file("/tmp")"#;
    assert_eq!(run_with_result(src).unwrap(), "false");
}

// --- mkdir ---

#[test]
fn test_mkdir_creates_directory() {
    let path = format!("/tmp/aether_mkdir_{}/sub/dir", std::process::id());
    let src = format!(r#"mkdir("{}")"#, path);
    run(&src).unwrap();
    assert!(std::path::Path::new(&path).is_dir());
    let _ = std::fs::remove_dir_all(format!("/tmp/aether_mkdir_{}", std::process::id()));
}

// --- lines_iter (streaming) ---

#[test]
fn test_lines_iter_for_loop() {
    with_temp_file("line1\nline2\nline3\n", |path| {
        let src = format!(
            r#"let count = 0
for line in lines_iter("{}") {{
    count = count + 1
}}
let result = count"#,
            path
        );
        assert_eq!(run_with_result(&src).unwrap(), "3");
    });
}

#[test]
fn test_lines_iter_has_next_next() {
    with_temp_file("alpha\nbeta\n", |path| {
        let src = format!(
            r#"let it = lines_iter("{}")
let a = it.next()
let b = it.next()
let c = it.next()
let result = a"#,
            path
        );
        assert_eq!(run_with_result(&src).unwrap(), "alpha");

        let src2 = format!(
            r#"let it = lines_iter("{}")
it.next()
let result = it.next()"#,
            path
        );
        assert_eq!(run_with_result(&src2).unwrap(), "beta");
    });
}

#[test]
fn test_lines_iter_next_at_eof_returns_null() {
    with_temp_file("only\n", |path| {
        let src = format!(
            r#"let it = lines_iter("{}")
it.next()
let result = it.next()"#,
            path
        );
        assert_eq!(run_with_result(&src).unwrap(), "null");
    });
}

#[test]
fn test_lines_iter_has_next() {
    with_temp_file("x\n", |path| {
        let src = format!(
            r#"let it = lines_iter("{}")
let r1 = it.has_next()
it.next()
let result = it.has_next()"#,
            path
        );
        assert_eq!(run_with_result(&src).unwrap(), "false");
    });
}

// --- read_bytes / write_bytes ---

#[test]
fn test_write_bytes_and_read_bytes_roundtrip() {
    let path = format!("/tmp/aether_bytes_{}.bin", std::process::id());
    let src = format!(r#"write_bytes("{}", [72, 101, 108, 108, 111])"#, path);
    run(&src).unwrap();
    let content = std::fs::read(&path).unwrap();
    assert_eq!(content, b"Hello");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_read_bytes_returns_int_array() {
    let path = format!("/tmp/aether_rbytes_{}.bin", std::process::id());
    std::fs::write(&path, b"\x00\xFF\x7F").unwrap();
    let src = format!(r#"let result = read_bytes("{}")"#, path);
    assert_eq!(run_with_result(&src).unwrap(), "[0, 255, 127]");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_bytes_out_of_range_errors() {
    let path = format!("/tmp/aether_berr_{}.bin", std::process::id());
    let src = format!(r#"write_bytes("{}", [100, 300])"#, path);
    assert!(run(&src).is_err());
    let _ = std::fs::remove_file(&path);
}

// --- list_dir ---

#[test]
fn test_list_dir_returns_sorted_names() {
    let dir = format!("/tmp/aether_lsdir_{}", std::process::id());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{}/b.txt", dir), "").unwrap();
    std::fs::write(format!("{}/a.txt", dir), "").unwrap();
    std::fs::write(format!("{}/c.txt", dir), "").unwrap();
    let src = format!(r#"let result = list_dir("{}")"#, dir);
    let result = run_with_result(&src).unwrap();
    assert_eq!(result, "[a.txt, b.txt, c.txt]");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_list_dir_missing_path_errors() {
    let src = r#"list_dir("/nonexistent_aether_dir_xyz")"#;
    let err = run(src).err().unwrap();
    assert!(
        err.contains("list_dir"),
        "error should mention list_dir: {}",
        err
    );
    assert!(
        err.contains("nonexistent_aether_dir_xyz"),
        "error should contain path: {}",
        err
    );
}

// --- path_join ---

#[test]
fn test_path_join_two_parts() {
    let src = r#"let result = path_join("/tmp", "file.txt")"#;
    assert_eq!(run_with_result(src).unwrap(), "/tmp/file.txt");
}

#[test]
fn test_path_join_three_parts() {
    let src = r#"let result = path_join("/home", "user", "docs")"#;
    assert_eq!(run_with_result(src).unwrap(), "/home/user/docs");
}

// --- rename ---

#[test]
fn test_rename_moves_file() {
    let src_path = format!("/tmp/aether_rename_src_{}.txt", std::process::id());
    let dst_path = format!("/tmp/aether_rename_dst_{}.txt", std::process::id());
    std::fs::write(&src_path, "hello").unwrap();
    let code = format!(r#"rename("{}", "{}")"#, src_path, dst_path);
    run(&code).unwrap();
    assert!(!std::path::Path::new(&src_path).exists());
    assert_eq!(std::fs::read_to_string(&dst_path).unwrap(), "hello");
    let _ = std::fs::remove_file(&dst_path);
}

#[test]
fn test_rename_missing_src_errors() {
    let src = r#"rename("/nonexistent_src_xyz.txt", "/tmp/dst.txt")"#;
    let err = run(src).err().unwrap();
    assert!(
        err.contains("rename"),
        "error should mention rename: {}",
        err
    );
}

// --- rm ---

#[test]
fn test_rm_removes_file() {
    let path = format!("/tmp/aether_rm_{}.txt", std::process::id());
    std::fs::write(&path, "bye").unwrap();
    let code = format!(r#"rm("{}")"#, path);
    run(&code).unwrap();
    assert!(!std::path::Path::new(&path).exists());
}

#[test]
fn test_rm_missing_file_errors() {
    let src = r#"rm("/nonexistent_aether_file_xyz.txt")"#;
    let err = run(src).err().unwrap();
    assert!(err.contains("rm"), "error should mention rm: {}", err);
    assert!(
        err.contains("nonexistent_aether_file_xyz"),
        "error should contain path: {}",
        err
    );
}
