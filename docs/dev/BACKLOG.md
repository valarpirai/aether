---
layout: default
title: "Aether — Backlog"
---

[Home](../index.md) › Developer Docs › Backlog

# Aether Language Backlog

Features planned for future development, ordered by priority within each tier.
Items without a milestone are unscheduled.

---

## Recently Completed

| Feature | Date |
|---------|------|
| `format()` / string format specifiers | 2026-07-05 |
| `random()` / `rand_int(n)` — random number generation | 2026-07-05 |
| Dict O(1) lookup — `IndexMap` replaces `Vec` linear scan | 2026-06-03 |
| `finally` block | 2026-04-29 |
| `??` null coalescing | 2026-04-29 |
| `?.` optional chaining | 2026-04-29 |
| Triple-quoted multi-line strings `"""..."""` | 2026-04-29 |
| Labeled `break` / `continue` for nested loops | 2026-04-29 |
| File utilities: `list_dir`, `path_join`, `rename`, `rm` | 2026-04-29 |
| Event loop — `on_ready`, `event_loop`, per-task timeout, backpressure, error isolation | 2026-04-29 |
| Debugger — `debugger` statement with step/next/continue/backtrace | 2026-04-30 |
| Command-line args — global `args` array, max 100 | 2026-04-30 |
| `**` power operator (right-associative, int/float) | 2026-04-30 |
| Bitwise operators — `&` `\|` `^` `~` `<<` `>>` (integers only) | 2026-04-30 |
| Ternary expression — `condition ? then : else` | 2026-04-30 |
| `match` statement — literal, wildcard, bind, or-patterns, enum variants | 2026-04-30 |
| Destructuring assignment — array, dict, rest, rename, defaults | 2026-04-30 |

---

## Tier 1 — High value, low complexity

### ~~`format()` / string format specifiers~~ ✅ DONE
Formatted output beyond `"${}"` interpolation. Implemented in `builtin_format` (`src/interpreter/builtins.rs`): positional `{}`, `{:.Nf}` precision, `{:>N}`/`{:<N}`/`{:^N}` alignment, fill chars, and `{:x}`/`{:o}`/`{:b}` bases. See `docs/lang/FORMAT.md`.

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

### ~~`random()` / `rand_int(n)` — random number generation~~ ✅ DONE
`random()` returns a float in `[0, 1)`; `rand_int(n)` returns an int in `[0, n)`. Implemented via the `rand` crate in `builtin_random`/`builtin_rand_int` (`src/interpreter/builtins.rs`). See `docs/lang/RANDOM.md`.

---

### ~~`int(str)` implicit base 10~~ ✅ DONE
`int("42")` already defaults to base 10 in `builtin_int` (`src/interpreter/builtins.rs`); no base argument is required. Covered by `tests/number_conversion_test.rs`.

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

### ~~TCP / UDP server support~~ ✅ DONE (2026-05-27)

TCP and UDP both implemented with event-driven mio I/O.
See `docs/lang/TCP.md`.

- TCP: `tcp_listen`, `tcp_connect`, full lifecycle callbacks, delimiter framing
- UDP: `udp_bind`, `on_message`, `send_to`, `listen`, `close`
- Both: async callbacks (`await`, `.then()`) work inside event handlers

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


### `defer` statement (Go-style)
Execute a statement when the current function returns, regardless of how.

```aether
fn read_config(path) {
    let f = open(path)
    defer f.close()
    return parse(f.read())
}
```

Note: `try/finally` already covers the same use case.

---

## Tier 5 — FFI and extensibility

### ~~FFI / Plugin system~~ ✅ DONE (Phase 3, 2026-07-24)
Load compiled Rust shared libraries (`.so`/`.dylib`/`.dll`) from Aether programs, enabling access to the entire Rust ecosystem without rewriting libraries as built-ins. V1 protocol handles `int`-only functions; V2 protocol handles `String`, `Vec<i64>`, and `HashMap<String,i64>`. Remaining: async plugin functions (Phase 4). See `docs/lang/PLUGINS.md`.

```aether
let db = load_plugin("libaether_postgres.so")
let conn = db.pg_connect("postgresql://localhost/mydb")
let rows = db.pg_query(conn, "SELECT * FROM users")
```

**Provides access to:**
- Database drivers (postgres, redis, sqlite)
- Image processing (image, imageproc)
- Cryptography (ring, openssl)
- ML libraries (onnxruntime, candle)
- Any Rust crate

**Architecture:**
- Dynamic library loading via `libloading`
- Type conversion: `Value` ↔ Rust primitives/String/Vec/HashMap
- Plugin registration protocol with `#[aether_export]` macro
- No shared `Rc` — copy semantics at FFI boundary
- Synchronous only (Phase 1)

See **[FFI_PLUGIN_SYSTEM.md](FFI_PLUGIN_SYSTEM.md)** for full design.

---

## Tier 6 — Type system

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

## Tier 7 — I/O and stdlib

### Missing math functions
`sqrt`, `log2`, `log10` are absent; workarounds exist but are unreadable.

```aether
sqrt(9)       // 3.0
log2(1024)    // 10.0
log10(1000)   // 3.0
```

**Implementation:** Add to `stdlib/math.ae` using `exp(0.5 * log(n))` for `sqrt` and `log(n, 2)` / `log(n, 10)` for the others — no Rust changes needed.

---

### `dict.merge(other)` — merge two dicts
No clean way to combine two dicts today; must iterate manually.

```aether
let defaults = {"timeout": 30, "retries": 3}
let overrides = {"timeout": 10}
let config = defaults.merge(overrides)   // {"timeout": 10, "retries": 3}
```

Keys in `other` overwrite keys in `self`. Returns a new dict; does not mutate either input.

---

### `array.flat_map(fn)` and `array.group_by(fn)`

```aether
// flat_map: map then flatten one level
let words = [["hello", "world"], ["foo"]].flat_map(fn(arr) { return arr })
// ["hello", "world", "foo"]

// group_by: partition into a dict of arrays
let grouped = [1,2,3,4,5,6].group_by(fn(n) { return n % 2 == 0 ? "even" : "odd" })
// {"odd": [1,3,5], "even": [2,4,6]}
```

Both can be written in `stdlib/collections.ae`.

---

### `sort_by_key(arr, fn)` — sort with a key extractor

```aether
let people = [{"name": "Zara", "age": 25}, {"name": "Ali", "age": 30}]
sort_by_key(people, fn(p) { return p["age"] })
```

Currently `.sort()` on an array of dicts fails. Add to `stdlib/collections.ae`.

---

### `str.find(substr)` — returns index

```aether
"hello world".find("world")   // 6
"hello world".find("xyz")     // -1
```

Today `str.contains(s)` gives a bool but not the position. Add to `members.rs`.

---

### `str.replace_all(from, to)` — replace every occurrence

```aether
"aabbaa".replace_all("a", "x")   // "xxbbxx"
```

Current `replace` only replaces the first match. Add a `replace_all` arm in `members.rs`.

---

### Other I/O and stdlib

| Feature | API sketch |
|---------|-----------|
| stderr / log levels | `eprintln(msg)`, `log.warn(msg)` |
| Path helpers | `path.basename(p)`, `path.ext(p)` — `path_join` already implemented |
| Environment variables | `env("HOME")`, `env("PORT", "8080")` |
| Regular expressions | `re.match(pattern, text)`, `re.find_all(p, t)` |

---

## Tier 8 — Performance (interpreter internals)

### String interning
Every `Value::string()` allocates a new `Rc<String>`. Common dict keys (`"min"`, `"max"`, `"status"`, `"message"`) are heap-allocated fresh on every call.

**Fix:** A global intern table `HashMap<String, Rc<String>>` — identical strings share one allocation. No user-visible API change; `Value::string(s)` looks up before allocating.

**Impact:** 2–5× speedup on programs that do heavy dict access with string keys (JSON parsing, config handling, HTTP headers).

---

### `split()` lazy iterator / limit parameter
`str.split(delim)` allocates a full `Rc<RefCell<Vec<Value>>>` plus a new `Rc<String>` per part. For 10M rows in 1BRC, that is ~30M allocations from split alone.

**Two independent fixes:**
1. `str.split(delim, limit)` — stop after `limit` parts. Avoids allocating the tail when only the first part is needed.
2. `str.split_iter(delim)` — returns a lazy iterator that yields parts one at a time without building the full array.

---

## Tier 9 — Tooling (longer horizon)

| Tool | Description |
|------|-------------|
| ~~`aether fmt`~~ | ~~Auto-formatter for `.ae` files~~ — **shipped 2026-05-23** |
| ~~`aether check`~~ | ~~Type checker / linter without running~~ — **shipped 2026-05-23** (undefined variable detection) |
| ~~`aether test`~~ | ~~Built-in test runner~~ — **shipped 2026-05-23** |
| ~~REPL multi-line input~~ | ~~Paste multi-line code blocks in the REPL~~ — **shipped 2026-05-23** |
| Package manager | `aether.toml`, versioned deps, registry |
| ~~Debugger~~ | ~~Breakpoints, step-through, variable inspection~~ — **shipped 2026-04-30** |
| Bytecode compiler | Replace tree-walking interpreter; 5–20× speedup |
| LSP server | Language Server Protocol — autocomplete, go-to-def, inline errors in VS Code / Neovim |
| `aether watch` | Re-run file on save: `aether watch script.ae` |

### Known bug: `aether fmt` corrupts `&&` / `||` and drops comments

`aether fmt` prints `BinaryOp::And`/`BinaryOp::Or` as the bareword `and`/`or` (`src/formatter.rs`), but the lexer only accepts `&&`/`||` — running `fmt` on a file with logical operators produces a file that no longer parses. `fmt` also silently drops `//` comments from statement bodies. Found 2026-07-05 while writing `examples/random_demo.ae`. No existing example used `&&`/`||`, so this was previously undiscovered.

---

## Deferred (explicit non-goals for now)

- Concurrency primitives (channels, mutex) — async I/O pool covers most use cases
- Macros / metaprogramming — adds significant parser/evaluator complexity
- JIT compilation — follow bytecode compiler first

---
[← Memory Management](MEMORY_MANAGEMENT.md) &nbsp;&nbsp; [Home →](../index.md)
