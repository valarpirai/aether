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
> - **[CLAUDE.md](../CLAUDE.md)** - Quick reference, project status, where to add features
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
| **I/O Thread Pool** | ✅ Complete | Concurrent I/O | See [ASYNC.md](ASYNC.md) |
| **Event Loop** | ✅ Complete | Callback-based async | See [EVENT_LOOP.md](EVENT_LOOP.md) |
| **REPL** | ✅ Complete | Interactive mode | See [REPL.md](REPL.md) |
| **Standard Library** | ✅ Complete | Core functions | See [STDLIB.md](STDLIB.md) |
| **Garbage Collection** | ✅ Complete | Memory management | See [GC_DESIGN.md](GC_DESIGN.md) |

### Project Structure

```
aether/
├── docs/              # Comprehensive documentation
├── stdlib/            # Standard library (written in Aether)
├── examples/          # Example programs
├── tests/             # Integration tests (~559 tests)
└── src/
    ├── lexer/         # Tokenization (14 unit tests)
    ├── parser/        # Parsing (53 unit tests)
    ├── interpreter/   # Execution (17 unit tests)
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
    │   └── value.rs            — Value enum (16 variants)
    └── repl/          # Interactive mode
```

## Current Status

### Phase 5 Complete ✅

**Test Coverage**: ~693 tests passing (134 unit + ~559 integration)
**Code Quality**: cargo clippy clean (5 acceptable `mutable_key_type` warnings for HashSet)

### What's Implemented

**Core Language** ✅
- Dynamic typing with runtime type checking
- First-class functions with closures and function expressions
- Primitives: int, float, string (UTF-8), bool, null
- Collections: array, dict (ordered insertion), set (unique, unordered)
- Automatic memory management (Rc-based GC)
- C-like syntax (curly braces, no semicolons)
- String interpolation: `"Hello ${name}"`
- String indexing: `str[0]`, slicing: `str[1:3]`
- Multi-line strings: `"""..."""`

**Control Flow** ✅
- if/else conditionals
- while loops
- for-in loops (array, dict, set, string, iterator)
- Labeled `break`/`continue` for nested loops
- return statements

**Error Handling** ✅
- `try { ... } catch(e) { ... } finally { ... }`
- `throw value` — throw any value as an error
- `e.message`, `e.stack_trace` — error object properties
- Stack traces include filename and line numbers

**Null Safety** ✅
- `??` null coalescing (short-circuits if left side is non-null)
- `?.` optional member access — returns null instead of throwing
- `?.` optional method call — returns null instead of throwing

**Structs** ✅
- User-defined types with fields and methods
- `self` binding in methods
- Mutable fields via `RefCell`

**Async / Await** ✅
- `async fn` — returns a Promise
- `await expr` — resolves a Promise (polls until ready)
- `Promise.all([p1, p2, ...])` — concurrent resolution
- Promise result caching

**I/O Thread Pool** ✅
- `set_workers(n)` — configure pool size at runtime
- `AETHER_IO_WORKERS` env var — set at startup
- Async: `http_get`, `http_post`, `sleep`, `read_file`, `write_file`
- Per-request HTTP options: `{timeout: N, user_agent: "..."}`
- Env-var defaults: `AETHER_HTTP_TIMEOUT`, `AETHER_HTTP_USER_AGENT`

**Event Loop** ✅
- `on_ready(promise, callback)` — register callback
- `event_loop([timeout_secs])` — run until all callbacks resolve
- Chained callbacks (register from inside a callback)
- `set_queue_limit(n)` — backpressure cap
- `set_task_timeout(secs|null)` — per-callback deadline

**Module System** ✅
- `import module` — namespace import
- `from module import fn1, fn2` — selective import
- `import module as alias` — aliased import
- User `.ae` modules from filesystem + embedded stdlib

**Built-in Functions** ✅
- I/O: `print`, `println`, `input`, `read_file`, `write_file`, `read_lines`, `append_file`, `lines_iter`, `read_bytes`, `write_bytes`
- File system: `file_exists`, `is_file`, `is_dir`, `mkdir`, `list_dir`, `path_join`, `rename`, `rm`
- HTTP: `http_get`, `http_post`
- Time: `clock`, `sleep`
- JSON: `json_parse`, `json_stringify`
- Type: `type`, `len`, `int`, `float`, `str`, `bool`, `set`
- Async: `set_workers`, `on_ready`, `event_loop`, `set_queue_limit`, `set_task_timeout`
- Iterator: `has_next`, `next`

**Standard Library** ✅ (40+ functions, written in Aether)
- **Core**: `range()`, `enumerate()`
- **Collections**: `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`
- **Math**: `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`
- **String**: `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`
- **Testing**: `assert_eq()`, `assert_true()`, `assert_false()`, `assert_null()`, `assert_not_null()`, `expect_error()`, `test()`, `test_summary()`

### Test Coverage

```
Total: ~693 tests passing ✅ (2 permanently ignored — deep recursion stack overflow in debug builds)

Unit Tests (134):
├── Lexer: 14 tests
├── Parser: 53 tests
├── Interpreter: 17 tests
├── Built-ins: 15 tests
└── Other unit: 35 tests

Integration Tests (~559):
├── Core features: 29 tests
├── Async: 21 tests
├── I/O pool: 14 tests
├── Event loop: 15 tests
├── Structs: 14 tests
├── Iterators: 22 tests
├── Sets: 24 tests
├── Null safety: 23 tests
├── JSON: 25 tests
├── HTTP: 5 tests (0 ignored — all network-free)
├── Error handling: 10 + 11 tests
├── String features: 16 + 9 + 8 + 15 tests
├── Array methods: 22 tests
├── Dict: 27 tests
├── Module system: 13 tests
├── Stdlib (collections/math/string/core): 38+26+24+9 tests
├── Stdlib testing framework: 19 tests
├── Function expressions: 13 tests
├── GC/leak: 6 + 4 tests
└── ... (35 test files total)
```

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

### Near-Term Backlog (Tier 1)

See **[BACKLOG.md](BACKLOG.md)** for the full prioritised list. Top items:

- `match` statement — pattern matching, replaces chained if/else
- Destructuring — `let [a, b] = arr`, `let {x, y} = dict`
- `format(fmt, args...)` — printf-style string formatting
- Variadic functions — `fn sum(...args)`

### Tier 2: Type System

- Enums with associated data
- Generics (lightweight)
- Interface / trait system

### Tier 3: Networking

- TCP/UDP server support
- WebSocket client
- DNS resolution

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
- **[REPL.md](REPL.md)** — Interactive mode implementation
- **[STDLIB.md](STDLIB.md)** — Standard library design
- **[GC_DESIGN.md](GC_DESIGN.md)** — Garbage collection architecture

**Language Features:**
- **[STRUCT.md](STRUCT.md)** — User-defined types with fields and methods
- **[ERROR_HANDLING.md](ERROR_HANDLING.md)** — try/catch/finally/throw
- **[STRING_FEATURES.md](STRING_FEATURES.md)** — String indexing, interpolation, slicing
- **[ASYNC.md](ASYNC.md)** — Async/await and I/O thread pool
- **[EVENT_LOOP.md](EVENT_LOOP.md)** — Callback-based async
- **[JSON.md](JSON.md)** — JSON parsing and serialization
- **[TIME.md](TIME.md)** — Time functions (clock, sleep)
- **[HTTP.md](HTTP.md)** — HTTP client functions
- **[MODULE_SYSTEM.md](MODULE_SYSTEM.md)** — Import and module loading
- **[ITERATOR_PROTOCOL.md](ITERATOR_PROTOCOL.md)** — Iterator protocol
- **[BACKLOG.md](BACKLOG.md)** — Feature backlog (~30 items, 7 tiers)

### External Resources
- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- [Rust Programming Language Book](https://doc.rust-lang.org/book/)

### Quick Links
- **Main README**: [../README.md](../README.md)
- **Project Guide**: [../CLAUDE.md](../CLAUDE.md)
- **Examples**: [../examples/](../examples/)
- **Standard Library**: [../stdlib/](../stdlib/)

---

**Last Updated**: April 29, 2026
**Current Phase**: Phase 5 Complete ✅
**Test Count**: ~693 passing
