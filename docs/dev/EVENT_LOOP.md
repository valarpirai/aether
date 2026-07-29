---
layout: default
title: "Aether — Event Loop Internals"
---

[Home](../index.md) › Developer Docs › Event Loop Internals

# Event Loop

The event loop drives async callbacks to completion. In most programs, it runs automatically — you rarely need to call it directly.

## value.then(callback)

Registers a callback to fire when a value is ready. This is the primary way to attach async callbacks.

- **I/O Promise** (returned by `http_get`, `sleep`, etc. when workers are active): callback fires when the I/O completes
- **Any other value**: callback fires immediately with that value

```aether
set_workers(4)

let p = http_get("https://api.example.com/users")
p.then(fn(data) {
    println(data)
})
```

Because the event loop drains automatically when `main()` returns, no explicit `event_loop()` call is needed for this pattern.

## Auto-drain at main() exit

When `main()` finishes, the runtime automatically drains any pending callbacks before the program exits. This mirrors how Node.js keeps the process alive for pending async work.

```aether
fn main() {
    set_workers(4)

    http_get("https://api.example.com/users").then(fn(users) {
        println(users)
    })

    http_get("https://api.example.com/posts").then(fn(posts) {
        println(posts)
    })

    // Both callbacks fire automatically — no event_loop() needed
}
```

## Chained callbacks

Callbacks registered inside a callback are also drained — the runtime keeps going until the queue is completely empty.

```aether
fn main() {
    set_workers(2)

    sleep(0.01).then(fn(v) {
        sleep(0.01).then(fn(v2) {
            println("both done")
        })
    })
}
```

## on_ready(value, callback)

`on_ready` is the older function-based form of `.then()`. Both are equivalent.

```aether
set_workers(2)
on_ready(sleep(0.5), fn(v) {
    println("done")
})
```

## event_loop([timeout_secs])

Explicitly runs the event loop until all pending callbacks have fired. Use this when you need callbacks to complete before the next line of code (rather than at `main()` exit).

```aether
set_workers(4)

http_get("https://api.example.com/data").then(fn(data) {
    await write_file("/tmp/result.txt", data)
})

event_loop()   // wait for the write to finish

let content = await read_file("/tmp/result.txt")
println(content)
```

Accepts an optional timeout in seconds:

```aether
event_loop(5.0)   // give up after 5 seconds
```

## .then() vs await

| | `await` | `.then()` |
|-|---------|-----------|
| Style | Pull — caller waits for result | Push — callback receives result |
| Result | Returned directly | Passed to callback |
| Use when | You need the result on the next line | Fire-and-forget, or dispatching many I/O tasks |

```aether
// await — sequential
let users = await http_get("https://api.example.com/users")
let posts = await http_get("https://api.example.com/posts")

// .then() — concurrent (both start immediately)
http_get("https://api.example.com/users").then(fn(u) { process_users(u) })
http_get("https://api.example.com/posts").then(fn(p) { process_posts(p) })
```

## Inside a TCP or UDP server

Inside a TCP `on_message` or UDP `on_message` handler, `.then()` callbacks do **not** require `event_loop()`. The TCP/UDP dispatch loop calls `tick_async_callbacks()` after every event, which drains the queue automatically.

```aether
server.on_message(fn(conn, data) {
    http_get("https://api.example.com/").then(fn(resp) {
        conn.write(resp)   // fires via tick_async_callbacks, not event_loop
    })
})
server.accept()
```

Using `await` inside a TCP handler blocks the dispatch loop for the duration of the I/O call — no further messages can be processed until it returns. Prefer `.then()` when handling multiple concurrent connections.

## Queue controls

These are operational knobs, not part of normal programs. See [Configuration](../lang/CONFIGURATION.md) for details.

- `set_queue_limit(n)` — cap the number of pending callbacks (backpressure)
- `set_task_timeout(secs)` / `set_task_timeout(null)` — per-callback deadline

---
[← Architecture](ARCHITECTURE.md) &nbsp;&nbsp; [Async I/O Architecture →](ASYNC_IO.md)
