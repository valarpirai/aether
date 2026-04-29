# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aether is a general-purpose programming language implemented in Rust — a fully-working tree-walking interpreter with a rich standard library, async I/O, structs, and a module system.

### Language Characteristics
- **Execution**: Interpreted (tree-walking interpreter)
- **Typing**: Dynamic with runtime type checking
- **Memory**: Rc-based reference counting (GC)
- **Syntax**: C-like with curly braces, no semicolons
- **File Extension**: `.ae`
- **Entry Point**: Required `main()` function

### Key Features
- Primitive types: `int`, `float`, `string` (UTF-8), `bool`, `null`
- Collections: `array`, `dict`, `set` (unique, unordered)
- First-class functions with closures, optional parameters, function expressions
- Block-scoped variables using `let` keyword
- Range-based and for-each loops
- String interpolation: `"Hello ${name}"`, string indexing: `str[0]`
- Error handling: `try/catch/throw` with `e.message` and `e.stack_trace`
- Module system: `import`, `from ... import`, aliases
- Structs with fields, methods, and `self` binding
- Async/await — `async fn`, `await`, `Promise.all`, I/O thread pool
- Event loop — `on_ready(promise, callback)`, `event_loop()` for callback-based async
- Null safety — `??` null coalescing, `?.` optional chaining
- REPL with history and tab-completion

## Documentation Index

| Document | Description | Status |
|----------|-------------|--------|
| **Language & Project** | | |
| [DESIGN.md](docs/DESIGN.md) | Complete language specification (types, syntax, features) | ✅ |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, roadmap, and feature checklist | ✅ |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Development guidelines and best practices | ✅ |
| [TESTING.md](docs/TESTING.md) | Testing guide: TDD workflow, running tests, debugging | ✅ |
| [CONFIGURATION.md](docs/CONFIGURATION.md) | All env vars, runtime builtins, and compile-time constants | ✅ |
| [BACKLOG.md](docs/BACKLOG.md) | Prioritised feature backlog (6 tiers, ~30 features) | ✅ |
| **Components** | | |
| [LEXER.md](docs/LEXER.md) | Lexer implementation (tokenization, 14 tests) | ✅ |
| [PARSER.md](docs/PARSER.md) | Parser implementation (recursive descent, 53 tests) | ✅ |
| [INTERPRETER.md](docs/INTERPRETER.md) | Interpreter / evaluator split into sub-modules | ✅ |
| [REPL.md](docs/REPL.md) | REPL and file execution | ✅ |
| [STDLIB.md](docs/STDLIB.md) | Standard library written in Aether | ✅ |
| [GC_DESIGN.md](docs/GC_DESIGN.md) | Rc-based garbage collection design | ✅ |
| **Features** | | |
| [STRUCT.md](docs/STRUCT.md) | User-defined types with fields and methods | ✅ |
| [ERROR_HANDLING.md](docs/ERROR_HANDLING.md) | Try/catch/throw with stack traces | ✅ |
| [STRING_FEATURES.md](docs/STRING_FEATURES.md) | Indexing, interpolation, slicing, spread | ✅ |
| [ASYNC.md](docs/ASYNC.md) | Async/await and I/O thread pool | ✅ |
| [ITERATOR_PROTOCOL.md](docs/ITERATOR_PROTOCOL.md) | Iterator protocol for collections | ✅ |
| [MODULE_SYSTEM.md](docs/MODULE_SYSTEM.md) | Module imports and filesystem resolution | ✅ |
| [JSON.md](docs/JSON.md) | JSON parsing and serialization | ✅ |
| [TIME.md](docs/TIME.md) | Time functions: clock(), sleep() | ✅ |
| [HTTP.md](docs/HTTP.md) | HTTP client: http_get(), http_post() | ✅ |
| [EVENT_LOOP.md](docs/EVENT_LOOP.md) | Callback-based async: on_ready, event_loop | ✅ |

## Quick Reference for Claude Code

### Where to Add New Features

| Task | Primary File | Test File |
|------|-------------|-----------|
| Add token type | `src/lexer/token.rs` | `src/lexer/lexer_tests.rs` |
| Add syntax/AST node | `src/parser/ast.rs` | `src/parser/parser_tests.rs` |
| Add built-in function | `src/interpreter/builtins.rs` | `tests/integration_test.rs` |
| Add stdlib function | `stdlib/*.ae` | `tests/stdlib_test.rs` |
| Add GC-managed value type | `src/interpreter/value.rs` (use Rc) | — |
| Add member property/method | `src/interpreter/evaluator/members.rs` | — |
| Add statement execution | `src/interpreter/evaluator/statements.rs` | — |
| Add expression evaluation | `src/interpreter/evaluator/expressions.rs` | — |
| Add I/O async builtin | `src/interpreter/evaluator/functions.rs` (`try_submit_io_task`) | `tests/io_pool_test.rs` |

### Evaluator Sub-module Layout

```
src/interpreter/evaluator/
  mod.rs          — Evaluator struct, constructors, public API, call_main
  expressions.rs  — eval_expr, eval_index, eval_slice, await_value
  statements.rs   — exec_stmt_internal (all Stmt variants)
  functions.rs    — eval_call, call_value, exec_async_body, try_submit_io_task
  members.rs      — eval_member, eval_method_call (all collection/struct methods)
  modules.rs      — load_module, import_from, resolve_module_path
  operators.rs    — eval_unary, eval_binary, arithmetic, comparison
```

### Key Helper Functions
- `Value::string(s)` — create Rc-wrapped string
- `Value::array(vec)` — create Rc-wrapped array
- `Value::dict(map)` — create Rc-wrapped dict
- `Value::set(hashset)` — create Rc-wrapped set
- `Value::promise(func, args)` — create a pending Promise
- `Value::promise_io(rx)` — create a channel-backed I/O Promise
- `Value::error_val(msg, stack, line)` — create an error object for catch blocks
- `Value::is_truthy()` — boolean coercion for conditionals
- `Value::is_hashable()` — check if value can be used as a set/dict key
- `Environment::with_parent()` — create nested scope
- `Evaluator::await_value(val)` — resolve a Promise (handles Pending and IoWaiting)

### Stdlib Module Locations
- **Core**: `stdlib/core.ae` — `range()`, `enumerate()`
- **Collections**: `stdlib/collections.ae` — `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`
- **Math**: `stdlib/math.ae` — `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`
- **String**: `stdlib/string.ae` — `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`
- **Testing**: `stdlib/testing.ae` — `assert_eq()`, `assert_true()`, `assert_false()`, `assert_null()`, `assert_not_null()`, `expect_error()`, `test()`, `test_summary()`

### Built-in vs Stdlib Decision Tree

**Built-in (Rust)** if the function:
- Requires interpreter internals (`type()`, `len()`, `await`)
- Is performance-critical (operators, indexing)
- Performs native I/O (`print`, `read_file`, `http_get`, `sleep`)

**Stdlib (Aether)** if:
- Can be written in Aether
- Built on existing primitives
- User-modifiable logic (`map`, `filter`, `range`)

**Rule of thumb**: If you can write it in Aether, put it in stdlib.

## Development Commands

```bash
# Build
cargo build               # debug
cargo build --release     # optimised

# Test (always use --test-threads=1)
cargo test -- --test-threads=1
cargo test -- --test-threads=1 --nocapture   # show output
cargo test --test error_context_test -- --test-threads=1  # single file

# Memory / GC tests
cargo test --test gc_test -- --test-threads=1

# macOS leak check (spot-check after adding new Value variants)
leaks --atExit -- ./target/debug/aether examples/<feature>_demo.ae

# Run
cargo run -- examples/error_context.ae
AETHER_IO_WORKERS=4 cargo run -- examples/concurrent_io.ae

# Code quality
cargo fmt
cargo clippy
```

## Post-Feature Checklist

After implementing any feature, before committing:

1. **Tests** — `tests/<feature>_test.rs` with happy path, edge cases, and error cases
2. **Example program** — `examples/<feature>_demo.ae` covering all new functions/syntax
3. **Docs** — update the relevant component doc + CLAUDE.md feature table + BACKLOG.md
4. **Memory check** — run `cargo test --test gc_test`; for new `Value` variants also run `leaks --atExit`
5. **Code quality** — `cargo fmt && cargo clippy && cargo test -- --test-threads=1`

Full details: **[DEVELOPMENT.md — Post-Feature Checklist](docs/DEVELOPMENT.md#post-feature-checklist)**

## Project Status

**Phase**: 5 complete — language is fully functional with async I/O and rich stdlib.

### Completed Feature Summary

| Area | Features |
|------|---------|
| **Core language** | int, float, string, bool, null, array, dict, set; all operators; let, if/else, while, for, break, continue, return |
| **Functions** | declarations, expressions, closures, optional params, recursion (depth limit 100) |
| **Strings** | indexing, interpolation `${expr}`, slicing `str[1:3]`, spread `[...arr]`, upper/lower/trim/split |
| **Collections** | array (push/pop/sort/concat/slice/spread), dict (keys/values/contains), set (union/intersection/difference/subset) |
| **Error handling** | try/catch/throw; `e.message`, `e.stack_trace`; stack frames include filename and line number |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias`; filesystem + embedded stdlib |
| **Structs** | fields, methods, `self` binding, mutable fields via RefCell |
| **Iterators** | `has_next()`, `next()`, for-in over array/dict/set/string/iterator |
| **Async/await** | `async fn`, `await expr`, Promise caching; `Promise.all([p1, p2])` |
| **I/O thread pool** | `set_workers(n)`, `AETHER_IO_WORKERS` env var; async `http_get`, `sleep`, `read_file`, `write_file`, `http_post` |
| **Event loop** | `on_ready(promise, callback)`, `event_loop()`; callback-based async; chained callbacks |
| **Null safety** | `??` null coalescing (short-circuit), `?.` optional member/method chaining |
| **JSON** | `json_parse()`, `json_stringify()` via serde_json |
| **HTTP** | `http_get(url)`, `http_post(url, body)` via reqwest (blocking or async) |
| **Time** | `clock()` (Unix epoch float), `sleep(secs)` |
| **Standard library** | range, enumerate, map, filter, reduce, find, every, some, abs, min, max, sum, clamp, sign, join, repeat, reverse, starts_with, ends_with |
| **Testing framework** | assert_eq, assert_true/false/null, expect_error, test, test_summary |
| **REPL** | rustyline with history (`~/.aether_history`), tab-completion, `_help`/`_env`/`_exit` |
| **Configuration** | `AETHER_IO_WORKERS`, `AETHER_CALL_DEPTH`, `HOME` (see [CONFIGURATION.md](docs/CONFIGURATION.md)) |

### Completed Milestones

| Milestone | Tests at completion |
|-----------|-------------------|
| Phase 1: Core Interpreter | 102 |
| Phase 2: Essential Features | 147 |
| Phase 3: Standard Library | 230 |
| Phase 4: Advanced Language Features | 314 |
| Phase 5 Sprint 1: Testing Framework | 333 |
| Phase 5 Sprint 2: Advanced Types (structs, sets, iterators) | 420 |
| Phase 5 Sprint 3: Async/await + I/O pool | 476 |
| Phase 5 Sprint 4: Error context + stack traces | ~547 |
| Phase 5 Sprint 5: Null safety + Event loop | ~693 |

### Test Coverage (2026-04-29)

- **Total**: ~693 tests passing (134 unit + ~559 integration)
- **Ignored/skipped**: 5 http tests (require network), 2 known recursion stack-overflow
- **Code quality**: cargo clippy clean (5 acceptable `mutable_key_type` warnings for HashSet)

**Unit tests (134):**

| Suite | Count |
|-------|-------|
| Lexer | 14 |
| Parser | 53 |
| Interpreter | 17 |
| Built-ins | 15 |
| Other unit | 35 |

**Integration tests (~559):**

| Suite | Count |
|-------|-------|
| `stdlib_collections_test` | 38 |
| `integration_test` | 29 |
| `dict_test` | 27 |
| `null_coalesce_test` | 23 |
| `stdlib_math_test` | 26 |
| `json_test` | 25 |
| `stdlib_string_test` | 24 |
| `set_test` | 24 |
| `iterator_test` | 22 |
| `array_methods_test` | 22 |
| `async_test` | 21 |
| `clippy_fix_regression_test` | 20 |
| `stdlib_testing_test` | 19 |
| `string_indexing_test` | 16 |
| `event_loop_test` | 15 |
| `slice_test` | 15 |
| `struct_test` | 14 |
| `io_pool_test` | 14 |
| `module_test` | 13 |
| `function_expr_test` | 13 |
| `error_context_test` | 11 |
| `time_test` | 10 |
| `error_handling_test` | 10 |
| `string_interp_test` | 9 |
| `stdlib_test` | 9 |
| `spread_test` | 9 |
| `string_methods_test` | 8 |
| `member_access_test` | 8 |
| `gc_test` | 6 |
| `io_test` | 5 |
| `http_test` | 5 (ignored — network) |
| `closure_leak_test` | 4 |
| `small_recursion_test` | 1 |
| `recursion_limit_test` | 1 |

### Backlog

See **[docs/BACKLOG.md](docs/BACKLOG.md)** for the full prioritised backlog (~30 features across 6 tiers).

Top-of-backlog highlights: `match` statement, destructuring, `format()`, variadic args, enums, TCP/UDP server support.

## Development Resources

- **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** — guidelines, TDD workflow, file-size limits (max 1000 lines), code organisation
- **[TESTING.md](docs/TESTING.md)** — comprehensive testing guide with examples
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** — system design and roadmap
- **[BACKLOG.md](docs/BACKLOG.md)** — feature backlog
- **[CONFIGURATION.md](docs/CONFIGURATION.md)** — all knobs and env vars
- Component docs: LEXER.md, PARSER.md, INTERPRETER.md, REPL.md, STDLIB.md, GC_DESIGN.md

## Documentation
- gh-pages branch is used for the GitHub Pages website
- Update docs in gh-pages when adding new user-facing features
