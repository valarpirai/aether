---
layout: default
title: "Aether — Async I/O Architecture"
---

[Home](../index.html) › Developer Docs › Async I/O Architecture

# Async I/O Architecture

This document explains how async I/O works internally: the I/O thread pool, the event loop queue, the TCP/UDP I/O thread, and how the main interpreter thread coordinates all of them.

---

## Thread model

When a TCP server runs with `AETHER_IO_WORKERS=2`:

```
┌─────────────────────────────────┐
│  Main thread (interpreter)      │  runs all Aether code and callbacks
├─────────────────────────────────┤
│  TCP I/O thread (mio)           │  epoll/kqueue, owns all sockets
├─────────────────────────────────┤
│  IoPool worker 1                │  HTTP, file I/O, sleep
├─────────────────────────────────┤
│  IoPool worker 2                │  HTTP, file I/O, sleep
└─────────────────────────────────┘
```

The main thread is the only thread that ever touches `Value` or `Rc<T>`.  
Worker threads and the TCP I/O thread communicate with the main thread through channels using only primitive types (`String`, `f64`, `Vec<u8>`).

---

## Component 1 — IoPool (`src/interpreter/io_pool.rs`)

A fixed set of OS worker threads that run blocking I/O tasks.

```
IoPool
  task_tx ──► shared Mutex<Receiver<IoTask>>
                        │
          ┌─────────────┴──────────────┐
          ▼                            ▼
     Worker 1                    Worker 2
     loop {                      loop {
       rx.lock().recv()            rx.lock().recv()   ← blocks when idle
       match task {                ...
         HttpGet  → reqwest
         HttpPost → reqwest
         Sleep    → thread::sleep
         ReadFile → fs::read_to_string
         WriteFile→ fs::write
       }
       tx.send(result)            }
     }
```

**Key design points:**

- Workers loop forever — they block on `recv()` when idle, never exit.
- Each submitted task carries its own **dedicated oneshot channel** `(tx, rx)`. The worker sends the result through `tx`; the main thread holds `rx`.
- Only `IoPool` is created per `Evaluator`. `AETHER_IO_WORKERS` controls how many threads it has.
- If `AETHER_IO_WORKERS` is not set and `set_workers()` is never called, `io_pool` is `None` and all async builtins fall back to their synchronous implementations.

**Supported tasks:**

| Task | Builtin |
|------|---------|
| `HttpGet` | `http_get(url)` |
| `HttpPost` | `http_post(url, body)` |
| `Sleep` | `sleep(secs)` |
| `ReadFile` | `read_file(path)` |
| `WriteFile` | `write_file(path, content)` |

---

## Component 2 — EventLoopQueue (`src/interpreter/event_loop.rs`)

Lives entirely on the main thread. A `Vec` of pending entries — each pairing an I/O result receiver with an Aether callback.

```rust
struct EventLoopEntry {
    rx:       Receiver<IoResult>,  // result channel from IoPool worker
    callback: Value,               // Aether fn to call when result arrives
    deadline: Option<Instant>,     // optional per-task timeout
}
```

**`drain_ready()`** — called on every tick, non-blocking:

```
for each entry in pending:
  if deadline expired  → fire callback with Err("task timed out")
  elif try_recv() == Ok → fire callback with result
  else                  → keep in pending for next tick
```

`drain_ready` uses `try_recv()` (never blocks). An entry stays in `pending` until its worker sends a result or its deadline expires.

Maximum 1024 entries by default (configurable with `set_queue_limit(n)`).

---

## Component 3 — TCP I/O thread (`src/interpreter/tcp.rs`)

Spawned when `server.accept()` or `client.start()` is called. Runs a mio poll loop on its own OS thread. Completely separate from IoPool.

```
TCP I/O thread                           Main thread
────────────────────────────────         ─────────────────────────────
mio Poll (50 ms timeout):                TCP dispatch loop (10 ms):

  new connection                           recv_timeout(10ms)
    → event_tx.send(Connected)  ────────►    dispatch_tcp_server_event()
  data received                              → call on_connect / on_message
    → event_tx.send(Message)    ────────►      / on_disconnect callbacks
  client disconnected
    → event_tx.send(Disconnected)────────►
                                           tick_async_callbacks()
  cmd_rx.try_recv():           ◄────────    cmd_tx.send(Write { data })
    Write   → stream.write_all           ◄────────    cmd_tx.send(CloseConn)
    CloseConn → close stream             ◄────────    cmd_tx.send(Shutdown)
    Shutdown  → set shutdown flag
```

Two channels connect the threads:

| Channel | Direction | Carries |
|---------|-----------|---------|
| `event_tx / event_rx` | I/O thread → main | `TcpEvent` (Connected, Message, Disconnected, Error, Timeout) |
| `cmd_tx / cmd_rx` | main → I/O thread | `TcpCommand` (Write, CloseConn, Shutdown) |

UDP uses the same pattern (`src/interpreter/udp.rs`).

---

## Component 4 — Main thread dispatch loop

The main thread runs a tight loop after `server.accept()` is called:

```rust
loop {
    // SIGINT handling (see TCP/UDP Internals doc)
    ...

    // Wait up to 10 ms for the next TCP event
    match event_rx.recv_timeout(Duration::from_millis(10)) {
        Ok(evt)                        => dispatch_tcp_server_event(evt),
        Err(RecvTimeoutError::Timeout) => {}        // no event, loop again
        Err(RecvTimeoutError::Disconnected) => break // I/O thread exited
    }

    // Fire any .then() / on_ready() callbacks whose I/O is ready
    tick_async_callbacks()   // calls EventLoopQueue::drain_ready()
}
```

`tick_async_callbacks()` is what makes `.then()` work inside TCP handlers — after each TCP event (or timeout), it drains the EventLoopQueue and fires any callbacks whose HTTP/file response has arrived.

---

## The three async patterns

### Pattern A — `await` (blocking)

```
Aether:  let resp = await http_get(url)

Main thread:
  http_get(url)
    → IoPool.submit(HttpGet { url, tx })
    → return Promise(IoWaiting(rx))
  await Promise(IoWaiting(rx))
    → rx.recv()   ◄── BLOCKS until worker sends result

Worker thread:
  → reqwest::blocking::get(url)
  → tx.send(IoResult::Str(Ok(body)))

Main thread unblocks, gets result string
```

**EventLoopQueue: not used.**  
If called inside a TCP `on_message`, the dispatch loop is stalled for the duration of the HTTP request. Messages queue up in the TCP I/O thread's read buffer and are processed one at a time.

---

### Pattern B — `on_ready` + `event_loop()` (non-blocking, standalone)

```
Aether:  on_ready(http_get(url), fn(resp) { ... })
         event_loop()

Main thread:
  http_get(url) → Promise(IoWaiting(rx))
  on_ready(promise, callback)
    → EventLoopQueue.push(rx, callback)   ← returns immediately
  event_loop()
    loop {
      if queue.is_empty() → break
      ready = queue.drain_ready()         ← non-blocking try_recv
      for (result, cb) in ready → call cb
      if nothing ready → sleep(1ms)
    }
```

**EventLoopQueue: used.**  
Multiple `on_ready` calls can be registered before `event_loop()` — all their HTTP requests run concurrently on IoPool workers.

---

### Pattern C — `.then()` inside TCP `on_message` (non-blocking, recommended)

```
Aether:  server.on_message(fn(conn, data) {
             http_get(url).then(fn(resp) { conn.write(resp) })
         })

TCP dispatch loop:
  recv_timeout → Message 1
    on_message(conn, data)
      http_get(url) → Promise(IoWaiting(rx))
      .then(callback)
        → EventLoopQueue.push(rx, callback)  ← returns immediately
      on_message returns

  tick_async_callbacks()                     ← fires callback when HTTP done

  recv_timeout → Message 2                  ← processed immediately, not blocked
    on_message(conn, data)
      http_get(url).then(...)               ← another queued, concurrent
```

**EventLoopQueue: used. TCP dispatch loop keeps running.**  
Both HTTP requests run concurrently on IoPool. Their callbacks fire via `tick_async_callbacks()` as results arrive.

---

## Auto-drain at `main()` exit

After `main()` returns, `call_main()` (`evaluator/mod.rs:957`) checks whether the EventLoopQueue is non-empty and, if so, calls `run_event_loop(None)` automatically:

```rust
// evaluator/mod.rs — call_main()
result?;
if !self.async_rt.event_loop_queue.is_empty() {
    self.run_event_loop(None)?;
}
```

This is why `.then()` callbacks fire without an explicit `event_loop()` call in normal programs. It does not apply inside TCP/UDP servers — those have their own dispatch loop and never return from `call_main` until the server is closed.

## Chained callbacks

A callback registered inside another callback (`sleep(0.1).then(fn { sleep(0.1).then(...) })`) works because `run_event_loop` re-checks `is_empty()` at the top of each iteration. When the inner `.then()` pushes a new entry mid-loop, it is caught on the next tick — the loop only exits when the queue stays empty after a full drain.

---

## Comparison

| | `await` | `on_ready` + `event_loop()` | `.then()` in TCP handler |
|--|---------|----------------------------|--------------------------|
| EventLoopQueue used | No | Yes | Yes |
| Blocks main thread | Yes | No | No |
| Concurrent requests | No | Yes | Yes |
| Works in TCP handler | Yes (but serialises) | No (`event_loop` exits when empty) | Yes |
| Requires `event_loop()` | No | Yes | No (`tick_async_callbacks`) |
| Auto-drains at `main()` exit | No | Yes | N/A (TCP server never reaches exit) |

For usage examples and the user-facing API see [EVENT_LOOP.md](EVENT_LOOP.html).

---

## Data flow diagram

```
Aether code
    │
    ├── await http_get(url)
    │       │
    │       ▼
    │   IoPool.submit(task)          IoPool worker
    │       │                              │
    │       │◄── Promise(IoWaiting(rx)) ───┘
    │       │
    │   rx.recv()  ← blocks main thread
    │       │
    │       ▼
    │   result (String)
    │
    └── http_get(url).then(cb)
    │       │
    │       ▼
    │   IoPool.submit(task)          IoPool worker
    │       │                              │
    │       │◄── Promise(IoWaiting(rx)) ───┘
    │       │
    │   EventLoopQueue.push(rx, cb)
    │       │
    │       ▼  (returns immediately)
    │
    │   [later] tick_async_callbacks()
    │       │
    │       ▼
    │   drain_ready() → try_recv() → call cb(result)
    │
TCP I/O thread (mio)
    │
    ├── new connection → event_tx.send(Connected)
    ├── data received  → event_tx.send(Message)
    └── disconnected   → event_tx.send(Disconnected)
            │
            ▼  (main thread dispatch loop)
        on_connect / on_message / on_disconnect callbacks
```

---

## See also

- [EVENT_LOOP.md](EVENT_LOOP.html) — `on_ready`, `event_loop`, queue controls, backpressure
- [TCP_UDP.md](TCP_UDP.html) — TCP/UDP mio loop, SIGINT handling, graceful shutdown
- [INTERPRETER.md](INTERPRETER.html) — evaluator sub-module layout
- `src/interpreter/io_pool.rs` — IoPool and IoTask definitions
- `src/interpreter/event_loop.rs` — EventLoopQueue implementation
- `src/interpreter/tcp.rs` — TCP I/O thread
- `src/interpreter/udp.rs` — UDP I/O thread
- `src/interpreter/evaluator/functions.rs` — dispatch loops, `await_value`, `register_on_ready`

---
[← Event Loop Internals](EVENT_LOOP.html) &nbsp;&nbsp; [TCP/UDP Internals →](TCP_UDP.html)
