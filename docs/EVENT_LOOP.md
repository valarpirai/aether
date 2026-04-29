# Event Loop

Aether's event loop provides callback-based async I/O — an alternative to `await` for fire-and-forget patterns and future network programming.

## Overview

The event loop complements `await` by allowing you to register callbacks on I/O operations and let the interpreter drive them to completion without blocking in user code.

```
┌──────────────────────────────────────────┐
│            Main Thread                    │
│                                           │
│  on_ready(promise, fn(v) { ... })         │
│      → push Receiver+callback to queue    │
│                                           │
│  event_loop()                             │
│      → poll queue with try_recv()         │
│      → call callbacks when I/O resolves   │
│      → repeat until queue is empty        │
└──────────────────────────────────────────┘
```

## API

### `on_ready(value, callback)`

Registers a callback to fire when a value is ready.

- If `value` is a **Promise** (IoWaiting): registers the callback in the event loop queue; callback fires when I/O completes
- If `value` is any **non-Promise**: callback fires synchronously and immediately
- `callback` receives the resolved value as its first argument

```aether
set_workers(2)
let p = sleep(0.5)
on_ready(p, fn(v) {
    println("sleep finished")
})
event_loop()
```

### `event_loop()`

Runs the event loop until all pending callbacks have fired.

- Non-blocking poll via `try_recv()` — does not block the interpreter per tick
- Sleeps 1 ms when nothing is ready (avoids 100% CPU spin)
- Picks up new registrations made inside callbacks (chaining works)
- Returns `null` when queue is empty

```aether
event_loop()   // null
```

## How It Differs from `await`

| | `await` | `event_loop` |
|-|---------|-------------|
| Style | Pull (caller blocks) | Push (callback driven) |
| Blocking | Yes — main thread blocks on `rx.recv()` | No — polls with `try_recv()`, sleeps 1ms |
| Result capture | Variable binding | Callback argument |
| Multiple I/O | Sequential unless `Promise.all` | Concurrent — all resolve in parallel |
| Callback mutation | N/A | Closures can't mutate outer scope (use file/struct side effects) |

**When to use `await`**: You need the result before the next line.

**When to use `event_loop`**: Fire-and-forget patterns, concurrently dispatching many I/O tasks, or future event-driven server loops.

## Example: Concurrent File Writes

```aether
set_workers(4)

let p1 = sleep(0.01)
let p2 = sleep(0.02)

on_ready(p1, fn(v) {
    await write_file("/tmp/out1.txt", "done1")
})
on_ready(p2, fn(v) {
    await write_file("/tmp/out2.txt", "done2")
})

event_loop()   // both callbacks fire before returning

let r1 = await read_file("/tmp/out1.txt")
let r2 = await read_file("/tmp/out2.txt")
println(r1 + " " + r2)   // done1 done2
```

## Chained Callbacks

Callbacks registered inside a callback are picked up by the same `event_loop()` call — the loop continues until the queue is empty.

```aether
set_workers(2)
let p = sleep(0.01)

on_ready(p, fn(v) {
    let p2 = sleep(0.01)
    on_ready(p2, fn(v2) {
        await write_file("/tmp/chained.txt", "final")
    })
})

event_loop()
println(await read_file("/tmp/chained.txt"))   // final
```

## Non-Promise Values Fire Immediately

`on_ready` with a non-Promise fires the callback synchronously before returning, without needing `event_loop()`.

```aether
on_ready(42, fn(v) {
    println(v)   // prints 42 immediately
})
// event_loop() not needed
```

## File I/O in Callbacks

When the I/O pool is active (`set_workers` called), `read_file` and `write_file` return Promises. Use `await` inside callbacks:

```aether
set_workers(2)
let p = read_file("/tmp/input.txt")

on_ready(p, fn(content) {
    await write_file("/tmp/output.txt", content)   // await required here
})

event_loop()
let result = await read_file("/tmp/output.txt")
```

Without `set_workers`, `read_file` and `write_file` are synchronous and `await` is a no-op.

## Architecture

### `EventLoopQueue`

Located in `src/interpreter/event_loop.rs`.

```rust
pub struct EventLoopEntry {
    pub rx: Receiver<IoResult>,   // channel from I/O worker thread
    pub callback: Value,          // Aether function to call on completion
}

pub struct EventLoopQueue {
    pub pending: Vec<EventLoopEntry>,
}
```

`drain_ready()` does a single non-blocking pass:
- `try_recv()` on each entry
- Ready entries → returned for callback dispatch
- `Empty` → kept in pending
- `Disconnected` → silently dropped (worker panicked or task cancelled)

### Thread Safety

Worker threads only see `IoResult` (an enum over `Result<String, String>` and `Result<(), String>`). They never touch `Value` or `Rc<T>`. This ensures Rc-safety — all `Value` objects stay on the main thread.

### Evaluator Integration

`on_ready` and `event_loop` are intercepted in `eval_call` (in `functions.rs`) before the generic BuiltinFn dispatch:

- `register_on_ready`: extracts `Receiver<IoResult>` from `PromiseState::IoWaiting`, pushes to queue
- `run_event_loop`: loop → `drain_ready()` → `call_value(callback, [result])` → repeat

## Future: Network Programming

The event loop is designed to support TCP/UDP server patterns in a future phase:

```aether
// Future API (not yet implemented)
let server = tcp_listen("0.0.0.0:8080")
on_ready(server.accept(), fn(conn) {
    let req = await conn.read()
    await conn.write("HTTP/1.1 200 OK\r\n\r\nHello")
    on_ready(server.accept(), ...)   // re-register for next connection
})
event_loop()
```

The same `EventLoopQueue` mechanism works for any `Receiver<IoResult>`, so TCP accept/read/write can plug in by submitting tasks to the I/O pool and returning `Value::Promise(IoWaiting(rx))`.

## Tests

`tests/event_loop_test.rs` — 15 integration tests:

| Test | What it verifies |
|------|-----------------|
| `test_event_loop_empty_returns_null` | Empty queue returns null |
| `test_event_loop_multiple_calls_idempotent` | Multiple event_loop() calls are safe |
| `test_on_ready_non_promise_int_fires_immediately` | Non-promise int fires synchronously |
| `test_on_ready_non_promise_string_fires_immediately` | Non-promise string fires synchronously |
| `test_on_ready_non_promise_null_fires_immediately` | Non-promise null fires synchronously |
| `test_on_ready_non_promise_fires_without_event_loop` | No event_loop() needed for non-promise |
| `test_on_ready_sleep_fires_callback` | sleep promise callback fires via event_loop |
| `test_event_loop_returns_null_after_completion` | Returns null after all callbacks done |
| `test_multiple_on_ready_all_fire` | All 3 concurrent callbacks fire |
| `test_event_loop_waits_for_all_pending` | Waits for both fast and slow tasks |
| `test_chained_on_ready_event_loop_continues` | Nested on_ready inside callback works |
| `test_on_ready_read_file_async` | Async read_file resolved via event_loop |
| `test_on_ready_too_few_args_errors` | Arity error: on_ready(42) |
| `test_on_ready_too_many_args_errors` | Arity error: on_ready(42, fn, 99) |
| `test_event_loop_with_arg_errors` | Arity error: event_loop(1) |
