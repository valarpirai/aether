//! TCP server and client state types for Aether's network builtins.

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::value::Value;

/// Counts SIGINT signals received. 1 = graceful shutdown; 2+ = force exit.
pub static SIGINT_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Register a signal handler that increments SIGINT_COUNT on Ctrl+C.
/// Safe to call multiple times (idempotent after the first call).
pub fn register_sigint_handler() {
    #[cfg(unix)]
    unsafe {
        extern "C" fn handler(_: i32) {
            SIGINT_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        // SIGINT = 2 on all POSIX platforms
        extern "C" {
            fn signal(signum: i32, handler: unsafe extern "C" fn(i32)) -> usize;
        }
        signal(2, handler);
    }
}

/// Read the graceful shutdown timeout from the environment (default 5 s).
pub fn graceful_shutdown_timeout_secs() -> u64 {
    std::env::var("AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

/// Events sent from background I/O threads to the main evaluator thread.
pub enum TcpEvent {
    /// A new client connected (server-side accept).
    Connected {
        conn_id: u64,
        stream: Arc<TcpStream>,
        peer_addr: String,
    },
    /// Data received on an existing connection.
    Message { conn_id: u64, data: Vec<u8> },
    /// A connection was closed by the remote end.
    Disconnected { conn_id: u64 },
    /// An I/O error occurred.
    Error(String),
    /// A per-connection timeout fired (no data within the timeout window).
    Timeout { conn_id: u64 },
}

/// Mutable state for a `Value::TcpServer`.
pub struct TcpServerState {
    pub listener: Arc<std::net::TcpListener>,
    // (Debug is implemented manually below to avoid requiring Value: Debug)
    pub event_tx: std::sync::mpsc::Sender<TcpEvent>,
    pub event_rx: std::sync::mpsc::Receiver<TcpEvent>,
    pub on_listen: Option<Value>,
    pub on_connect: Option<Value>,
    pub on_message: Option<Value>,
    pub on_disconnect: Option<Value>,
    pub on_error: Option<Value>,
    pub on_timeout: Option<Value>,
    /// Optional line delimiter for framed message mode (e.g. `"\n"`).
    pub delimiter: Option<String>,
    /// Per-connection objects keyed by ID — passed to callbacks as the `conn` argument.
    pub active_conns: HashMap<u64, Value>,
    /// Set to true when `server.close()` is called.
    pub closed: bool,
    pub next_conn_id: u64,
    /// Shared flag — background threads check this to stop when set.
    pub shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for TcpServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TcpServerState(closed={})", self.closed)
    }
}

/// Mutable state for a `Value::TcpConnection`.
pub struct TcpConnectionState {
    /// Remote address (peer for server-side conns; target for client-side conns).
    pub addr: String,
    /// Write stream; `None` before `conn.start()` is called on a client connection.
    pub stream: Option<Arc<TcpStream>>,
    pub event_tx: std::sync::mpsc::Sender<TcpEvent>,
    pub event_rx: std::sync::mpsc::Receiver<TcpEvent>,
    pub on_connect: Option<Value>,
    pub on_message: Option<Value>,
    pub on_disconnect: Option<Value>,
    pub on_error: Option<Value>,
    pub on_timeout: Option<Value>,
    /// Set to true when `conn.close()` is called.
    pub closed: bool,
    /// Shared flag — the reader thread checks this to stop when set.
    pub shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for TcpConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TcpConnectionState(addr={}, closed={})",
            self.addr, self.closed
        )
    }
}
