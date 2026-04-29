# Aether Configuration Reference

This document lists every knob that controls Aether's behaviour, grouped by how and when it takes effect.

---

## Environment Variables (read at startup)

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `AETHER_IO_WORKERS` | positive integer | _(none)_ | Enable the I/O thread pool with this many worker threads. When unset, I/O builtins (`http_get`, `sleep`, `read_file`, `write_file`, `http_post`) run synchronously on the main thread. When set, each call returns a `Promise` immediately and the I/O runs on a worker thread. |
| `AETHER_EVENT_LOOP_TIMEOUT` | positive float (seconds) | _(none)_ | Default timeout for `event_loop()` calls that pass no argument. `event_loop(n)` with an explicit argument always overrides this. When unset, `event_loop()` runs until the queue is empty with no deadline. |
| `AETHER_QUEUE_LIMIT` | positive integer | `1024` | Maximum number of pending callbacks in the event loop queue. `on_ready()` throws a runtime error when this limit is reached (backpressure). Can also be changed at runtime via `set_queue_limit(n)`. |
| `AETHER_CALL_DEPTH` | positive integer | `100` | Maximum Aether call stack depth before a `StackOverflow` error is raised. |
| `AETHER_HTTP_TIMEOUT` | positive integer (seconds) | `30` | Default request timeout for `http_get` and `http_post`. Per-request `timeout` key in the config dict overrides this. |
| `AETHER_HTTP_USER_AGENT` | string | `aether/0.1` | Default `User-Agent` header for `http_get` and `http_post`. Per-request `user_agent` key in the config dict overrides this. |
| `HOME` | path | _(OS default)_ | REPL history file is written to `$HOME/.aether_history`. If `HOME` is unset, REPL history is disabled for the session. |

### Example

```bash
# Run a script with a 4-worker I/O pool
AETHER_IO_WORKERS=4 aether examples/concurrent_io.ae

# Limit event_loop() to 30 seconds and cap queue at 500 entries
AETHER_EVENT_LOOP_TIMEOUT=30 AETHER_QUEUE_LIMIT=500 aether server.ae

# Set a global HTTP timeout of 10s and custom user-agent
AETHER_HTTP_TIMEOUT=10 AETHER_HTTP_USER_AGENT="mybot/1.0" aether scraper.ae
```

---

## Runtime Configuration (Aether builtins)

These are called from inside `.ae` programs and take effect immediately.

### `set_workers(n)`

Replaces the I/O thread pool with a new pool of `n` workers.

- **`n`**: positive integer
- Can be called before or after any I/O calls.
- A previous pool is discarded; in-flight tasks on the old pool complete normally (the old worker threads drain their queue before exiting).
- If called without `AETHER_IO_WORKERS` having been set, this is the first way to enable the pool.

```aether
set_workers(4)          // create/replace pool with 4 workers
let p = sleep(0.1)      // now runs on pool
await p
set_workers(2)          // shrink pool at runtime
```

---

### `set_queue_limit(n)`

Caps the event loop queue at `n` pending callbacks (backpressure).

- **`n`**: positive integer
- `on_ready()` throws immediately if the queue is already at the limit.
- Overrides `AETHER_QUEUE_LIMIT` for the remainder of the program.
- Default: `1024` (or `AETHER_QUEUE_LIMIT` if set at startup).

```aether
set_queue_limit(100)    // refuse more than 100 pending callbacks
on_ready(p1, cb)        // ok if queue < 100
on_ready(p2, cb)        // throws if queue == 100
```

---

### `set_task_timeout(secs)` / `set_task_timeout(null)`

Sets a per-task deadline for event loop callbacks registered via `on_ready()`. Callbacks whose I/O doesn't resolve within `secs` seconds are dropped silently. Pass `null` to clear the deadline.

- **`secs`**: positive number (int or float), or `null`
- Applies to all subsequent `on_ready()` registrations for the program's lifetime
- Overrides `AETHER_EVENT_LOOP_TIMEOUT` for per-task timing

```aether
set_task_timeout(10)      // each on_ready callback must complete within 10s
set_task_timeout(null)    // remove per-task timeout
```

---

### Per-request HTTP options

`http_get` and `http_post` accept an optional config dict as their last argument. Keys in the dict override the corresponding env-var defaults for that single request only.

| Key | Type | Description |
|-----|------|-------------|
| `timeout` | int (seconds) | Request timeout. Overrides `AETHER_HTTP_TIMEOUT`. |
| `user_agent` | string | `User-Agent` header. Overrides `AETHER_HTTP_USER_AGENT`. |

```aether
// single request with a 5-second timeout
let body = http_get("https://api.example.com/data", {timeout: 5})

// custom user-agent for one call
let resp = http_post("https://api.example.com/submit", payload, {
    timeout: 10,
    user_agent: "myapp/2.0"
})

// async path works the same way
let p = http_get("https://slow.example.com/", {timeout: 60})
let result = await p
```

---

## Compile-time / Hardcoded Constants

These values are baked into the binary. Changing them requires recompiling.

| Constant | Location | Value | Description |
|----------|----------|-------|-------------|
| `max_call_depth` | `evaluator/mod.rs` (constructors) | `100` | Maximum Aether call stack depth before a `StackOverflow` error is raised. Each Aether call uses ~10–20 Rust stack frames internally; 100 keeps the Rust stack comfortably under OS limits (~8 MB default). |
| `IoPool::default_workers()` | `interpreter/io_pool.rs` | `max(available_parallelism − 1, 4)` | Worker count chosen when only `AETHER_IO_WORKERS` is set (no explicit value override is currently exposed, but the formula is used for future `new_default_pool()` convenience). |
| REPL edit mode | `repl.rs` | Emacs | Key-binding style for the readline REPL (`Ctrl-A`, `Ctrl-E`, `Ctrl-K`, etc.). |
| REPL completion style | `repl.rs` | List | Tab-completion shows a list of candidates rather than cycling. |
| REPL history file name | `repl.rs` | `.aether_history` | Base name appended to `$HOME/`. |

---

## Configuration by Category

### I/O Concurrency

| Config | Mechanism | Scope |
|--------|-----------|-------|
| Number of I/O workers | `AETHER_IO_WORKERS` env var | process lifetime |
| Change workers at runtime | `set_workers(n)` builtin | immediate |
| Default worker formula | hardcoded in `IoPool::default_workers()` | compile-time |

### Call Stack

| Config | Mechanism | Scope |
|--------|-----------|-------|
| Max recursion depth | `max_call_depth = 100` in constructors | compile-time |

### REPL

| Config | Mechanism | Scope |
|--------|-----------|-------|
| History storage path | `HOME` env var | process lifetime |
| Edit mode (Emacs) | hardcoded in `rustyline` config | compile-time |
| Completion style (List) | hardcoded in `rustyline` config | compile-time |

---

## Potential Future Env Vars

The following constants are currently hardcoded but are candidates for env-var overrides in the future:

| Future variable | Controls | Current default |
|-----------------|----------|-----------------|
| `AETHER_HISTORY` | REPL history file path | `$HOME/.aether_history` |
| `AETHER_EDIT_MODE` | REPL edit mode (`emacs`/`vi`) | `emacs` |
