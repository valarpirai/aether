---
layout: default
title: Aether: General purpose programming language
---

# Aether Programming Language

A general-purpose, dynamically typed programming language implemented in Rust — a fully-working tree-walking interpreter with async I/O, structs, iterators, and a self-hosted standard library.

## Features

- **Dynamic Typing** — Runtime type checking with clear error messages
- **Automatic Memory Management** — Reference-counted garbage collection (no GC pauses)
- **Async/Await** — `async fn`, `await`, `Promise.all`, configurable I/O thread pool
- **First-Class Functions** — Closures, optional parameters, function expressions
- **User-Defined Types** — Structs with fields, methods, and `self` binding
- **Error Handling** — `try/catch/throw` with `e.message` and `e.stack_trace`
- **Module System** — `import`, `from ... import`, aliases, filesystem resolution
- **Collections** — Arrays, dicts, sets with comprehensive methods
- **File I/O** — Read/write files, line-by-line streaming, binary I/O
- **Rich Standard Library** — 40+ functions written in Aether itself
- **Interactive REPL** — Line editing with history and tab-completion

## Quick Example

```aether
fn main() {
    // Functional stdlib
    let numbers = range(1, 11)
    let total = sum(filter(map(numbers, fn(x) { return x * x }), fn(x) { return x % 2 == 0 }))
    println("sum of even squares:", total)   // 220

    // File I/O
    let lines = read_lines("data.csv")
    println("rows:", len(lines))

    // Error handling with stack traces
    try {
        throw "something went wrong"
    } catch(e) {
        println("caught:", e.message)
        println(e.stack_trace)
    }
}
```

## Status

- **Phase**: 5 Complete ✅
- **Tests**: ~564 passing (99 unit + ~465 integration)
- **Code Quality**: 0 clippy warnings
- **Documentation**: 20+ comprehensive guides

## Quick Start

### Building from Source

```bash
git clone https://github.com/valarpirai/aether.git
cd aether
cargo build --release
./target/release/aether          # REPL
./target/release/aether hello.ae # run a file
```

### Your First Program

```aether
fn main() {
    let name = "World"
    println("Hello, ${name}!")

    let nums = range(1, 6)
    for n in nums {
        println(n, "squared =", n * n)
    }
}
```

### Async I/O

```aether
fn main() {
    set_workers(4)
    let p1 = http_get("https://httpbin.org/get")
    let p2 = http_get("https://httpbin.org/ip")
    let results = await Promise.all([p1, p2])
    println(results[0])
}
```

## Documentation

### Language Reference
- [Language Design](DESIGN.html) — Types, syntax, operators, all features
- [Architecture](ARCHITECTURE.html) — System design and roadmap
- [Configuration](CONFIGURATION.html) — Env vars, runtime knobs, compile-time constants
- [Backlog](BACKLOG.html) — Planned features (6 tiers, ~30 items)

### Implementation Guides
- [Lexer](LEXER.html) — Tokenization (14 tests)
- [Parser](PARSER.html) — Recursive descent parsing (53 tests)
- [Interpreter](INTERPRETER.html) — Tree-walking evaluator
- [REPL](REPL.html) — Interactive mode
- [Standard Library](STDLIB.html) — Self-hosted stdlib
- [Garbage Collection](GC_DESIGN.html) — Rc-based memory management
- [Module System](MODULE_SYSTEM.html) — Import mechanism

### Language Features
- [Async/Await](ASYNC.html) — Promises, `async fn`, I/O thread pool
- [Iterators](ITERATOR_PROTOCOL.html) — `has_next()` / `next()` protocol
- [Structs](STRUCT.html) — User-defined types with methods
- [Error Handling](ERROR_HANDLING.html) — Try/catch/throw with stack traces
- [String Features](STRING_FEATURES.html) — Indexing, interpolation, slicing, methods
- [JSON](JSON.html) — Parsing and serialization
- [HTTP](HTTP.html) — `http_get()`, `http_post()`
- [Time](TIME.html) — `clock()`, `sleep()`

### Contributing
- [Development Guide](DEVELOPMENT.html) — Post-feature checklist, TDD workflow, memory leak detection
- [Testing Guide](TESTING.html) — Test organisation and patterns

## What's Working

| Area | Features |
|------|---------|
| **Core language** | int, float, string, bool, null, array, dict, set; all operators; let, if/else, while, for, break, continue, return |
| **Functions** | declarations, expressions, closures, optional params, recursion |
| **Strings** | indexing, interpolation `${expr}`, slicing, `contains`, `index_of`, `replace`, `upper`/`lower`/`trim`/`split` |
| **Collections** | array (push/pop/sort/slice/spread), dict (keys/values/contains), set (union/intersection/difference) |
| **File I/O** | `read_file`, `write_file`, `read_lines`, `append_file`, `lines_iter` (streaming), `read_bytes`, `write_bytes`, `file_exists`, `is_file`, `is_dir`, `mkdir` |
| **Error handling** | try/catch/throw; `e.message`, `e.stack_trace`; frames include filename and line |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias` |
| **Structs** | fields, methods, `self` binding, mutable fields |
| **Iterators** | `has_next()`, `next()`, for-in over array/dict/set/string/iterator |
| **Async/await** | `async fn`, `await expr`, Promise caching, `Promise.all` |
| **I/O thread pool** | `set_workers(n)`, `AETHER_WORKERS`; async http, sleep, file I/O |
| **JSON** | `json_parse()`, `json_stringify()` |
| **HTTP** | `http_get(url)`, `http_post(url, body)` |
| **Standard library** | range, enumerate, map, filter, reduce, find, every, some, abs, min, max, sum, clamp, join, repeat, reverse, starts_with, ends_with |
| **Testing framework** | assert_eq, assert_true/false/null, expect_error, test, test_summary |
| **REPL** | rustyline with history, tab-completion, `_help`/`_env`/`_exit` |

## Examples

Browse the [examples directory](EXAMPLES.html) or jump straight to a topic:

| Example | What it shows |
|---------|--------------|
| [Hello World](EXAMPLES.html#hello) | First program, interpolation |
| [File Utilities](EXAMPLES.html#file-utilities) | basename, grep, head/tail, CSV reader |
| [String Utilities](EXAMPLES.html#string-utilities) | All string operations |
| [Shapes (Structs)](EXAMPLES.html#shapes) | User-defined types with methods |
| [Error Context](EXAMPLES.html#error-context) | Stack traces in catch blocks |
| [Async / Concurrent I/O](EXAMPLES.html#async) | Promise.all, thread pool |
| [Data Processing](EXAMPLES.html#data-processing) | Functional pipeline |
| [Task Manager](EXAMPLES.html#task-manager) | Real-world struct usage |
| [Collections](EXAMPLES.html#collections) | Arrays, dicts, sets |

## License

MIT License — see [LICENSE](https://github.com/valarpirai/aether/blob/main/LICENSE)

---

**Last Updated**: 2026-04-28  
**Version**: 0.1.0  
**Status**: Active Development

[Language Reference](DESIGN.html){: .btn .btn-primary}
[View on GitHub](https://github.com/valarpirai/aether){: .btn .btn-outline}
