---
layout: default
title: "Aether — Time"
---

[Home](../index.html) › Language Reference › Time

# Time

Aether provides two built-in time functions: `clock()` for measuring elapsed time, and `sleep()` for pausing execution.

## clock()

Returns the current Unix timestamp as a float (seconds since the Unix epoch).

```aether
let now = clock()
println(now)  // 1714252800.123456
```

Precision is microsecond (6 decimal places). Use two `clock()` calls to measure elapsed time:

```aether
let start = clock()
// ... work ...
let elapsed = clock() - start
println("Took ${elapsed} seconds")
```

## sleep(seconds)

Pauses execution for the given number of seconds. Accepts int or float.

```aether
sleep(2)      // pause 2 seconds
sleep(0.5)    // pause 500ms
```

When `set_workers(n)` is active or `AETHER_IO_WORKERS` is set, `sleep` runs on the I/O pool and does not block the main thread — use `await sleep(n)` in that case.

## Examples

### Timing a function

```aether
fn time_it(func) {
    let start = clock()
    func()
    println("Elapsed: ${clock() - start}s")
}

fn main() {
    time_it(fn() {
        let s = 0
        for i in range(0, 1000000) { s = s + i }
    })
}
```

### Rate limiting

```aether
fn main() {
    for i in range(0, 5) {
        println("Task ${i} at ${clock()}")
        sleep(1.0)
    }
}
```

### Retry with exponential backoff

```aether
fn retry(func, max_attempts) {
    let delay = 0.5
    for attempt in range(0, max_attempts) {
        try {
            return func()
        } catch(e) {
            if (attempt < max_attempts - 1) {
                sleep(delay)
                delay = delay * 2
            }
        }
    }
    throw "All ${max_attempts} attempts failed"
}
```

## Limitations

- No date/time formatting or timezone support
- `sleep()` is not interruptible (use small intervals if you need cancellation)
- Minimum sleep duration depends on the OS scheduler; sleep of 0 or negative is a no-op

## Related

- [Async/Await](ASYNC.html) — `await sleep(n)` when workers are active; `.then()` for non-blocking callbacks

---
[← HTTP](HTTP.html) &nbsp;&nbsp; [Random →](RANDOM.html)
