---
layout: default
title: "Aether — TCP/UDP Internals"
---

[Home](../index.html) › Developer Docs › TCP/UDP Internals

# TCP/UDP Internals

This document describes the internal implementation of Aether's TCP networking layer.  
For the user-facing API, see [docs/lang/TCP.md](../lang/TCP.md).

---

## Source files

| File | Purpose |
|---|---|
| `src/interpreter/tcp.rs` | State types, command/event enums, mio I/O loops |
| `src/interpreter/builtins.rs` | `builtin_tcp_listen`, `builtin_tcp_connect` |
| `src/interpreter/evaluator/functions.rs` | `run_tcp_server`, `run_tcp_client`, dispatch loops |
| `src/interpreter/evaluator/members.rs` | Method dispatch: `accept`, `close`, `write`, `start`, lifecycle callbacks |
| `src/interpreter/value.rs` | `Value::TcpServer`, `Value::TcpConnection` variants |
| `tests/tcp_test.rs` | 27 integration tests |

---

## Architecture

Two threads per server or client. The Aether evaluator (main thread) never touches a socket directly.

```
Main thread (Aether evaluator)           I/O thread (mio Poll)
─────────────────────────────────        ─────────────────────────────
server.accept() / client.start()         run_server_io_loop()
│                                        │  or run_client_io_loop()
│  recv_timeout(10ms)                    │
│    ◄─── event_tx (TcpEvent) ──────────┤  accept / read
│                                        │  streams: HashMap<u64, MioStream>
│  dispatch callback                     │  read_bufs: HashMap<u64, Vec<u8>>
│    conn.write(data)                    │
│      cmd_tx.send(Write) ─────────────►│  write_all(data)
│      waker.wake()        ─────────────►│  (returns from poll immediately)
│                                        │
│  server.close()                        │
│    cmd_tx.send(Shutdown) ─────────────►│  close all streams
│    waker.wake()           ─────────────►│  drop event_tx → main loop exits
│  recv_timeout → Disconnected           │
│  loop exits                            │
```

All `TcpStream` handles live exclusively on the I/O thread. The main thread
communicates only through two channels and a `mio::Waker`.

---

## State types (`src/interpreter/tcp.rs`)

### `TcpServerState`

Stored inside `Value::TcpServer(Rc<RefCell<TcpServerState>>)`.

```rust
pub struct TcpServerState {
    pub std_listener: Option<std::net::TcpListener>, // consumed by accept()
    pub cmd_tx:  Option<Sender<TcpCommand>>,          // injected by accept()
    pub waker:   Option<Arc<mio::Waker>>,             // injected by accept()
    pub shutdown: Arc<AtomicBool>,                    // shared with I/O thread
    pub on_listen/connect/message/disconnect/error/timeout: Option<Value>,
    pub delimiter: Option<String>,
    pub active_conns: HashMap<u64, Value>,  // Value::TcpConnection per conn
    pub closed: bool,
}
```

`std_listener` is created eagerly in `builtin_tcp_listen` (port is bound immediately).  
`cmd_tx` and `waker` are injected lazily when `server.accept()` is called, because
`mio::Poll` and `mio::Waker` must be created together and the poll is moved into the I/O thread.

### `TcpConnectionState`

Stored inside `Value::TcpConnection(Rc<RefCell<TcpConnectionState>>)`.  
Used for both server-side accepted connections and standalone client connections.

```rust
pub struct TcpConnectionState {
    pub conn_id:  u64,
    pub addr:     String,
    pub is_client: bool,      // true = tcp_connect; false = server-accepted
    pub cmd_tx:   Option<Sender<TcpCommand>>,
    pub waker:    Option<Arc<mio::Waker>>,
    pub shutdown: Arc<AtomicBool>,
    // client-side only:
    pub on_connect/message/disconnect/error/timeout: Option<Value>,
    pub closed: bool,
}
```

Server-side connection objects share the server's `cmd_tx` and `waker` (cloned from
`TcpServerState` when the `Connected` event is dispatched). They have no independent
I/O thread — the server's single I/O thread handles all connections.

---

## mio token scheme

```
Token(0)       WAKER_TOKEN    — mio::Waker (wakes poll from main thread)
Token(1)       LISTENER_TOKEN — TcpListener (server only)
Token(2 + id)  connection     — TcpStream with conn_id = id
```

`conn_id` is a monotonically increasing `u64` per server instance.
Tokens are never reused within a server's lifetime — a disconnected connection's
token is deregistered and never reassigned.

---

## Command / event channels

### `TcpCommand` (main → I/O thread)

```rust
pub enum TcpCommand {
    Write { conn_id: u64, data: Vec<u8> },  // write bytes to conn
    CloseConn { conn_id: u64 },              // close one connection
    Shutdown,                                // stop the I/O loop
}
```

Every `cmd_tx.send(...)` is followed immediately by `waker.wake()` so the I/O thread
returns from `poll.poll(50ms)` without delay.

### `TcpEvent` (I/O thread → main)

```rust
pub enum TcpEvent {
    Connected    { conn_id: u64, peer_addr: String },
    Message      { conn_id: u64, data: Vec<u8> },
    Disconnected { conn_id: u64 },
    Error(String),
    Timeout      { conn_id: u64 },
}
```

When the I/O thread exits (on `Shutdown` or error), it drops `event_tx`.
The main thread's `recv_timeout` then returns `RecvTimeoutError::Disconnected`,
which is the signal to exit the dispatch loop.

---

## I/O thread loop (`run_server_io_loop`)

```
loop:
  if shutdown { break }
  poll.poll(events, 50ms timeout)   ← waker.wake() interrupts this
  drain cmd_rx:
    Write      → stream.write_all(data)
    CloseConn  → deregister + remove stream, send Disconnected event
    Shutdown   → set shutdown = true
  if shutdown { break }
  for event in events:
    WAKER_TOKEN    → no-op (already drained cmd_rx)
    LISTENER_TOKEN → accept loop (do_accept)
    conn token     → read_data or close on EOF/error

cleanup:
  for each remaining stream: close, send Disconnected
  return  ← drops event_tx, signalling main thread to exit
```

Commands are processed **before** I/O events in each iteration, guaranteeing
that a `Write` followed by a `Shutdown` in the same batch always writes first.

---

## Delimiter framing

When `tcp_listen(addr, { "delimiter": "\n" })` is used, the I/O thread maintains
a per-connection `read_bufs: HashMap<u64, Vec<u8>>`. Incoming bytes are appended
to the buffer; complete frames (terminated by the delimiter bytes) are extracted
and sent as individual `Message` events. Partial frames are held until the next read.

Without a delimiter, each `read()` call produces one `Message` event (raw bytes).

---

## SIGINT handling (`src/interpreter/tcp.rs`)

```rust
pub static SIGINT_COUNT: AtomicUsize = AtomicUsize::new(0);
```

`register_sigint_handler()` installs a raw C signal handler (no external crate)
that increments `SIGINT_COUNT`. The main dispatch loop checks this counter:

| Count | Action |
|---|---|
| 0 | Normal operation |
| 1 | Graceful shutdown: send `Shutdown`, drain events up to `AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS` |
| ≥ 2 | Force exit: break immediately |

---

## Closure variable capture caveat

Aether closures capture the environment by **deep copy** (`Rc::new(self.environment.clone())`).
Primitive values (`Int`, `Float`, `String`, `Bool`) are per-call copies — mutations inside
a callback do not persist across calls.

Reference types (`Array`, `Dict`, `TcpServer`, `TcpConnection`) carry an `Rc` pointer,
so mutations through them (method calls, index assignment) are visible across calls.

**Pattern for mutable closure state:**

```aether
// WRONG — count resets to 0 on every call
let count = 0
server.on_message(fn(conn, data) {
    count = count + 1   // modifies a per-call copy
})

// CORRECT — array shares state via Rc
let count = [0]
server.on_message(fn(conn, data) {
    count[0] = count[0] + 1   // mutates the shared array
})
```

This affects any Aether callback (TCP, event loop, async `.then()`), not just TCP.

---

## Memory per idle connection

| Resource | Cost |
|---|---|
| `TcpConnectionState` (Aether heap) | ~390 B |
| Read buffer (I/O thread heap) | 0–4 KB (grows to frame size) |
| Kernel socket send/recv buffers | 8–256 KB (OS-controlled) |
| I/O thread stack | shared — 0 extra per connection |
| **Total** | **~8–260 KB** |

The I/O thread is shared across all connections (unlike the old one-thread-per-connection
model that cost ~8 MB per connection due to the OS thread stack).  
Practical upper bound: ~100 k simultaneous connections per GB of RAM
(kernel buffers dominate at that scale).

---

## Adding a new server or client event

1. Add the variant to `TcpEvent` in `src/interpreter/tcp.rs`.
2. Send it from the appropriate place in `run_server_io_loop` or `run_client_io_loop`.
3. Add a match arm in `dispatch_tcp_server_event` or `dispatch_tcp_client_event`
   in `src/interpreter/evaluator/functions.rs`.
4. Add the corresponding `on_<event>` method handler in `eval_method_call` in
   `src/interpreter/evaluator/members.rs`.
5. Add tests in `tests/tcp_test.rs`.

---

## UDP — design (not yet implemented)

UDP support was designed alongside TCP. Key differences from TCP:

- **Connectionless**: no accept loop, no per-connection state.
- **Datagrams**: each `send_to` / `on_message` is one self-contained packet.
- **No ordering or reliability** guarantees.

Planned API:

```aether
let sock = udp_bind("0.0.0.0:9000")

sock.on_message(fn(data, addr) {
    sock.send_to(data, addr)   // echo back to sender
})

sock.listen()    // enters event loop
sock.close()
```

Implementation plan:
- Add `Value::UdpSocket(Rc<RefCell<UdpSocketState>>)` to `value.rs`.
- Add `builtin_udp_bind(args)` to `builtins.rs`.
- Add `run_udp_io_loop` to `tcp.rs` (or a new `udp.rs`).
- Use `mio::net::UdpSocket` registered with `Interest::READABLE`.
- `send_to` is a `TcpCommand::SendTo { addr, data }` sent to the I/O thread
  (same channel pattern as TCP writes).
- No delimiter framing (datagrams are already message-bounded).

---

## See also

- [docs/lang/TCP.md](../lang/TCP.md) — user-facing API reference
- [EVENT_LOOP.md](EVENT_LOOP.md) — async event loop internals
- [INTERPRETER.md](INTERPRETER.md) — evaluator sub-module layout
- [MEMORY_MANAGEMENT.md](MEMORY_MANAGEMENT.md) — Rc/RefCell rules and cycle prevention
