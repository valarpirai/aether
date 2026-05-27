//! TCP server and client state types and event-driven I/O loops.
//!
//! A single background thread runs a mio `Poll` loop for each server or client.
//! All `TcpStream` handles live exclusively on that thread.  The main evaluator
//! thread communicates with the I/O thread via an mpsc channel of `TcpCommand`s
//! and wakes it immediately with a `mio::Waker`.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use mio::net::{TcpListener as MioListener, TcpStream as MioStream};
use mio::{Events, Interest, Poll, Token, Waker};

use super::value::Value;

// ---------------------------------------------------------------------------
// SIGINT handling
// ---------------------------------------------------------------------------

/// Counts SIGINT signals received.  1 = graceful shutdown; 2+ = force exit.
pub static SIGINT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn register_sigint_handler() {
    #[cfg(unix)]
    unsafe {
        extern "C" fn handler(_: i32) {
            SIGINT_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        extern "C" {
            fn signal(signum: i32, handler: unsafe extern "C" fn(i32)) -> usize;
        }
        signal(2, handler);
    }
}

pub fn graceful_shutdown_timeout_secs() -> u64 {
    std::env::var("AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

// ---------------------------------------------------------------------------
// mio token constants
// ---------------------------------------------------------------------------

pub const WAKER_TOKEN: Token = Token(0);
const LISTENER_TOKEN: Token = Token(1);
const CONN_BASE: usize = 2;

// ---------------------------------------------------------------------------
// Event / command types
// ---------------------------------------------------------------------------

/// Events sent from the I/O thread to the main evaluator thread.
pub enum TcpEvent {
    Connected { conn_id: u64, peer_addr: String },
    Message { conn_id: u64, data: Vec<u8> },
    Disconnected { conn_id: u64 },
    Error(String),
    Timeout { conn_id: u64 },
}

/// Commands sent from the main thread to the I/O thread.
pub enum TcpCommand {
    Write { conn_id: u64, data: Vec<u8> },
    CloseConn { conn_id: u64 },
    Shutdown,
}

// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

/// Mutable state for a `Value::TcpServer`.
pub struct TcpServerState {
    /// Bound listener; consumed by `run_tcp_server` when `accept()` is called.
    pub std_listener: Option<std::net::TcpListener>,
    /// Injected by `run_tcp_server`; used by `close()`.
    pub cmd_tx: Option<mpsc::Sender<TcpCommand>>,
    /// Injected by `run_tcp_server`; used to wake the I/O thread.
    pub waker: Option<Arc<Waker>>,
    pub shutdown: Arc<AtomicBool>,
    pub on_listen: Option<Value>,
    pub on_connect: Option<Value>,
    pub on_message: Option<Value>,
    pub on_disconnect: Option<Value>,
    pub on_error: Option<Value>,
    pub on_timeout: Option<Value>,
    pub delimiter: Option<String>,
    pub active_conns: HashMap<u64, Value>,
    pub closed: bool,
}

impl std::fmt::Debug for TcpServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TcpServerState(closed={})", self.closed)
    }
}

/// Mutable state for a `Value::TcpConnection`.
pub struct TcpConnectionState {
    pub conn_id: u64,
    pub addr: String,
    /// `true` for standalone client connections (`tcp_connect`); `false` for
    /// server-side connection objects created on accept.
    pub is_client: bool,
    /// Set after the I/O thread starts; used by `write()` / `close()`.
    pub cmd_tx: Option<mpsc::Sender<TcpCommand>>,
    pub waker: Option<Arc<Waker>>,
    pub shutdown: Arc<AtomicBool>,
    /// Client-side only.
    pub on_connect: Option<Value>,
    pub on_message: Option<Value>,
    pub on_disconnect: Option<Value>,
    pub on_error: Option<Value>,
    pub on_timeout: Option<Value>,
    pub closed: bool,
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

// ---------------------------------------------------------------------------
// Server I/O loop
// ---------------------------------------------------------------------------

/// Background I/O thread for a TCP server.
///
/// Owns the mio `Poll` and all `TcpStream` handles.  Accepts connections,
/// reads data, and processes commands (`Write`, `CloseConn`, `Shutdown`) from
/// the main thread.  Sends `TcpEvent`s back through `event_tx`.
pub fn run_server_io_loop(
    std_listener: std::net::TcpListener,
    event_tx: mpsc::Sender<TcpEvent>,
    cmd_rx: mpsc::Receiver<TcpCommand>,
    mut poll: Poll,
    delimiter: Option<String>,
    shutdown: Arc<AtomicBool>,
) {
    let mut listener = MioListener::from_std(std_listener);
    if let Err(e) = poll
        .registry()
        .register(&mut listener, LISTENER_TOKEN, Interest::READABLE)
    {
        let _ = event_tx.send(TcpEvent::Error(format!(
            "io-loop: register listener: {}",
            e
        )));
        return;
    }

    let delim = delimiter.map(|d| d.into_bytes());
    let mut streams: HashMap<u64, MioStream> = HashMap::new();
    let mut read_bufs: HashMap<u64, Vec<u8>> = HashMap::new();
    let mut next_conn_id: u64 = 0;
    let mut events = Events::with_capacity(256);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let _ = poll.poll(&mut events, Some(Duration::from_millis(50)));

        // Process commands before handling I/O events.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                TcpCommand::Write { conn_id, data } => {
                    if let Some(s) = streams.get_mut(&conn_id) {
                        let _ = s.write_all(&data);
                    }
                }
                TcpCommand::CloseConn { conn_id } => {
                    close_one(&mut streams, &mut read_bufs, poll.registry(), conn_id);
                    let _ = event_tx.send(TcpEvent::Disconnected { conn_id });
                }
                TcpCommand::Shutdown => {
                    shutdown.store(true, Ordering::Relaxed);
                }
            }
        }

        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        for event in events.iter() {
            match event.token() {
                WAKER_TOKEN => {}
                LISTENER_TOKEN => do_accept(
                    &mut listener,
                    &mut streams,
                    &mut read_bufs,
                    &mut next_conn_id,
                    poll.registry(),
                    &event_tx,
                ),
                token => {
                    let conn_id = (token.0 - CONN_BASE) as u64;
                    let closed_event = event.is_read_closed() || event.is_error();
                    let disc = read_data(
                        conn_id,
                        &mut streams,
                        &mut read_bufs,
                        &delim,
                        &event_tx,
                        closed_event,
                    );
                    if disc {
                        close_one(&mut streams, &mut read_bufs, poll.registry(), conn_id);
                        let _ = event_tx.send(TcpEvent::Disconnected { conn_id });
                    }
                }
            }
        }
    }

    // Graceful teardown: close all remaining streams.
    let ids: Vec<u64> = streams.keys().copied().collect();
    for conn_id in ids {
        close_one(&mut streams, &mut read_bufs, poll.registry(), conn_id);
        let _ = event_tx.send(TcpEvent::Disconnected { conn_id });
    }
    // Dropping event_tx here signals the main thread to exit its dispatch loop.
}

// ---------------------------------------------------------------------------
// Client I/O loop
// ---------------------------------------------------------------------------

/// Background I/O thread for a TCP client connection.
pub fn run_client_io_loop(
    addr: &str,
    event_tx: mpsc::Sender<TcpEvent>,
    cmd_rx: mpsc::Receiver<TcpCommand>,
    mut poll: Poll,
    shutdown: Arc<AtomicBool>,
) {
    const CLIENT_TOKEN: Token = Token(CONN_BASE);

    let sock_addr: std::net::SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = event_tx.send(TcpEvent::Error(format!("tcp_connect: invalid addr: {}", e)));
            return;
        }
    };

    let mut stream = match MioStream::connect(sock_addr) {
        Ok(s) => s,
        Err(e) => {
            let _ = event_tx.send(TcpEvent::Error(format!("tcp_connect: {}", e)));
            return;
        }
    };

    if let Err(e) = poll.registry().register(
        &mut stream,
        CLIENT_TOKEN,
        Interest::READABLE | Interest::WRITABLE,
    ) {
        let _ = event_tx.send(TcpEvent::Error(format!("tcp_connect: register: {}", e)));
        return;
    }

    let mut connected = false;
    let mut events = Events::with_capacity(32);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let _ = poll.poll(&mut events, Some(Duration::from_millis(50)));

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                TcpCommand::Write { data, .. } => {
                    let _ = stream.write_all(&data);
                }
                TcpCommand::CloseConn { .. } | TcpCommand::Shutdown => {
                    shutdown.store(true, Ordering::Relaxed);
                }
            }
        }

        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        for event in events.iter() {
            if event.token() == WAKER_TOKEN {
                continue;
            }
            if !connected {
                match stream.peer_addr() {
                    Ok(_) => {
                        connected = true;
                        let _ = poll.registry().reregister(
                            &mut stream,
                            CLIENT_TOKEN,
                            Interest::READABLE,
                        );
                        let _ = event_tx.send(TcpEvent::Connected {
                            conn_id: 0,
                            peer_addr: sock_addr.to_string(),
                        });
                    }
                    Err(e) => {
                        let _ = event_tx
                            .send(TcpEvent::Error(format!("tcp_connect: handshake: {}", e)));
                        return;
                    }
                }
            }
            if connected {
                if event.is_read_closed() || event.is_error() {
                    let _ = event_tx.send(TcpEvent::Disconnected { conn_id: 0 });
                    return;
                }
                if event.is_readable() {
                    let mut buf = [0u8; 4096];
                    loop {
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                let _ = event_tx.send(TcpEvent::Disconnected { conn_id: 0 });
                                return;
                            }
                            Ok(n) => {
                                let _ = event_tx.send(TcpEvent::Message {
                                    conn_id: 0,
                                    data: buf[..n].to_vec(),
                                });
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(_) => {
                                let _ = event_tx.send(TcpEvent::Disconnected { conn_id: 0 });
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers used by the I/O loops
// ---------------------------------------------------------------------------

fn do_accept(
    listener: &mut MioListener,
    streams: &mut HashMap<u64, MioStream>,
    read_bufs: &mut HashMap<u64, Vec<u8>>,
    next_id: &mut u64,
    registry: &mio::Registry,
    event_tx: &mpsc::Sender<TcpEvent>,
) {
    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                let conn_id = *next_id;
                *next_id += 1;
                let token = Token(CONN_BASE + conn_id as usize);
                if let Err(e) = registry.register(&mut stream, token, Interest::READABLE) {
                    let _ = event_tx.send(TcpEvent::Error(format!("register conn: {}", e)));
                    continue;
                }
                streams.insert(conn_id, stream);
                read_bufs.insert(conn_id, Vec::new());
                let _ = event_tx.send(TcpEvent::Connected {
                    conn_id,
                    peer_addr: addr.to_string(),
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                let _ = event_tx.send(TcpEvent::Error(e.to_string()));
                break;
            }
        }
    }
}

/// Read available data from a connection.  Returns `true` if the connection
/// should be closed (EOF, error, or the caller already signalled close).
fn read_data(
    conn_id: u64,
    streams: &mut HashMap<u64, MioStream>,
    read_bufs: &mut HashMap<u64, Vec<u8>>,
    delim: &Option<Vec<u8>>,
    event_tx: &mpsc::Sender<TcpEvent>,
    closed_event: bool,
) -> bool {
    if closed_event {
        return true;
    }
    let stream = match streams.get_mut(&conn_id) {
        Some(s) => s,
        None => return false,
    };
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => return true,
            Ok(n) => {
                let data = &buf[..n];
                match delim {
                    None => {
                        let _ = event_tx.send(TcpEvent::Message {
                            conn_id,
                            data: data.to_vec(),
                        });
                    }
                    Some(d) => {
                        let rb = read_bufs.entry(conn_id).or_default();
                        rb.extend_from_slice(data);
                        while let Some(pos) = rb.windows(d.len()).position(|w| w == d.as_slice()) {
                            let frame: Vec<u8> = rb.drain(..pos).collect();
                            rb.drain(..d.len());
                            let _ = event_tx.send(TcpEvent::Message {
                                conn_id,
                                data: frame,
                            });
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => return true,
        }
    }
    false
}

fn close_one(
    streams: &mut HashMap<u64, MioStream>,
    read_bufs: &mut HashMap<u64, Vec<u8>>,
    registry: &mio::Registry,
    conn_id: u64,
) {
    if let Some(mut s) = streams.remove(&conn_id) {
        let _ = registry.deregister(&mut s);
    }
    read_bufs.remove(&conn_id);
}
