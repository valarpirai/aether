# Event Loop

The event loop is a callback-based alternative to `await` for running concurrent I/O. Instead of blocking on each result, you register a callback with `on_ready()` and let `event_loop()` drive all callbacks to completion.

## on_ready(value, callback)

Registers a callback to fire when a value is ready.

- If `value` is an I/O Promise (returned by `http_get`, `sleep`, `read_file`, etc. with workers active): the callback fires when the I/O completes
- If `value` is any other value: the callback fires synchronously and immediately

The callback receives the resolved value as its argument.

```aether
set_workers(2)

let p = sleep(0.5)
on_ready(p, fn(v) {
    println("sleep finished")
})

event_loop()
```

## event_loop([timeout_secs])

Runs until all pending callbacks have fired, then returns `null`. Optionally accepts a timeout in seconds.

```aether
event_loop()       // run until all callbacks fire
event_loop(5.0)    // run for at most 5 seconds
```

Callbacks registered inside a callback are picked up by the same `event_loop()` call — the loop continues until the queue is completely empty.

## Concurrent I/O with the event loop

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

event_loop()
```

Both sleeps run concurrently on the thread pool. Both callbacks fire before `event_loop()` returns.

## Chained callbacks

Register a new callback from inside a callback — the outer `event_loop()` handles it:

```aether
set_workers(2)

let p = sleep(0.01)
on_ready(p, fn(v) {
    let p2 = sleep(0.01)
    on_ready(p2, fn(v2) {
        println("both done")
    })
})

event_loop()
```

## Non-Promise values fire immediately

`on_ready` with a non-Promise fires synchronously — `event_loop()` is not needed:

```aether
on_ready(42, fn(v) {
    println(v)   // prints 42 immediately
})
```

## Queue controls

`set_queue_limit(n)` — caps the number of pending callbacks. `on_ready()` throws if the queue is full. Default: 1024 (override with `AETHER_QUEUE_LIMIT`).

`set_task_timeout(secs)` / `set_task_timeout(null)` — sets a per-task deadline; callbacks that don't resolve in time are silently dropped. Pass `null` to remove the deadline.

## await vs event_loop

| | `await` | `event_loop` |
|-|---------|-------------|
| Style | Pull — caller waits for result | Push — callback receives result |
| Main thread | Blocks until I/O resolves | Polls without blocking |
| Multiple I/O | Sequential (or `Promise.all`) | All run concurrently |

Use `await` when you need the result before the next line. Use the event loop for fire-and-forget patterns or when dispatching many I/O tasks concurrently.
