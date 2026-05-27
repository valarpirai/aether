# TCP

Aether provides built-in TCP server and client support via `tcp_listen` and `tcp_connect`.

## Server

### `tcp_listen(addr)` / `tcp_listen(addr, opts)`

Binds a TCP listener on `addr` and returns a `tcp_server` object.

```aether
let server = tcp_listen("0.0.0.0:8080")
let server = tcp_listen("0.0.0.0:8080", { "delimiter": "\n" })
```

`opts` is an optional dict. Supported keys:

| Key | Type | Description |
|---|---|---|
| `"delimiter"` | string | If set, the reader splits incoming bytes on this string and fires `on_message` once per frame |

### Server lifecycle methods

Register callbacks before calling `server.accept()`.

| Method | Signature | Fires when |
|---|---|---|
| `on_listen(fn() { })` | no args | Server is bound and ready to accept connections |
| `on_connect(fn(conn) { })` | `conn` = tcp_connection | A client connects |
| `on_message(fn(conn, data) { })` | `data` = array of ints (bytes) | A message arrives on `conn` |
| `on_disconnect(fn(conn) { })` | `conn` = tcp_connection | Client closes the connection |
| `on_error(fn(err) { })` | `err` = string | An I/O error occurs |
| `on_timeout(fn() { })` | no args | No activity within the timeout window |

### `server.accept()`

Starts the event loop. Blocks until `server.close()` is called (or Ctrl+C).

### `server.close()`

Signals a graceful shutdown: stops accepting new connections, sends FIN to all active connections, and exits the event loop.

### Example — echo server

```aether
let server = tcp_listen("127.0.0.1:9000")

server.on_listen(fn() {
    print("listening on 9000")
})

server.on_connect(fn(conn) {
    print("client connected")
})

server.on_message(fn(conn, data) {
    conn.write(data)
})

server.on_disconnect(fn(conn) {
    print("client disconnected")
})

server.accept()
```

---

## Connection object (`conn`)

`conn` is passed to `on_connect`, `on_message`, and `on_disconnect` callbacks.

| Method | Description |
|---|---|
| `conn.write(data)` | Send `data` to the client. Accepts a `string` or an `array` of ints (bytes). |
| `conn.close()` | Close this connection. |

---

## Client

### `tcp_connect(addr)`

Creates a TCP client object targeting `addr`. The connection is not opened until `client.start()` is called.

```aether
let c = tcp_connect("127.0.0.1:9000")
```

### Client lifecycle methods

| Method | Signature | Fires when |
|---|---|---|
| `on_connect(fn() { })` | no args | Connection established |
| `on_message(fn(data) { })` | `data` = array of ints | Data arrives |
| `on_disconnect(fn() { })` | no args | Server closes the connection |
| `on_error(fn(err) { })` | `err` = string | An I/O error occurs |
| `on_timeout(fn() { })` | no args | No activity within the timeout window |

### `client.write(data)`

Send data to the server. Accepts a `string` or an `array` of ints (bytes).

### `client.start()`

Opens the connection and starts the event loop. Blocks until `client.close()` is called.

### `client.close()`

Closes the connection and exits the event loop.

### Example — echo client

```aether
let c = tcp_connect("127.0.0.1:9000")

c.on_connect(fn() {
    print("connected")
    c.write("hello")
})

c.on_message(fn(data) {
    print("echo: " + string(data))
    c.close()
})

c.start()
```

---

## Signal handling (Ctrl+C)

`server.accept()` / `client.start()` register a SIGINT handler automatically.

| Ctrl+C count | Behaviour |
|---|---|
| First | Graceful shutdown: stop accepting, drain existing connections up to `AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS` (default 5 s) |
| Second | Force exit |

Set the timeout with the environment variable:

```bash
AETHER_GRACEFUL_SHUTDOWN_TIMEOUT_SECS=30 aether server.ae
```

---

## Memory per connection

Each standby TCP connection uses approximately:

| Resource | Cost |
|---|---|
| Aether `TcpConnectionState` | ~390 B |
| Reader thread stack | 8 MB (OS default) |
| Kernel socket buffers | 8–256 KB |
| **Total** | **~8.4–8.6 MB** |

The thread-per-connection model limits practical concurrency to ~100 connections per GB of RAM. For higher concurrency, consider batching work or reducing the reader thread stack size.

---

## See also

- [ASYNC.md](ASYNC.md) — async/await and the I/O thread pool
- [EVENT_LOOP.md](../dev/EVENT_LOOP.md) — event loop internals
