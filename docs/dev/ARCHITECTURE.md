---
layout: default
title: "Aether — Architecture"
---

[Home](../index.md) › Developer Docs › Architecture

# Aether Architecture & Roadmap

This document provides a high-level overview of Aether's architecture, current status, and future roadmap.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Current Status](#current-status)
- [Roadmap](#roadmap)
- [Design Principles](#design-principles)
- [Resources](#resources)

---

> **📖 For Practical Development**: This document focuses on high-level architecture and long-term roadmap.
>
> For day-to-day development guidance, see:
> - **`CLAUDE.md`** (repo root) - Quick reference, project status, where to add features
> - **[DEVELOPMENT.md](DEVELOPMENT.md)** - TDD workflow, testing strategy, code style, common pitfalls

---

## Architecture Overview

Aether is a **tree-walking interpreter** written in Rust following a classic three-stage pipeline:

```
Source Code (.ae)
      ↓
   [Lexer]  ──→  Tokens
      ↓
   [Parser] ──→  Abstract Syntax Tree (AST)
      ↓
[Interpreter] ──→  Execution / Output
         ↕
   [I/O Thread Pool]  (async tasks)
```

### Core Components

| Component | Status | Purpose | Details |
|-----------|--------|---------|---------|
| **Lexer** | ✅ Complete | Tokenization | See [LEXER.md](LEXER.md) |
| **Parser** | ✅ Complete | Syntax analysis | See [PARSER.md](PARSER.md) |
| **Interpreter** | ✅ Complete | AST execution | See [INTERPRETER.md](INTERPRETER.md) |
| **I/O Thread Pool** | ✅ Complete | Concurrent I/O | See [ASYNC.md](../lang/ASYNC.md) |
| **Event Loop** | ✅ Complete | Callback-based async | See [EVENT_LOOP.md](EVENT_LOOP.md) |
| **REPL** | ✅ Complete | Interactive mode | See [REPL.md](../lang/REPL.md) |
| **Standard Library** | ✅ Complete | Core functions | See [STDLIB.md](../lang/STDLIB.md) |
| **Memory Management** | ✅ Complete | Memory management | See [MEMORY_MANAGEMENT.md](MEMORY_MANAGEMENT.md) |

### Project Structure

```
aether/
├── docs/              # Comprehensive documentation
├── stdlib/            # Standard library (written in Aether)
├── examples/          # Example programs
├── aether-plugin/     # Plugin SDK crate (FFI helpers, type conversion)
├── plugins/           # Example and real plugins (redis, v1/v2 protocol demos)
├── benches/           # Criterion benchmarks
├── tests/             # Integration tests, one file per feature
└── src/
    ├── lexer/         # Tokenization
    ├── parser/        # Parsing
    ├── interpreter/   # Execution
    │   ├── evaluator/
    │   │   ├── mod.rs          — Evaluator struct, constructors, call_main
    │   │   ├── expressions.rs  — eval_expr, eval_index, await_value
    │   │   ├── statements.rs   — exec_stmt_internal (all Stmt variants)
    │   │   ├── functions.rs    — eval_call, try_submit_io_task
    │   │   ├── members.rs      — eval_member, eval_method_call
    │   │   ├── modules.rs      — load_module, resolve_module_path
    │   │   └── operators.rs    — eval_unary, eval_binary
    │   ├── builtins.rs         — Built-in function dispatch
    │   ├── environment.rs      — Scope chain
    │   ├── event_loop.rs       — on_ready / event_loop
    │   ├── io_pool.rs          — I/O thread pool
    │   └── value.rs            — Value enum (27 variants)
    ├── repl.rs        # Interactive mode
    ├── checker.rs     # aether check — undefined-variable linter
    ├── formatter.rs   # aether fmt
    └── test_runner.rs # aether test — discovers *_test.ae files
```

## Current Status

**Phase**: 5 complete — the language is fully functional with async I/O and a rich stdlib.

**Tests**: 1225 passing (134 unit + 1091 integration), 0 failed, 0 ignored — measured 2026-07-29. Run `cargo test -- --test-threads=1` for current counts.

**Code quality**: `cargo clippy` clean (5 acceptable `mutable_key_type` warnings for HashSet).

### Feature Summary

| Area | Features |
|------|---------|
| **Core language** | int, float, string, bool, null, array, dict, set; all operators; let, if/else, while, for, break, continue, return |
| **Operators** | arithmetic, comparison, logical, bitwise `& \| ^ ~ << >>`, power `**`, ternary `?:`, null coalesce `??`, optional chain `?.` |
| **Pattern matching** | `match` statement — literals, wildcard `_`, binding, or-patterns `\|`, enum variant patterns |
| **Destructuring** | `let [a, b, ...rest] = arr`, `let {host, port: p = 5432} = dict` — array/dict, rest, rename, defaults |
| **Functions** | declarations, expressions, closures, optional params, recursion (default depth limit 100, override with `AETHER_CALL_DEPTH`) |
| **Strings** | indexing, interpolation `${expr}`, slicing `str[1:3]`, spread `[...arr]`, upper/lower/trim/split |
| **Collections** | array (push/pop/sort/concat/slice/spread), dict (keys/values/contains), set (union/intersection/difference/subset); reference semantics for array/dict/struct; `==` is identity; `.equals()` depth-1 structural; `copy()` depth-1 shallow clone; `id()` for object identity |
| **Error handling** | try/catch/finally/throw; `e.message`, `e.stack_trace`; stack frames include filename and line number |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias`; filesystem + embedded stdlib |
| **Structs** | fields, methods, `self` binding, mutable fields via RefCell; `.equals()` for depth-1 structural comparison |
| **Iterators** | `has_next()`, `next()`, for-in over array/dict/set/string/iterator |
| **Async/await** | `async fn`, `await expr`, Promise caching; `Promise.all`, `Promise.race`, `Promise.allSettled` |
| **I/O thread pool** | `set_workers(n)`, `AETHER_IO_WORKERS` env var; async `http_get`, `sleep`, `read_file`, `write_file`, `http_post` |
| **Event loop** | `on_ready(promise, callback)`, `event_loop()`; callback-based async; chained callbacks |
| **Null safety** | `??` null coalescing (short-circuit), `?.` optional member/method chaining |
| **JSON** | `json_parse()`, `json_stringify()` via serde_json |
| **CSV** | `csv_parse(str[, delim])`, `csv_stringify(rows[, delim])` |
| **HTTP** | `http_get(url)`, `http_post(url, body)` via reqwest (blocking or async) |
| **Time** | `clock()` (Unix epoch float), `sleep(secs)` |
| **Random** | `random()` (float in `[0, 1)`), `rand_int(n)` (int in `[0, n)`) via the `rand` crate |
| **TCP** | `tcp_listen(addr[, opts])`, `tcp_connect(addr)`; server events: `on_listen/connect/message/disconnect/error/timeout`, `accept()`, `close()`; client events: `on_connect/message/disconnect/error/timeout`, `start()`, `close()`, `write(data)`; event-driven via mio (single I/O thread, ~8–260 KB per connection) |
| **UDP** | `udp_bind(addr)`; `on_message(fn(data, addr) { })`, `send_to(data, addr)`, `listen()`, `close()`; connectionless datagram socket |
| **FFI / Plugins** | `load_plugin(path)` — load Rust shared libraries (`.so`/`.dylib`/`.dll`); call functions as methods; V1 protocol (int-only) and V2 protocol (`String`, `Vec<i64>`, `Vec<String>`, `HashMap<String,i64>`) auto-detected at load |
| **Number/string conversions** | `hex(n)`, `oct(n)`, `bin(n)`, `int(s, base)`, `base64_encode(s)`, `base64_decode(s)` |
| **String formatting** | `format(fmt, ...args)` — `{}` positional, `{:.2f}` precision, `{:>10}`/`{:<10}`/`{:^10}` width+alignment, `{:0>5d}` fill, `{:x}`/`{:o}`/`{:b}` bases |
| **Standard library** | See [STDLIB.md](../lang/STDLIB.md) for the stdlib reference and [BUILTINS.md](../lang/BUILTINS.md) for built-ins |
| **Testing framework** | assert_eq, assert_true/false/null, expect_error, test, test_summary |
| **REPL** | rustyline with history (`~/.aether_history`), tab-completion, `_help`/`_env`/`_exit`, multi-line input (`>>` / `..`) |
| **Configuration** | `AETHER_IO_WORKERS`, `AETHER_CALL_DEPTH`, `HOME` (see [CONFIGURATION.md](../lang/CONFIGURATION.md)) |
| **Tooling** | `aether ast` (AST printer), `aether fmt` (formatter), `aether test` (test runner), `aether check [file\|dir]` (undefined variable linter) |

### Test Coverage

Per-suite counts are not tracked here — they go stale the moment a test is added.
Get current numbers from the tool that knows them:

```bash
# Total, and the per-suite breakdown
cargo test -- --test-threads=1

# Just the totals
cargo test -- --test-threads=1 2>&1 | grep '^test result:'
```

Unit tests live beside the code they cover (`src/**/*_tests.rs`, run as
`unittests src/lib.rs`). Integration tests are one file per feature under
`tests/`. See [TESTING.md](TESTING.md) for the layout and conventions.

## Roadmap

### Completed Phases

| Phase | Description | Tests at completion |
|-------|-------------|-------------------|
| Phase 1 | Core interpreter (lexer, parser, evaluator, REPL) | 102 |
| Phase 2 | Essential features (collections, error handling, modules) | 147 |
| Phase 3 | Standard library (stdlib written in Aether) | 230 |
| Phase 4 | Advanced language features (structs, sets, iterators) | 314 |
| Phase 5 Sprint 1 | Testing framework | 333 |
| Phase 5 Sprint 2 | Advanced types (structs, sets, iterators) | 420 |
| Phase 5 Sprint 3 | Async/await + I/O pool | 476 |
| Phase 5 Sprint 4 | Error context + stack traces | ~547 |
| Phase 5 Sprint 5 | Null safety + Event loop | ~693 |
| Phase 5 Sprint 6 | Tooling (fmt, test, check, REPL multi-line) | ~1112 |

### Near-Term Backlog

See **[BACKLOG.md](BACKLOG.md)** for the full prioritised list (~30 features
across 6 tiers). Top items: variadic args, enums/tuples, named/default params.

Delivered since the list above was written: `match`, destructuring, `format()`,
TCP/UDP, and the FFI plugin system.

### Longer-Term

**Compiler Improvements**
- Bytecode compilation (instead of tree-walking)
- Constant folding and dead code elimination
- Tail call optimization

**Runtime Optimization**
- JIT compilation for hot paths
- Generational garbage collection
- String interning

## Design Principles

### Core Philosophy

1. **Simplicity First** — start with straightforward implementations; optimize later
2. **Test-Driven Development** — write tests before implementation; tests serve as documentation
3. **User Empowerment** — stdlib in Aether (users can read and extend); clear error messages
4. **Pragmatic Evolution** — ship working features quickly; iterate based on usage

### Technical Decisions

**Why Tree-Walking Interpreter?**
- Faster to implement and iterate on
- Easier to debug and extend
- Good enough performance for scripting use cases

**Why Rust?**
- Memory safety without GC overhead (for the interpreter itself)
- Strong type system catches bugs at compile time
- Excellent tooling (cargo, clippy, rustfmt)

**Why Rc for GC?**
- Simple reference counting
- Predictable, deterministic memory behavior
- No stop-the-world pauses
- Good enough for single-threaded interpreter

**Why Stdlib in Aether?**
- Validates language expressiveness ("dogfooding")
- User-readable, user-modifiable implementations
- Proves the language works for real code

**Why std::sync::mpsc for async?**
- No new dependencies — uses Rust stdlib channels
- Worker threads run blocking I/O; main thread stays single-threaded
- All `Value`/`Rc<T>` objects stay on the main thread (thread-safe by design)

## Resources

### Documentation

**Core Implementation:**
- **[DESIGN.md](DESIGN.md)** — Complete language specification
- **[DEVELOPMENT.md](DEVELOPMENT.md)** — Development guidelines and best practices
- **[LEXER.md](LEXER.md)** — Tokenization implementation
- **[PARSER.md](PARSER.md)** — Syntax analysis implementation
- **[INTERPRETER.md](INTERPRETER.md)** — Execution engine implementation
- **[REPL.md](../lang/REPL.md)** — Interactive mode implementation
- **[STDLIB.md](../lang/STDLIB.md)** — Standard library design
- **[MEMORY_MANAGEMENT.md](MEMORY_MANAGEMENT.md)** — Garbage collection architecture

**Language Features** (all in `docs/lang/`):
- **[STRUCT.md](../lang/STRUCT.md)** — User-defined types with fields and methods
- **[ERROR_HANDLING.md](../lang/ERROR_HANDLING.md)** — try/catch/finally/throw
- **[STRINGS.md](../lang/STRINGS.md)** — String indexing, interpolation, slicing
- **[ASYNC.md](../lang/ASYNC.md)** — Async/await and I/O thread pool
- **[EVENT_LOOP.md](EVENT_LOOP.md)** — Callback-based async
- **[JSON.md](../lang/JSON.md)** — JSON parsing and serialization
- **[TIME.md](../lang/TIME.md)** — Time functions (clock, sleep)
- **[HTTP.md](../lang/HTTP.md)** — HTTP client functions
- **[MODULE_SYSTEM.md](../lang/MODULE_SYSTEM.md)** — Import and module loading
- **[ITERATORS.md](../lang/ITERATORS.md)** — Iterator protocol
- **[BACKLOG.md](BACKLOG.md)** — Feature backlog

### External Resources
- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- [Rust Programming Language Book](https://doc.rust-lang.org/book/)

### Quick Links
- **Main README**: `README.md` (repo root)
- **Project Guide**: `CLAUDE.md` (repo root)
- **Examples**: `examples/` (repo root)
- **Standard Library**: `stdlib/` (repo root)

---

**Last Updated**: July 29, 2026
**Current Phase**: Phase 5 Complete ✅
**Test Count**: see [Current Status](#current-status)

---
[← Language Design](DESIGN.md) &nbsp;&nbsp; [Development Guide →](DEVELOPMENT.md)
