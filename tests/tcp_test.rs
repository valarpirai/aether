//! Tests for TCP server and client builtins

use aether_lang::interpreter::value::Value;
use aether_lang::interpreter::Evaluator;
use aether_lang::lexer::Scanner;
use aether_lang::parser::Parser;

fn run(source: &str) -> Result<Value, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut evaluator = Evaluator::new_without_stdlib();
    let stmts = &program.statements;
    if stmts.is_empty() {
        return Ok(Value::Null);
    }
    for stmt in &stmts[..stmts.len() - 1] {
        evaluator.exec_stmt(stmt).map_err(|e| e.to_string())?;
    }
    let last = stmts.last().unwrap();
    if let aether_lang::parser::ast::Stmt::Expr(expr) = last {
        return evaluator.eval_expr(expr).map_err(|e| e.to_string());
    }
    evaluator.exec_stmt(last).map_err(|e| e.to_string())?;
    Ok(Value::Null)
}

// --- tcp_listen ---

#[test]
fn test_tcp_listen_returns_server() {
    // tcp_listen binds a port and returns a tcp_server value
    let result = run(r#"type(tcp_listen("127.0.0.1:0"))"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_tcp_listen_with_delimiter() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0", { "delimiter": "\n" })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_tcp_listen_invalid_addr_errors() {
    let result = run(r#"tcp_listen("not_an_address")"#);
    assert!(result.is_err());
}

#[test]
fn test_tcp_listen_wrong_arg_count() {
    let result = run(r#"tcp_listen()"#);
    assert!(result.is_err());
}

// --- tcp_connect ---

#[test]
fn test_tcp_connect_returns_connection() {
    let result = run(r#"type(tcp_connect("127.0.0.1:9"))"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

#[test]
fn test_tcp_connect_wrong_arg_count() {
    let result = run(r#"tcp_connect()"#);
    assert!(result.is_err());
}

// --- server lifecycle method registration ---

#[test]
fn test_server_on_listen_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_listen(fn() { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_server_on_connect_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_connect(fn(conn) { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_server_on_message_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_message(fn(conn, data) { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_server_on_disconnect_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_disconnect(fn(conn) { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_server_on_error_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_error(fn(err) { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

#[test]
fn test_server_on_timeout_registration() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_timeout(fn() { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

// --- server.close() ---

#[test]
fn test_server_close() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.close()
type(s)
"#);
    assert_eq!(result, Ok(Value::string("tcp_server")));
}

// --- client lifecycle method registration ---

#[test]
fn test_client_on_connect_registration() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_connect(fn() { })
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

#[test]
fn test_client_on_message_registration() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_message(fn(data) { })
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

#[test]
fn test_client_on_disconnect_registration() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_disconnect(fn() { })
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

#[test]
fn test_client_on_error_registration() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_error(fn(err) { })
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

#[test]
fn test_client_on_timeout_registration() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_timeout(fn() { })
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

// --- client.close() ---

#[test]
fn test_client_close() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.close()
type(c)
"#);
    assert_eq!(result, Ok(Value::string("tcp_connection")));
}

// --- method arg validation ---

#[test]
fn test_server_on_connect_requires_fn() {
    let result = run(r#"
let s = tcp_listen("127.0.0.1:0")
s.on_connect("not a function")
"#);
    assert!(result.is_err());
}

#[test]
fn test_client_on_message_requires_fn() {
    let result = run(r#"
let c = tcp_connect("127.0.0.1:9")
c.on_message(42)
"#);
    assert!(result.is_err());
}

// ---- helpers for integration tests ----

fn free_port() -> u16 {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    drop(l);
    port
}

fn spawn_server(source: String) -> std::thread::JoinHandle<Result<String, String>> {
    std::thread::spawn(move || run(&source).map(|v| format!("{}", v)))
}

// --- echo server integration test (uses threads) ---
//
// Binds an ephemeral port, exchanges one message, then closes.
// Verifies that the server echoes bytes back to the client.
#[test]
fn test_echo_server_one_message() {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    let server_thread = spawn_server(format!(
        r#"
let server = tcp_listen("{addr}")
server.on_message(fn(conn, data) {{
    conn.write(data)
    server.close()
}})
server.accept()
"#
    ));

    // Give server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Client: connect, send, read echo
    let mut stream = TcpStream::connect(&addr_clone).unwrap();
    stream.write_all(b"hello").unwrap();

    let mut buf = [0u8; 5];
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello");

    // Server should finish cleanly after close()
    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "server error: {:?}", result);
}

// on_listen fires before the first accept
#[test]
fn test_on_listen_fires() {
    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);

    // We verify on_listen via a side-channel: write_bytes is not available so we
    // use the server closing immediately after on_listen as a proxy.
    let server_thread = spawn_server(format!(
        r#"
let server = tcp_listen("{addr}")
server.on_listen(fn() {{
    server.close()
}})
server.accept()
"#
    ));

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "on_listen test failed: {:?}", result);
}

// on_connect fires when a client connects
#[test]
fn test_on_connect_fires() {
    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    let server_thread = spawn_server(format!(
        r#"
let server = tcp_listen("{addr}")
server.on_connect(fn(conn) {{
    server.close()
}})
server.accept()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    // Connecting triggers on_connect which calls server.close()
    let _ = std::net::TcpStream::connect(&addr_clone);

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "on_connect test failed: {:?}", result);
}

// on_disconnect fires when the client closes the connection
#[test]
fn test_on_disconnect_fires() {
    use std::io::{Read, Write};

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    let server_thread = spawn_server(format!(
        r#"
let server = tcp_listen("{addr}")
server.on_connect(fn(conn) {{
    conn.write("ready")
}})
server.on_disconnect(fn(conn) {{
    server.close()
}})
server.accept()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut stream = std::net::TcpStream::connect(&addr_clone).unwrap();
    let mut buf = [0u8; 5];
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"ready");
    // Drop stream → server detects disconnect → on_disconnect → server.close()
    drop(stream);

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "on_disconnect test failed: {:?}", result);
}

// Multiple sequential connections: each client connects, sends one message, reads echo.
// This avoids TCP coalescing ambiguity while still testing multi-client handling.
// Marked ignore: slow due to accept-thread 10ms poll + on_disconnect processing delay.
// Run explicitly with: cargo test test_multiple_sequential_connections -- --ignored
#[test]
#[ignore]
fn test_multiple_sequential_connections() {
    use std::io::{Read, Write};

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    // Server echoes each message and counts; closes after 3 connections
    let server_thread = spawn_server(format!(
        r#"
let count = 0
let server = tcp_listen("{addr}")
server.on_message(fn(conn, data) {{
    conn.write(data)
    count = count + 1
    if (count >= 3) {{
        server.close()
    }}
}})
server.accept()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));

    for msg in &[b"one" as &[u8], b"two", b"thr"] {
        let mut stream = std::net::TcpStream::connect(&addr_clone).unwrap();
        stream.set_nodelay(true).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        stream.write_all(msg).unwrap();
        let mut buf = vec![0u8; msg.len()];
        stream.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, *msg, "echo mismatch for {:?}", msg);
        drop(stream); // close before next connection
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let result = server_thread.join().expect("server thread panicked");
    assert!(
        result.is_ok(),
        "sequential connections test failed: {:?}",
        result
    );
}

// conn.write accepts a string directly
#[test]
fn test_conn_write_string() {
    #[allow(unused_imports)]
    use std::io::{Read, Write};

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    let server_thread = spawn_server(format!(
        r#"
let server = tcp_listen("{addr}")
server.on_connect(fn(conn) {{
    conn.write("hello from server")
    server.close()
}})
server.accept()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut stream = std::net::TcpStream::connect(&addr_clone).unwrap();
    let mut buf = [0u8; 17];
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello from server");

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "write string test failed: {:?}", result);
}
