//! Tests for UDP socket builtin

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

fn spawn_udp(source: String) -> std::thread::JoinHandle<Result<String, String>> {
    std::thread::spawn(move || run(&source).map(|v| format!("{}", v)))
}

fn free_port() -> u16 {
    let l = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

// --- udp_bind ---

#[test]
fn test_udp_bind_returns_socket() {
    let result = run(r#"type(udp_bind("127.0.0.1:0"))"#);
    assert_eq!(result, Ok(Value::string("udp_socket")));
}

#[test]
fn test_udp_bind_invalid_addr_errors() {
    let result = run(r#"udp_bind("not_an_address")"#);
    assert!(result.is_err());
}

#[test]
fn test_udp_bind_wrong_arg_count() {
    let result = run(r#"udp_bind()"#);
    assert!(result.is_err());
}

// --- lifecycle event registration ---

#[test]
fn test_udp_on_message_registration() {
    let result = run(r#"
let s = udp_bind("127.0.0.1:0")
s.on_message(fn(data, addr) { })
type(s)
"#);
    assert_eq!(result, Ok(Value::string("udp_socket")));
}

#[test]
fn test_udp_on_message_requires_fn() {
    let result = run(r#"
let s = udp_bind("127.0.0.1:0")
s.on_message("not a function")
"#);
    assert!(result.is_err());
}

// --- close ---

#[test]
fn test_udp_close_before_listen() {
    let result = run(r#"
let s = udp_bind("127.0.0.1:0")
s.close()
type(s)
"#);
    assert_eq!(result, Ok(Value::string("udp_socket")));
}

// --- integration: echo server ---

// Binds a UDP socket, sends one datagram, expects it echoed back.
#[test]
fn test_udp_echo() {
    use std::net::UdpSocket;

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    let server_thread = spawn_udp(format!(
        r#"
let sock = udp_bind("{addr}")
sock.on_message(fn(data, addr) {{
    sock.send_to(data, addr)
    sock.close()
}})
sock.listen()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    client.send_to(b"hello-udp", &addr_clone).unwrap();

    let mut buf = [0u8; 9];
    let (n, _) = client.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"hello-udp");

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "udp echo test failed: {:?}", result);
}

// on_message callback receives addr as a string (host:port).
#[test]
fn test_udp_on_message_addr_is_string() {
    use std::net::UdpSocket;

    let port = free_port();
    let addr = format!("127.0.0.1:{}", port);
    let addr_clone = addr.clone();

    // Server sends the received addr string back to the client as the reply.
    let server_thread = spawn_udp(format!(
        r#"
let sock = udp_bind("{addr}")
sock.on_message(fn(data, addr) {{
    sock.send_to(addr, addr)
    sock.close()
}})
sock.listen()
"#
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let client_port = client.local_addr().unwrap().port();
    client
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    client.send_to(b"x", &addr_clone).unwrap();

    let mut buf = [0u8; 64];
    let (n, _) = client.recv_from(&mut buf).unwrap();
    let received = std::str::from_utf8(&buf[..n]).unwrap();
    // addr returned should contain the client's port
    assert!(
        received.contains(&client_port.to_string()),
        "expected addr with port {}, got '{}'",
        client_port,
        received
    );

    let result = server_thread.join().expect("server thread panicked");
    assert!(result.is_ok(), "addr test failed: {:?}", result);
}
