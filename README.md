# Aether Programming Language

[![crates.io](https://img.shields.io/crates/v/aether-lang.svg)](https://crates.io/crates/aether-lang)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Website**: https://valarpirai.github.io/aether/ &nbsp;|&nbsp;
**Crate**: [aether-lang on crates.io](https://crates.io/crates/aether-lang) &nbsp;|&nbsp;
**Docs**: [Language Reference](https://valarpirai.github.io/aether/lang/) · [Developer Docs](https://valarpirai.github.io/aether/dev/)

A general-purpose, dynamically typed programming language implemented in Rust — a fully-working tree-walking interpreter with a rich standard library, async I/O, structs, and a module system.

---

## Features

| Area | What's included |
|------|----------------|
| **Types** | `int`, `float`, `string`, `bool`, `null`, `array`, `dict`, `set` |
| **Operators** | arithmetic, comparison, logical, bitwise `& \| ^ ~ << >>`, power `**`, ternary `?:`, null coalesce `??`, optional chain `?.` |
| **Functions** | declarations, expressions, closures, optional params, recursion |
| **Strings** | interpolation `"Hello ${name}"`, indexing, slicing, upper/lower/trim/split |
| **Collections** | array (push/pop/sort/slice/spread), dict (keys/values/contains), set (union/intersection/difference) |
| **Pattern matching** | `match` with literals, wildcards `_`, bind, enum variants, or-patterns `\|` |
| **Destructuring** | `let [a, b, ...rest] = arr`, `let {host, port: p = 5432} = dict` |
| **Error handling** | `try`/`catch`/`throw` with `e.message` and `e.stack_trace` |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias` |
| **Structs** | fields, methods, `self` binding |
| **Async/await** | `async fn`, `await`, `Promise.all`, `Promise.race`, `Promise.allSettled`, I/O thread pool |
| **Null safety** | `??` null coalescing, `?.` optional chaining |
| **Standard library** | 70+ functions: range, map, filter, reduce, math, string, testing and more |
| **Tooling** | `aether fmt` formatter, `aether test` runner, `aether check` linter |
| **REPL** | history, tab-completion, multi-line input |

---

## Installation

### From crates.io

```bash
cargo install aether-lang
aether --version   # aether 0.2.0
```

### From source

```bash
git clone https://github.com/valarpirai/aether.git
cd aether
cargo build --release
./target/release/aether --version
```

---

## Quick Start

Create `hello.ae`:

```aether
fn main() {
    let name = "Aether"
    println("Hello, ${name}!")

    let numbers = range(1, 6)
    let squares = map(numbers, fn(x) { return x * x })
    println("Squares:", squares)  // [1, 4, 9, 16, 25]
}
```

Run it:

```bash
aether hello.ae
```

---

## Language Tour

### Variables and types

```aether
let x = 42
let pi = 3.14
let name = "Alice"
let active = true
let nothing = null
```

### Functions and closures

```aether
fn add(a, b) {
    return a + b
}

let multiply = fn(a, b) { return a * b }

fn make_adder(n) {
    return fn(x) { return x + n }
}

let add5 = make_adder(5)
println(add5(3))  // 8
```

### Collections

```aether
let arr = [1, 2, 3]
arr.push(4)
println(arr.len())   // 4

let d = {"key": "value", "count": 42}
println(d["key"])    // value

let s = set([1, 2, 2, 3])
println(s)           // {1, 2, 3}
```

### Control flow

```aether
// if / else
if (x > 0) {
    println("positive")
} else if (x < 0) {
    println("negative")
} else {
    println("zero")
}

// for loop
for item in arr {
    println(item)
}

// match
match status {
    "ok"  => println("success")
    "err" => println("failed")
    _     => println("unknown")
}
```

### Error handling

```aether
fn divide(a, b) {
    if (b == 0) { throw "division by zero" }
    return a / b
}

try {
    println(divide(10, 0))
} catch(e) {
    println("Error:", e.message)
    println(e.stack_trace)
}
```

### Structs

```aether
struct Point {
    x
    y
    fn distance() {
        return sqrt(self.x * self.x + self.y * self.y)
    }
}

let p = Point { x: 3, y: 4 }
println(p.distance())  // 5.0
```

### Async / await

```aether
async fn fetch(url) {
    let body = await http_get(url)
    return json_parse(body)
}

fn main() {
    set_workers(4)
    let data = await fetch("https://api.example.com/data")
    println(data)
}
```

### Modules

```aether
import math
from testing import test, assert_eq, test_summary

fn main() {
    let results = []
    results.push(test("sin", fn() {
        assert_eq(math.sin(0), 0)
    }))
    test_summary(results)
}
```

---

## CLI Reference

```
aether                        # Start interactive REPL
aether <file.ae>              # Run a script
aether fmt [--check] <file>   # Format source (--check: exit 1 if unformatted)
aether test [dir|file]        # Discover and run *_test.ae files
aether check <file>           # Lint for undefined variables
aether --version              # Print version
aether --help                 # Print help
```

### REPL

```
>> let x = 10
>> x * 2
20
>> fn greet(name) {
..     return "Hello, " + name
.. }
>> greet("World")
Hello, World
>> _help    # show commands
>> _env     # show bindings
>> _exit    # quit (or Ctrl+D)
```

Multi-line blocks continue automatically with `.. `. Press **Ctrl+C** to cancel.

### Formatter

```bash
aether fmt myfile.ae           # format in place
aether fmt --check myfile.ae   # CI-safe check (exit 1 if not canonical)
```

### Test runner

Write test files named `*_test.ae` using the `testing` stdlib module:

```aether
from testing import test, assert_eq, test_summary

fn main() {
    let results = []
    results.push(test("addition", fn() {
        assert_eq(1 + 2, 3)
    }))
    test_summary(results)
}
```

Run them:

```bash
aether test                     # discover all *_test.ae in current dir
aether test examples/           # run tests in a specific directory
aether test examples/math_test.ae  # run a single file
```

### Linter

```bash
aether check myfile.ae
# myfile.ae: ok                   (exit 0)
# myfile.ae:12: undefined variable 'resutls'  (exit 1)
```

---

## Standard Library

| Module | Functions |
|--------|-----------|
| **Core** | `range()`, `enumerate()` |
| **Collections** | `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`, `sort()`, `concat()`, `zip()`, `flatten()`, `flat_map()`, `take()`, `drop()`, `chunk()`, `partition()`, `uniq()`, `uniq_by()`, `group_by()`, `count_by()`, `zip_longest()`, `first()`, `last()` |
| **Math** | `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`, `floor()`, `ceil()`, `round()`, `sqrt()`, `pow()`, `log()`, `exp()`, `sin()`, `cos()`, `tan()`, `degrees()`, `radians()`, `hypot()`, `factorial()`, `gcd()`, `lcm()`, `pi`, `e`, `tau` |
| **String** | `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`, `contains()`, `index_of()`, `replace()`, `count()`, `pad_left()`, `pad_right()`, `strip_prefix()`, `strip_suffix()`, `is_alpha()`, `is_digit()`, `is_space()` |
| **Testing** | `test()`, `test_summary()`, `assert_eq()`, `assert_true()`, `assert_false()`, `assert_null()`, `assert_not_null()`, `expect_error()` |

Built-in functions also include: `len()`, `type()`, `str()`, `int()`, `float()`, `bool()`, `print()`, `println()`, `input()`, `clock()`, `sleep()`, `json_parse()`, `json_stringify()`, `http_get()`, `http_post()`, `read_file()`, `write_file()`, and more.

---

## Documentation

### Language Reference

| Document | Description |
|----------|-------------|
| [Strings](https://valarpirai.github.io/aether/lang/STRINGS.html) | Literals, indexing, slicing, interpolation, methods |
| [Destructuring](https://valarpirai.github.io/aether/lang/DESTRUCTURING.html) | Array and dict destructuring, rest, rename, defaults |
| [Structs](https://valarpirai.github.io/aether/lang/STRUCT.html) | User-defined types with fields and methods |
| [Error Handling](https://valarpirai.github.io/aether/lang/ERROR_HANDLING.html) | try/catch/throw with stack traces |
| [Async](https://valarpirai.github.io/aether/lang/ASYNC.html) | async fn, await, Promise combinators, I/O pool |
| [Iterators](https://valarpirai.github.io/aether/lang/ITERATORS.html) | Iterator protocol and built-in iterators |
| [Module System](https://valarpirai.github.io/aether/lang/MODULE_SYSTEM.html) | import, from…import, stdlib modules |
| [Standard Library](https://valarpirai.github.io/aether/lang/STDLIB.html) | Full stdlib reference |
| [HTTP](https://valarpirai.github.io/aether/lang/HTTP.html) | http_get(), http_post() |
| [JSON](https://valarpirai.github.io/aether/lang/JSON.html) | json_parse(), json_stringify() |
| [REPL](https://valarpirai.github.io/aether/lang/REPL.html) | Interactive REPL and multi-line input |
| [Configuration](https://valarpirai.github.io/aether/lang/CONFIGURATION.html) | Env vars and runtime knobs |

### Developer Docs

| Document | Description |
|----------|-------------|
| [Architecture](https://valarpirai.github.io/aether/dev/ARCHITECTURE.html) | System design and roadmap |
| [Design](https://valarpirai.github.io/aether/dev/DESIGN.html) | Complete language specification |
| [Development Guide](https://valarpirai.github.io/aether/dev/DEVELOPMENT.html) | TDD workflow, post-feature checklist |
| [Testing Guide](https://valarpirai.github.io/aether/dev/TESTING.html) | Running tests, writing integration tests |
| [Memory Management](https://valarpirai.github.io/aether/dev/MEMORY_MANAGEMENT.html) | Rc-based GC and design rationale |
| [Backlog](https://valarpirai.github.io/aether/dev/BACKLOG.html) | Prioritised feature backlog |

---

## Project Status

**Version**: 0.2.0 — tooling milestone complete

- ~1112 tests passing (134 unit + ~978 integration)
- `cargo clippy` clean
- Published on [crates.io](https://crates.io/crates/aether-lang)

---

## Contributing

1. Enable the pre-commit hook: `git config core.hooksPath .githooks`
2. Follow the [Development Guide](https://valarpirai.github.io/aether/dev/DEVELOPMENT.html)
3. Write tests first (TDD)
4. Run `cargo fmt && cargo clippy && cargo test -- --test-threads=1` before committing

---

## License

MIT — see [LICENSE](LICENSE) for details.
