//! UDP socket state type and event-driven I/O loop.
//!
//! A single background thread drives the mio Poll loop.
//! The main evaluator thread communicates via mpsc + mio::Waker.

use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use mio::net::UdpSocket as MioUdp;
use mio::{Events, Interest, Poll, Token, Waker};

use super::value::Value;

pub const WAKER_TOKEN: Token = Token(0);
const SOCKET_TOKEN: Token = Token(1);

// ---------------------------------------------------------------------------
// Event / command types
// ---------------------------------------------------------------------------

/// Events sent from the I/O thread to the main evaluator thread.
pub enum UdpEvent {
    Message { data: Vec<u8>, addr: String },
    Error(String),
}

/// Commands sent from the main thread to the I/O thread.
pub enum UdpCommand {
    SendTo { addr: String, data: Vec<u8> },
    Shutdown,
}

// ---------------------------------------------------------------------------
// State type
// ---------------------------------------------------------------------------

/// Mutable state for a `Value::UdpSocket`.
pub struct UdpSocketState {
    /// Bound socket; consumed by `run_udp_socket` when `listen()` is called.
    pub std_socket: Option<std::net::UdpSocket>,
    /// Injected by `run_udp_socket`; used by `send_to()` / `close()`.
    pub cmd_tx: Option<mpsc::Sender<UdpCommand>>,
    pub waker: Option<Arc<Waker>>,
    pub shutdown: Arc<AtomicBool>,
    pub on_message: Option<Value>,
    pub closed: bool,
}

impl std::fmt::Debug for UdpSocketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UdpSocketState(closed={})", self.closed)
    }
}

// ---------------------------------------------------------------------------
// I/O loop
// ---------------------------------------------------------------------------

/// Background I/O thread for a UDP socket.
///
/// Owns the mio `Poll` and the `UdpSocket`.  Receives datagrams and
/// processes commands (`SendTo`, `Shutdown`) from the main thread.
pub fn run_udp_io_loop(
    std_socket: std::net::UdpSocket,
    event_tx: mpsc::Sender<UdpEvent>,
    cmd_rx: mpsc::Receiver<UdpCommand>,
    mut poll: Poll,
    shutdown: Arc<AtomicBool>,
) {
    let mut socket = MioUdp::from_std(std_socket);
    if let Err(e) = poll
        .registry()
        .register(&mut socket, SOCKET_TOKEN, Interest::READABLE)
    {
        let _ = event_tx.send(UdpEvent::Error(format!("udp io-loop: register: {}", e)));
        return;
    }

    let mut buf = vec![0u8; 65535];
    let mut events = Events::with_capacity(64);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let _ = poll.poll(&mut events, Some(Duration::from_millis(50)));

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                UdpCommand::SendTo { addr, data } => match addr.parse::<SocketAddr>() {
                    Ok(target) => {
                        let _ = socket.send_to(&data, target);
                    }
                    Err(_) => {
                        let _ = event_tx.send(UdpEvent::Error(format!(
                            "send_to: invalid address '{}'",
                            addr
                        )));
                    }
                },
                UdpCommand::Shutdown => {
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
            if event.is_readable() {
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, addr)) => {
                            let _ = event_tx.send(UdpEvent::Message {
                                data: buf[..n].to_vec(),
                                addr: addr.to_string(),
                            });
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            let _ = event_tx.send(UdpEvent::Error(e.to_string()));
                            break;
                        }
                    }
                }
            }
        }
    }
}
