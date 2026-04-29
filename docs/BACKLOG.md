# Aether Language Backlog

Features planned for future development, ordered by priority within each tier.
Items without a milestone are unscheduled.

---

## Recently Completed

| Feature | Date |
|---------|------|
| `finally` block | 2026-04-29 |
| `??` null coalescing | 2026-04-29 |
| `?.` optional chaining | 2026-04-29 |
| Triple-quoted multi-line strings `"""..."""` | 2026-04-29 |
| Labeled `break` / `continue` for nested loops | 2026-04-29 |
| File utilities: `list_dir`, `path_join`, `rename`, `rm` | 2026-04-29 |
| Event loop — `on_ready`, `event_loop`, per-task timeout, backpressure, error isolation | 2026-04-29 |

---

## Tier 1 — High value, low complexity

### `match` statement
Pattern matching on values; replaces chained `if/else` for discriminated unions and literal checks.

```aether
match shape {
    "circle"  => println("round")
    "square"  => println("four sides")
    _         => println("unknown")
}
```

Stretch: structural patterns (`match point { {x: 0, y} => ... }`).

---

### Destructuring assignment
Unpack arrays and dicts directly into variables.

```aether
let [a, b, c] = [1, 2, 3]
let {x, y} = point
let [head, ...tail] = items
let {host, port: p} = config        // rename on unpack
let [first, second = 0] = values    // default value
```

---

### `format()` / string format specifiers
Formatted output beyond `"${}"` interpolation.

```aether
format("{:.2f}", 3.14159)       // "3.14"
format("{:>10}", "hi")          // "        hi"
format("{:0>5d}", 42)           // "00042"
format("Hello, {}!", name)      // positional
```

---

### Variadic arguments
Functions that accept any number of arguments.

```aether
fn sum(*args) {
    let total = 0
    for n in args { total = total + n }
    return total
}
sum(1, 2, 3, 4)   // 10
```

---

## Tier 2 — High value, medium complexity

### Enums / tagged unions
First-class sum types for domain modeling.

```aether
enum Shape {
    Circle(radius)
    Rect(width, height)
    Point
}

let s = Shape.Circle(5)
match s {
    Shape.Circle(r) => println("area:", 3.14 * r * r)
    Shape.Rect(w, h) => println("area:", w * h)
    Shape.Point => println("zero area")
}
```

---

### Tuples
Lightweight fixed-length heterogeneous values, cheaper than structs for small groupings.

```aether
let pair = (1, "hello")
let (n, s) = pair
fn min_max(arr) { return (min(arr), max(arr)) }
let (lo, hi) = min_max([3, 1, 4, 1, 5])
```

---

### Named / keyword arguments
Call functions with named parameters in any order.

```aether
fn connect(host, port, timeout) { ... }
connect(host="localhost", port=5432, timeout=30)
connect(port=5432, host="db.internal")
```

---

### Default parameter values (explicit syntax)
Proper default values without relying on `null` checks.

```aether
fn greet(name, greeting="Hello") {
    println(greeting + ", " + name + "!")
}
greet("Alice")            // Hello, Alice!
greet("Bob", "Hi")        // Hi, Bob!
```

---

### Numeric range literals (`0..10`)
Inline ranges without calling `range()`.

```aether
for i in 0..10 { println(i) }          // exclusive: 0–9
for i in 0..=10 { println(i) }         // inclusive: 0–10
for i in 10..0 { println(i) }          // descending
let evens = [i for i in 0..20 if i % 2 == 0]
```

---

### List / dict comprehensions
Compact collection construction.

```aether
let squares = [x * x for x in 1..11]
let even    = [x for x in data if x % 2 == 0]
let lookup  = {k: v for k, v in zip(keys, values)}
```

---

### Generators / `yield`
Lazy sequences and pipelines without building full arrays.

```aether
fn fibonacci() {
    let a = 0
    let b = 1
    loop {
        yield a
        let tmp = a + b
        a = b
        b = tmp
    }
}
for n in take(fibonacci(), 10) { println(n) }
```

---

### Error hierarchy
Typed errors instead of plain strings; catchable by type.

```aether
struct NetworkError {
    message
    code
}

try {
    http_get(url)
} catch(e) {
    if (type(e) == "NetworkError") {
        println("HTTP", e.code, e.message)
    } else {
        throw e
    }
}
```

---

### `Result` type
Explicit `Ok(val)` / `Err(msg)` return without exceptions; composable via `map`, `unwrap_or`.

```aether
fn parse_int(s) {
    if (is_numeric(s)) { return Ok(int(s)) }
    return Err("not a number: " + s)
}

let r = parse_int("42")
if (r.is_ok()) { println(r.unwrap()) }
println(r.unwrap_or(0))
```

---

## Tier 3 — Networking and concurrency

### TCP / UDP server support
Use the event loop for network programming. Requires OS-level async sockets feeding into `EventLoopQueue` via channels, mirroring how `sleep`/`read_file` work today.

```aether
fn main() {
    set_workers(4)
    let server = tcp_listen("0.0.0.0:8080")

    fn accept_next() {
        let conn = server.accept()           // returns Promise
        on_ready(conn, fn(c) {
            let req = await c.read()
            await c.write("HTTP/1.1 200 OK\r\n\r\nHello")
            c.close()
            accept_next()                    // chain: accept next connection
        })
    }

    accept_next()
    event_loop()
}
```

**What's needed:**
- `tcp_listen(addr)` → `Value::TcpListener` (wraps `std::net::TcpListener`)
- `server.accept()` → submits accept task to pool, returns Promise
- `conn.read()` / `conn.write(data)` / `conn.close()` → async via pool
- `IoTask::TcpAccept`, `IoTask::TcpRead`, `IoTask::TcpWrite` variants in `io_pool.rs`

---

### Persistent server event loop (`event_loop_forever()`)
Current `event_loop()` exits when the queue is empty. A server needs to keep waiting for new work even when temporarily idle.

```aether
event_loop_forever()   // blocks until explicit shutdown() call
shutdown()             // signals the loop to exit after current tick
```

**Implementation:** Replace the `is_empty() → break` logic with a `Condvar` wakeup. When `on_ready` pushes to an empty queue it signals the condvar; the loop blocks on the condvar instead of sleeping 1ms.

---

### Non-blocking network I/O (OS async sockets)
Currently `reqwest::blocking` and `std::net` consume one I/O thread per in-flight request. For high-connection-count servers, switch to OS-level async (`epoll`/`kqueue`) via `mio` or `polling` crate — sockets complete without tying up thread pool slots.

**Impact:** Allows thousands of concurrent connections with the same 4-worker pool. Prerequisite for production-grade HTTP/TCP servers.

---

### Worker threads (CPU-bound parallelism)
Separate Aether interpreter instances for CPU-bound work, each with their own event loop. Communicate via message passing (no shared `Value`).

```aether
let w = spawn_worker("worker.ae")
w.post({ task: "compress", data: large_array })
on_ready(w.message(), fn(result) {
    println("worker result:", result)
})
event_loop()
```

**Why:** The main interpreter is single-threaded; heavy computation blocks all callbacks. Worker threads offload CPU work without touching `Rc<T>` on the main thread. Each worker is a full `Evaluator` in its own OS thread; cross-thread values are serialised to JSON at the boundary.

---

### Error callback for failed / timed-out tasks
Currently I/O errors and per-task timeouts are logged to stderr and the callback is skipped. Add an optional error handler so Aether code can react.

```aether
on_ready(p, fn(v) {
    println("ok:", v)
}, fn(err) {
    println("failed:", err.message)   // timeout or I/O error
})
```

Or error-first style (single callback, `null` on success path):
```aether
on_ready(p, fn(err, v) {
    if err != null { println("error:", err) } else { println(v) }
})
```

---

### Stack trace attribution for event loop callbacks
Callbacks registered via `on_ready` show as `<anonymous>` in stack traces. Track the source line of the `on_ready` call so error messages read:

```
RuntimeError at line 12: undefined variable 'x'
  at <anonymous> (main.ae:12)
  at on_ready callback registered at (main.ae:9)
```

**Implementation:** Store `call_site_line` and `call_site_file` in `EventLoopEntry` at `register_on_ready` time; include in the stack frame pushed by `call_value` for callbacks.

---

## Tier 4 — Operators and syntax sugar


### `**` power operator
```aether
let cube = x ** 3
let dist = (dx ** 2 + dy ** 2) ** 0.5
```

### Bitwise operators
`&`, `|`, `^`, `~`, `<<`, `>>`
```aether
let flags = READ | WRITE
let masked = value & 0xFF
let shifted = 1 << n
```

### Integer division `//`
```aether
let pages = total_items // per_page
```

### Ternary / inline-if expression
```aether
let label = count == 1 ? "item" : "items"
let abs_x = if x >= 0 then x else -x
```

### `defer` statement (Go-style)
Execute a statement when the current function returns, regardless of how.

```aether
fn read_config(path) {
    let f = open(path)
    defer f.close()
    return parse(f.read())
}
```

---

## Tier 5 — Type system

### Type annotations (gradual)
Optional static types that the interpreter checks at call boundaries.

```aether
fn add(a: int, b: int) -> int {
    return a + b
}
let name: string = "Alice"
```

### Struct inheritance / interfaces
```aether
interface Printable {
    fn to_string() -> string
}

struct Dog extends Animal implements Printable {
    breed
    fn to_string() { return "Dog(" + self.name + ")" }
}
```

---

## Tier 6 — I/O and stdlib

| Feature | API sketch |
|---------|-----------|
| stderr / log levels | `eprintln(msg)`, `log.warn(msg)` |
| Directory operations | `ls(path)`, `mkdir(path)`, `rm(path)`, `exists(path)` |
| Path helpers | `path.join(a, b)`, `path.basename(p)`, `path.ext(p)` |
| Environment variables | `env("HOME")`, `env("PORT", "8080")` |
| Regular expressions | `re.match(pattern, text)`, `re.find_all(p, t)` |
| Raw strings | `r"no\escape\needed"` |

---

## Tier 7 — Tooling (longer horizon)

| Tool | Description |
|------|-------------|
| `aether fmt` | Auto-formatter for `.ae` files |
| `aether check` | Type checker / linter without running |
| `aether test` | Built-in test runner |
| Package manager | `aether.toml`, versioned deps, registry |
| REPL multi-line input | Paste multi-line code blocks in the REPL |
| Debugger | Breakpoints, step-through, variable inspection |
| Bytecode compiler | Replace tree-walking interpreter; 5–20× speedup |

---

## Deferred (explicit non-goals for now)

- Concurrency primitives (channels, mutex) — async I/O pool covers most use cases
- Macros / metaprogramming — adds significant parser/evaluator complexity
- JIT compilation — follow bytecode compiler first
