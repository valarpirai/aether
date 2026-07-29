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
- **Event Loop** — `on_ready(promise, callback)`, `event_loop()` for Node.js-style async
- **First-Class Functions** — Closures, optional parameters, function expressions
- **User-Defined Types** — Structs with fields, methods, and `self` binding
- **Error Handling** — `try/catch/finally/throw` with `e.message` and `e.stack_trace`
- **Null Safety** — `??` null coalescing and `?.` optional chaining
- **Module System** — `import`, `from ... import`, aliases, filesystem resolution
- **Collections** — Arrays, dicts, sets with reference semantics; `.equals()`, `copy()`, `id()`
- **Pattern Matching** — `match` with literals, wildcards, bindings, and or-patterns
- **Destructuring** — Array and dict destructuring with rest, rename, and defaults
- **Operators** — Bitwise `& | ^ ~ << >>`, power `**`, ternary `?:`, null coalesce `??`
- **File I/O** — Read/write files, directory listing, path utilities
- **Multi-line Strings** — Triple-quoted `"""..."""` strings with raw content
- **TCP/UDP** — `tcp_listen()`, `tcp_connect()`, `udp_bind()`; event-driven via mio, single I/O thread
- **FFI / Plugins** — `load_plugin()` loads Rust shared libraries; call functions as methods
- **CSV** — `csv_parse()` and `csv_stringify()` built-in
- **Number Conversions** — `hex()`, `oct()`, `bin()`, `int(s, base)`, `base64_encode()`, `base64_decode()`
- **Rich Standard Library** — 50+ functions written in Aether itself
- **Interactive REPL** — Line editing with history and tab-completion

## Quick Example

```aether
fn main() {
    // Functional stdlib
    let numbers = range(1, 11)
    let total = sum(filter(map(numbers, fn(x) { return x * x }), fn(x) { return x % 2 == 0 }))
    println("sum of even squares:", total)   // 220

    // Null safety: ?? and ?.
    let name = null
    println(name ?? "anonymous")          // anonymous
    println(name?.upper() ?? "UNKNOWN")   // UNKNOWN

    // Error handling with finally
    let f = null
    try {
        throw "something went wrong"
    } catch(e) {
        println("caught:", e.message)
    } finally {
        println("cleanup always runs")
    }

    // Multi-line string
    let query = """
        SELECT * FROM users
        WHERE active = true
    """
    println(query)
}
```

## Status

- **Phase**: 5 Complete ✅
- **Version**: 0.2.1
- **Tests**: ~1140 passing (134 unit + ~1017 integration)
- **Code Quality**: 0 clippy warnings
- **Documentation**: 20+ comprehensive guides

## Quick Start

### Download Pre-built Binary

Grab the latest binary from the [Releases page](https://github.com/valarpirai/aether/releases/latest):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `aether-macos-arm64` |
| macOS (Intel) | `aether-macos-x86` |
| Linux x86_64 | `aether-linux-x86` |
| Windows x86_64 | `aether-windows-x86.exe` |

```bash
# macOS / Linux
chmod +x aether-macos-arm64
./aether-macos-arm64          # REPL
./aether-macos-arm64 hello.ae # run a file
```

### Install via Cargo

```bash
cargo install aether-lang
aether          # REPL
aether hello.ae # run a file
```

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

### Writing programs in Aether

Start here if you want to use the language — build scripts, automate tasks, or learn Aether.

| Guide | What it covers |
|-------|---------------|
| [REPL](lang/REPL.md) | Interactive shell and running `.ae` files |
| [Strings](lang/STRINGS.md) | Literals, indexing, slicing, interpolation, methods |
| [Format](lang/FORMAT.md) | format() — `{}` placeholders, width, alignment, precision |
| [Error Handling](lang/ERROR_HANDLING.md) | try/catch/finally/throw with stack traces |
| [Structs](lang/STRUCT.md) | User-defined types with fields and methods |
| [Modules](lang/MODULE_SYSTEM.md) | import, from…import, stdlib modules |
| [Built-in Functions](lang/BUILTINS.md) | print, len, type, file I/O, sockets, conversions |
| [Standard Library](lang/STDLIB.md) | range, map, filter, reduce, math, string, testing |
| [Iterators](lang/ITERATORS.md) | has_next() / next() protocol, custom iterators |
| [JSON](lang/JSON.md) | json_parse(), json_stringify() |
| [CSV](lang/CSV.md) | csv_parse(), csv_stringify() |
| [HTTP](lang/HTTP.md) | http_get(), http_post() |
| [Time](lang/TIME.md) | clock(), sleep() |
| [Random](lang/RANDOM.md) | random(), rand_int(n) |
| [Async/Await](lang/ASYNC.md) | async fn, await, .then(), Promise.all/race/allSettled, I/O pool |
| [TCP](lang/TCP.md) | tcp_listen(), tcp_connect(), server/client lifecycle events, UDP |
| [Plugins](lang/PLUGINS.md) | load_plugin() — call Rust shared libraries from Aether |
| [Plugin Guide](lang/PLUGIN_GUIDE.md) | Step-by-step: write a plugin and wrap a Rust crate |
| [Configuration](lang/CONFIGURATION.md) | Env vars and runtime configuration |

### Hacking on Aether's interpreter

Start here if you want to extend the language, fix a bug, or understand how the interpreter is built. Assumes familiarity with Rust.

| Guide | What it covers |
|-------|---------------|
| [Development Guide](dev/DEVELOPMENT.md) | Post-feature checklist, TDD workflow, code organisation |
| [Architecture](dev/ARCHITECTURE.md) | System design, module layout, and roadmap |
| [Testing Guide](dev/TESTING.md) | Running tests, writing integration tests, debugging failures |
| [Language Design](dev/DESIGN.md) | Complete language specification and design decisions |
| [Lexer](dev/LEXER.md) | Tokenisation — `src/lexer/` |
| [Parser](dev/PARSER.md) | Recursive descent parsing — `src/parser/` |
| [Interpreter](dev/INTERPRETER.md) | Tree-walking evaluator — `src/interpreter/evaluator/` |
| [Memory Management](dev/MEMORY_MANAGEMENT.md) | Rc-based memory management and cycle avoidance |
| [Event Loop Internals](dev/EVENT_LOOP.md) | on_ready, event_loop, queue controls |
| [Async I/O Architecture](dev/ASYNC_IO.md) | IoPool, EventLoopQueue, TCP dispatch loop, await vs .then() |
| [TCP/UDP Internals](dev/TCP_UDP.md) | mio I/O loop, state types, channels, SIGINT, UDP design |
| [Backlog](dev/BACKLOG.md) | Planned features and open design questions |

## What's Working

| Area | Features |
|------|---------|
| **Core language** | int, float, string, bool, null, array, dict, set; all operators; let, if/else, while, for, break, continue, return |
| **Operators** | arithmetic, comparison, logical, bitwise `& \| ^ ~ << >>`, power `**`, ternary `?:`, null coalesce `??`, optional chain `?.` |
| **Pattern matching** | `match` — literals, wildcard `_`, binding, or-patterns `\|`, enum variant patterns |
| **Destructuring** | `let [a, b, ...rest] = arr`, `let {host, port: p = 5432} = dict` — rest, rename, defaults |
| **Functions** | declarations, expressions, closures, optional params, recursion (depth limit configurable) |
| **Strings** | indexing, interpolation `${expr}`, slicing, multi-line `"""..."""`, `contains`, `index_of`, `replace`, `upper`/`lower`/`trim`/`split` |
| **Null safety** | `??` null coalescing, `?.` optional member access, `?.` optional method call |
| **Collections** | array (push/pop/sort/slice/spread), dict (keys/values/contains), set (union/intersection/difference); reference semantics; `.equals()` depth-1 structural; `copy()` shallow clone; `id()` identity |
| **File I/O** | `read_file`, `write_file`, `read_lines`, `append_file`, `lines_iter`, `read_bytes`, `write_bytes`, `file_exists`, `is_file`, `is_dir`, `mkdir`, `list_dir`, `path_join`, `rename`, `rm` |
| **Error handling** | try/catch/finally/throw; `e.message`, `e.stack_trace`; frames include filename and line |
| **Loops** | while, for-in, labeled `break`/`continue` for nested loops |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias` |
| **Structs** | fields, methods, `self` binding, mutable fields |
| **Iterators** | `has_next()`, `next()`, for-in over array/dict/set/string/iterator |
| **Async/await** | `async fn`, `await expr`, Promise caching, `Promise.all` |
| **Event loop** | `on_ready(promise, callback)`, `event_loop()`; chained callbacks; Node.js-style concurrency |
| **I/O thread pool** | `set_workers(n)`, `AETHER_IO_WORKERS`; async http, sleep, file I/O |
| **JSON** | `json_parse()`, `json_stringify()` |
| **HTTP** | `http_get(url [, opts])`, `http_post(url, body [, opts])` — per-request timeout and user-agent |
| **TCP** | `tcp_listen(addr[, opts])`, `tcp_connect(addr)`; server events: `on_listen/connect/message/disconnect/error/timeout`; client events: `on_connect/message/disconnect`; event-driven via mio |
| **UDP** | `udp_bind(addr)`; `on_message(fn(data, addr))`, `send_to(data, addr)`, `listen()`, `close()`; connectionless datagrams |
| **Number conversions** | `hex(n)`, `oct(n)`, `bin(n)`, `int(s, base)`, `base64_encode(s)`, `base64_decode(s)` |
| **CSV** | `csv_parse(str)`, `csv_stringify(rows)` |
| **FFI / Plugins** | `load_plugin(path)` — load Rust shared libraries and call functions as methods; int/string/array/dict args and returns |
| **Standard library** | range, enumerate, map, filter, reduce, find, every, some, abs, min, max, sum, clamp, join, repeat, reverse, starts_with, ends_with, chunk, partition, zip_longest, uniq_by, pad_left, pad_right, factorial, sin, cos, tan, and more |
| **Testing framework** | assert_eq, assert_true/false/null, expect_error, test, test_summary |
| **REPL** | rustyline with history, tab-completion, `_help`/`_env`/`_exit`, multi-line input (`>>` / `..`), `--version`/`--help` flags |
| **Tooling** | `aether fmt` — auto-formatter; `aether test` — test runner; `aether check [file\|dir]` — undefined variable linter with directory scanning and `main()` entry-point enforcement |

## Examples

Browse the [examples directory](lang/EXAMPLES.md) or jump straight to a topic:

| Example | What it shows |
|---------|--------------|
| [Hello World](lang/EXAMPLES.md#hello) | First program, interpolation |
| [Null Safety](lang/EXAMPLES.md#null-safety) | `??` and `?.` operators |
| [Multi-line Strings](lang/EXAMPLES.md#multiline-strings) | Triple-quoted strings |
| [Error Handling](lang/EXAMPLES.md#error-handling) | try/catch/finally, stack traces |
| [File Utilities](lang/EXAMPLES.md#file-utilities) | list_dir, path_join, rename, rm |
| [Shapes (Structs)](lang/EXAMPLES.md#shapes) | User-defined types with methods |
| [Async / Concurrent I/O](lang/EXAMPLES.md#async) | Promise.all, thread pool |
| [Event Loop](lang/EXAMPLES.md#event-loop) | on_ready, chained callbacks |
| [TCP Server](lang/TCP.md#server) | tcp_listen, lifecycle events, echo server |
| [CSV Processing](lang/CSV.md) | csv_parse, csv_stringify, roundtrip |
| [Data Processing](lang/EXAMPLES.md#data-processing) | Functional pipeline |
| [Collections](lang/EXAMPLES.md#collections) | Arrays, dicts, sets |

## License

MIT License — see [LICENSE](https://github.com/valarpirai/aether/blob/main/LICENSE)

---

**Last Updated**: 2026-07-24  
**Version**: 0.2.1  
**Status**: Active Development

[View on GitHub](https://github.com/valarpirai/aether){: .btn .btn-outline}
