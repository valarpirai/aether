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
| Event loop — `on_ready(promise, callback)`, `event_loop()` | 2026-04-29 |

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

## Tier 3 — Operators and syntax sugar

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

## Tier 4 — Type system

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

## Tier 5 — I/O and stdlib

| Feature | API sketch |
|---------|-----------|
| stderr / log levels | `eprintln(msg)`, `log.warn(msg)` |
| Directory operations | `ls(path)`, `mkdir(path)`, `rm(path)`, `exists(path)` |
| Path helpers | `path.join(a, b)`, `path.basename(p)`, `path.ext(p)` |
| Environment variables | `env("HOME")`, `env("PORT", "8080")` |
| Regular expressions | `re.match(pattern, text)`, `re.find_all(p, t)` |
| Raw strings | `r"no\escape\needed"` |

---

## Tier 6 — Tooling (longer horizon)

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
