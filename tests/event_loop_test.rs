use aether::interpreter::Evaluator;
use aether::lexer::Scanner;
use aether::parser::Parser;

fn run(source: &str) -> Result<String, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();

    for stmt in &program.statements[..program.statements.len().saturating_sub(1)] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    if let Some(last) = program.statements.last() {
        if let aether::parser::ast::Stmt::Expr(expr) = last {
            let value = evaluator.eval_expr(expr).map_err(|e| e.to_string())?;
            return Ok(format!("{}", value));
        }
        evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    }
    Ok("null".to_string())
}

fn tmp(name: &str) -> String {
    format!("/tmp/aether_el_{}.txt", name)
}

// ── event_loop with no pending tasks ─────────────────────────────────────────

#[test]
fn test_event_loop_empty_returns_null() {
    assert_eq!(run("event_loop()").unwrap(), "null");
}

#[test]
fn test_event_loop_multiple_calls_idempotent() {
    let src = "event_loop()\nevent_loop()";
    assert_eq!(run(src).unwrap(), "null");
}

// ── on_ready with already-resolved (non-promise) values ──────────────────────

#[test]
fn test_on_ready_non_promise_int_fires_immediately() {
    let f = tmp("np_int");
    let src = format!(
        "on_ready(42, fn(v) {{ write_file(\"{}\", str(v)) }})\nread_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "42");
}

#[test]
fn test_on_ready_non_promise_string_fires_immediately() {
    let f = tmp("np_str");
    let src = format!(
        "on_ready(\"hello\", fn(v) {{ write_file(\"{}\", v) }})\nread_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "hello");
}

#[test]
fn test_on_ready_non_promise_null_fires_immediately() {
    let f = tmp("np_null");
    let src = format!(
        "on_ready(null, fn(v) {{ write_file(\"{}\", \"fired\") }})\nread_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "fired");
}

#[test]
fn test_on_ready_non_promise_fires_without_event_loop() {
    // Callback should fire synchronously — event_loop() not required
    let f = tmp("no_el");
    let src = format!(
        "on_ready(99, fn(v) {{ write_file(\"{}\", str(v)) }})\nread_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "99");
}

// ── on_ready + event_loop with async sleep ────────────────────────────────────
// When the I/O pool is active, write_file/read_file are also async,
// so we use `await` to resolve them inside callbacks and after the loop.

#[test]
fn test_on_ready_sleep_fires_callback() {
    let f = tmp("sleep_fires");
    let src = format!(
        "set_workers(2)\nlet p = sleep(0.01)\non_ready(p, fn(v) {{ await write_file(\"{}\", \"ok\") }})\nevent_loop()\nawait read_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "ok");
}

#[test]
fn test_event_loop_returns_null_after_completion() {
    let src = "set_workers(2)\nlet p = sleep(0.01)\non_ready(p, fn(v) {})\nevent_loop()";
    assert_eq!(run(src).unwrap(), "null");
}

#[test]
fn test_multiple_on_ready_all_fire() {
    let f1 = tmp("multi_f1");
    let f2 = tmp("multi_f2");
    let f3 = tmp("multi_f3");
    let src = format!(
        concat!(
            "set_workers(4)\n",
            "let p1 = sleep(0.01)\nlet p2 = sleep(0.01)\nlet p3 = sleep(0.01)\n",
            "on_ready(p1, fn(v) {{ await write_file(\"{}\", \"1\") }})\n",
            "on_ready(p2, fn(v) {{ await write_file(\"{}\", \"2\") }})\n",
            "on_ready(p3, fn(v) {{ await write_file(\"{}\", \"3\") }})\n",
            "event_loop()\n",
            "await read_file(\"{}\") + await read_file(\"{}\") + await read_file(\"{}\")"
        ),
        f1, f2, f3, f1, f2, f3
    );
    // All three callbacks fired — order may vary, so just check length
    assert_eq!(run(&src).unwrap().len(), 3);
}

#[test]
fn test_event_loop_waits_for_all_pending() {
    let f1 = tmp("wait_f1");
    let f2 = tmp("wait_f2");
    let src = format!(
        concat!(
            "set_workers(4)\n",
            "let p1 = sleep(0.01)\nlet p2 = sleep(0.02)\n",
            "on_ready(p1, fn(v) {{ await write_file(\"{}\", \"a\") }})\n",
            "on_ready(p2, fn(v) {{ await write_file(\"{}\", \"b\") }})\n",
            "event_loop()\n",
            "await read_file(\"{}\") + await read_file(\"{}\")"
        ),
        f1, f2, f1, f2
    );
    assert_eq!(run(&src).unwrap(), "ab");
}

// ── Chained on_ready (callback registers another on_ready) ────────────────────

#[test]
fn test_chained_on_ready_event_loop_continues() {
    let f = tmp("chained");
    let src = format!(
        concat!(
            "set_workers(2)\n",
            "let p1 = sleep(0.01)\n",
            "on_ready(p1, fn(v) {{\n",
            "    let p2 = sleep(0.01)\n",
            "    on_ready(p2, fn(v2) {{ await write_file(\"{}\", \"done\") }})\n",
            "}})\n",
            "event_loop()\n",
            "await read_file(\"{}\")"
        ),
        f, f
    );
    assert_eq!(run(&src).unwrap(), "done");
}

// ── read_file via event loop ──────────────────────────────────────────────────

#[test]
fn test_on_ready_read_file_async() {
    let src_f = tmp("rf_src");
    let out_f = tmp("rf_out");
    // write file synchronously (no pool), then async read it and capture result
    let src = format!(
        concat!(
            "write_file(\"{}\", \"hello async\")\n",
            "set_workers(2)\n",
            "let p = read_file(\"{}\")\n",
            "on_ready(p, fn(content) {{ await write_file(\"{}\", content) }})\n",
            "event_loop()\n",
            "await read_file(\"{}\")"
        ),
        src_f, src_f, out_f, out_f
    );
    assert_eq!(run(&src).unwrap(), "hello async");
}

// ── Arity errors ──────────────────────────────────────────────────────────────

#[test]
fn test_on_ready_too_few_args_errors() {
    assert!(run("on_ready(42)").is_err());
}

#[test]
fn test_on_ready_too_many_args_errors() {
    assert!(run("on_ready(42, fn(v) {}, 99)").is_err());
}

// ── event_loop timeout ────────────────────────────────────────────────────────

#[test]
fn test_event_loop_timeout_zero_exits_immediately() {
    // With timeout=0, exits immediately even if callbacks are pending
    let src = "set_workers(2)\nlet p = sleep(10.0)\non_ready(p, fn(v) {})\nevent_loop(0)";
    assert_eq!(run(src).unwrap(), "null");
}

#[test]
fn test_event_loop_timeout_sufficient_fires_callback() {
    let f = tmp("timeout_ok");
    let src = format!(
        "set_workers(2)\nlet p = sleep(0.01)\non_ready(p, fn(v) {{ await write_file(\"{}\", \"ok\") }})\nevent_loop(5.0)\nawait read_file(\"{}\")",
        f, f
    );
    assert_eq!(run(&src).unwrap(), "ok");
}

#[test]
fn test_event_loop_with_too_many_args_errors() {
    assert!(run("event_loop(1, 2)").is_err());
}

#[test]
fn test_event_loop_timeout_wrong_type_errors() {
    assert!(run("event_loop(\"bad\")").is_err());
}

// ── set_queue_limit backpressure ──────────────────────────────────────────────

#[test]
fn test_set_queue_limit_blocks_overflow() {
    // Limit to 1 entry; second on_ready should error
    let src = concat!(
        "set_workers(4)\n",
        "set_queue_limit(1)\n",
        "let p1 = sleep(10.0)\n",
        "let p2 = sleep(10.0)\n",
        "on_ready(p1, fn(v) {})\n",
        "on_ready(p2, fn(v) {})" // should error: queue full
    );
    assert!(run(src).is_err());
}

#[test]
fn test_set_queue_limit_allows_within_limit() {
    let f = tmp("ql_ok");
    let src = format!(
        concat!(
            "set_workers(2)\n",
            "set_queue_limit(5)\n",
            "let p = sleep(0.01)\n",
            "on_ready(p, fn(v) {{ await write_file(\"{}\", \"ok\") }})\n",
            "event_loop()\n",
            "await read_file(\"{}\")"
        ),
        f, f
    );
    assert_eq!(run(&src).unwrap(), "ok");
}

// ── error isolation ───────────────────────────────────────────────────────────

#[test]
fn test_callback_error_does_not_abort_remaining() {
    // First callback throws; second callback should still fire
    let f = tmp("iso_ok");
    let src = format!(
        concat!(
            "set_workers(4)\n",
            "let p1 = sleep(0.01)\n",
            "let p2 = sleep(0.01)\n",
            "on_ready(p1, fn(v) {{ throw \"deliberate\" }})\n",
            "on_ready(p2, fn(v) {{ await write_file(\"{}\", \"ran\") }})\n",
            "event_loop()\n",
            "await read_file(\"{}\")"
        ),
        f, f
    );
    assert_eq!(run(&src).unwrap(), "ran");
}
