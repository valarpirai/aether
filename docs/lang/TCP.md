# TCP

Aether provides built-in TCP server and client support via `tcp_listen` and `tcp_connect`.  
Both are event-driven: a single background I/O thread drives all connections via `epoll`/`kqueue` (mio), and events are dispatched to your Aether callbacks on the main thread.

---

## Server

### `tcp_listen(addr)` / `tcp_listen(addr, opts)`

Binds a TCP listener and returns a `tcp_server` object.

```aether
let server = tcp_listen("0.0.0.0:8080")
let server = tcp_listen("0.0.0.0:8080", { "delimiter": "\n" })
```

**`opts`** — optional dict:

| Key | Type | Effect |
|---|---|---|
| `"delimiter"` | string | Split incoming bytes on this string; `on_message` fires once per complete frame |

### Server lifecycle events

Register callbacks before calling `server.accept()`.

| Method | Callback signature | Fires when |
|---|---|---|
| `on_listen(fn() { })` | — | Server is bound and ready |
| `on_connect(fn(conn) { })` | `conn` = tcp_connection | A client connects |
| `on_message(fn(conn, data) { })` | `data` = array of ints | Data arrives from `conn` |
| `on_disconnect(fn(conn) { })` | `conn` = tcp_connection | Client closes the connection |
| `on_error(fn(err) { })` | `err` = string | An I/O error occurs |
| `on_timeout(fn() { })` | — | No activity within the timeout window |

### `server.accept()`

Starts the event loop. Blocks until `server.close()` is called or Ctrl+C is pressed.

### `server.close()`

Initiates graceful shutdown: stops accepting new connections, closes all active connections, exits the event loop.

### Echo server example

```aether
let server = tcp_listen("127.0.0.1:9000")

server.on_listen(fn() {
    print("listening on 9000")
})

server.on_connect(fn(conn) {
    conn.write("Welcome!\n")
})

// data is an array of byte ints [104, 101, 108, 108, 111, ...]
// conn.write accepts a string or a byte array
server.on_message(fn(conn, data) {
    conn.write(data)
})

server.on_disconnect(fn(conn) {
    print("client left")
})

server.accept()
```

---

## Connection object (`conn`)

Passed to `on_connect`, `on_message`, and `on_disconnect` callbacks.

| Method | Description |
|---|---|
| `conn.write(data)` | Send `data` to the client. Accepts a `string` or an `array` of ints (bytes). |
| `conn.close()` | Close this connection. Fires `on_disconnect`. |

---

## Client

### `tcp_connect(addr)`

Creates a client object targeting `addr`. No connection is made until `client.start()` is called.

```aether
let c = tcp_connect("127.0.0.1:9000")
```

### Client lifecycle events

| Method | Callback signature | Fires when |
|---|---|---|
| `on_connect(fn() { })` | — | Connection established |
| `on_message(fn(data) { })` | `data` = array of ints | Data arrives from server |
| `on_disconnect(fn() { })` | — | Server closes the connection |
| `on_error(fn(err) { })` | `err` = string | An I/O error occurs |
| `on_timeout(fn() { })` | — | No activity within the timeout window |

### `client.write(data)`

Send data to the server. Accepts a `string` or an `array` of ints (bytes).

### `client.start()`

Opens the connection and starts the event loop. Blocks until `client.close()` is called or the server disconnects.

### `client.close()`

Closes the connection and exits the event loop.

### Client example

```aether
let sent = [0]
let msgs = ["hello\n", "world\n"]

let c = tcp_connect("127.0.0.1:9000")

c.on_connect(fn() {
    c.write(msgs[sent[0]])
    sent[0] = sent[0] + 1
})

c.on_message(fn(data) {
    if (sent[0] < len(msgs)) {
        c.write(msgs[sent[0]])
        sent[0] = sent[0] + 1
    } else {
        c.close()
    }
})

c.start()
```

---

## Mutable state in callbacks

Aether closures capture variables **by copy**, not by shared reference.  
Reassigning a primitive inside a callback does **not** persist across calls:

```aether
let count = 0
server.on_message(fn(conn, data) {
    count = count + 1   // ← modifies a per-call copy; count stays 0 externally
})
```

Use an **array or dict** instead — these carry reference semantics through `Rc`:

```aether
let count = [0]
server.on_message(fn(conn, data) {
    count[0] = count[0] + 1   // ← mutates the shared array; persists across calls
    if (count[0] >= 10) {
        server.close()
    }
})
```

The same pattern applies to any shared list (e.g. connected clients in a chat server).

---

## Signal handling (Ctrl+C)

`server.accept()` / `client.start()` register a SIGINT handler automatically.

| Press | Behaviour |
|---|---|
| First Ctrl+C | Graceful shutdown: stop accepting, drain existing connections up to `AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS` (default 5 s) |
| Second Ctrl+C | Force exit |

```bash
AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS=30 aether server.ae
```

---

## Architecture

```
Aether script (main thread)           I/O thread (mio Poll)
────────────────────────────          ──────────────────────
server.accept()                       run_server_io_loop()
│                                     │
│   recv_timeout(10ms) ◄─ event_tx ──┤  accept / read / write
│                                     │  streams: HashMap<id, TcpStream>
│   dispatch callback                 │
│     conn.write(data)                │
│       cmd_tx.send(Write) ──────────►│  write_all(data)
│       waker.wake()      ──────────►│  (wakes immediately)
│                                     │
│   server.close()                    │
│     cmd_tx.send(Shutdown) ─────────►│  close all → drop event_tx
│     waker.wake()          ─────────►│
│   recv_timeout → Disconnected       │
│   loop exits                        │
```

All `TcpStream` handles live exclusively on the I/O thread.  
`conn.write()` and `server.close()` send commands through an `mpsc` channel; a `mio::Waker` wakes the I/O thread immediately rather than waiting for the next poll tick (≤ 50 ms).

---

## Memory per idle connection

| Resource | Cost |
|---|---|
| `TcpConnectionState` (Aether heap) | ~390 B |
| Read buffer (I/O thread heap) | 0–4 KB (grows to frame size) |
| Kernel socket buffers | 8–256 KB (OS-controlled) |
| **Total** | **~8–260 KB** |

The I/O thread is **shared** across all connections (unlike the old one-thread-per-connection model that cost ~8 MB per connection). Practical limit is ~100 k connections per GB of RAM (kernel buffers dominate).

---

## See also

- [ASYNC.md](ASYNC.md) — async/await and the I/O thread pool
- [EVENT_LOOP.md](../dev/EVENT_LOOP.md) — event loop internals
- `examples/tcp_server_demo.ae` — echo server with all lifecycle events
- `examples/tcp_client_demo.ae` — client that sends two messages and disconnects
- `examples/tcp_chat_server_demo.ae` — broadcast chat server with delimiter framing
