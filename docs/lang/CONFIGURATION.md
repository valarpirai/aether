---
layout: default
title: "Aether — Configuration"
---

[Home](../index.md) › Language Reference › Configuration

# Configuration

Aether's behavior can be tuned through environment variables (set before launch) and runtime built-in calls (called from `.ae` code).

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AETHER_IO_WORKERS` | _(unset)_ | Enable the I/O thread pool with this many worker threads. When unset, `http_get`, `sleep`, `read_file`, `write_file`, and `http_post` run synchronously. When set, each call returns a Promise and runs on a worker thread. |
| `AETHER_HTTP_TIMEOUT` | `30` | Default request timeout in seconds for `http_get` and `http_post`. |
| `AETHER_HTTP_USER_AGENT` | `aether/0.1` | Default `User-Agent` header for HTTP requests. |
| `AETHER_EVENT_LOOP_TIMEOUT` | _(none)_ | Default timeout in seconds for `event_loop()` calls with no argument. When unset, `event_loop()` runs until the queue is empty. Used with `.then()` / `on_ready()` callback-based async. |
| `AETHER_QUEUE_LIMIT` | `1024` | Maximum pending `.then()` / `on_ready()` callbacks that can be queued at once. Throws when this limit is reached. |
| `AETHER_CALL_DEPTH` | `100` | Maximum call stack depth before a `StackOverflow` error is raised. |
| `HOME` | OS default | REPL history is saved to `$HOME/.aether_history`. History is disabled for the session if `HOME` is unset. |

### Examples

```bash
# Enable 4-worker I/O pool
AETHER_IO_WORKERS=4 aether script.ae

# Limit event_loop to 30s, cap queue at 500
AETHER_EVENT_LOOP_TIMEOUT=30 AETHER_QUEUE_LIMIT=500 aether server.ae

# Custom HTTP timeout and user-agent
AETHER_HTTP_TIMEOUT=10 AETHER_HTTP_USER_AGENT="mybot/1.0" aether scraper.ae
```

## Runtime built-ins

These are called from inside your `.ae` program and take effect immediately.

### set_workers(n)

Enables (or replaces) the I/O thread pool with `n` workers. Any previously submitted tasks complete on the old pool before it shuts down.

```aether
set_workers(4)
let p = sleep(0.1)   // now runs on the pool
await p
```

### set_queue_limit(n)

Caps the event loop queue at `n` pending callbacks. Overrides `AETHER_QUEUE_LIMIT`.

```aether
set_queue_limit(100)
```

### set_task_timeout(secs) / set_task_timeout(null)

Sets a per-task deadline for `on_ready()` callbacks. Callbacks that don't resolve within `secs` seconds are silently dropped. Pass `null` to remove the deadline.

```aether
set_task_timeout(10)    // each callback must complete within 10s
set_task_timeout(null)  // remove deadline
```

## Per-request HTTP options

`http_get` and `http_post` accept an optional config dict as their last argument, overriding env-var defaults for that single request:

```aether
let data = http_get("https://slow.api.example.com/", {timeout: 60})
let resp = http_post("https://api.example.com/", payload, {
    timeout: 10,
    user_agent: "myapp/2.0"
})
```

| Key | Type | Overrides |
|-----|------|-----------|
| `timeout` | int (seconds) | `AETHER_HTTP_TIMEOUT` |
| `user_agent` | string | `AETHER_HTTP_USER_AGENT` |

---
[← Async](ASYNC.md) &nbsp;&nbsp; [Home →](../index.md)
