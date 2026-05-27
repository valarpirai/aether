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
- Async/await — `async fn`, `await`, `Promise.all`, `Promise.race`, `Promise.allSettled`, I/O thread pool
- Event loop — `on_ready(promise, callback)`, `event_loop()` for callback-based async
- Null safety — `??` null coalescing, `?.` optional chaining
- REPL with history and tab-completion

## Documentation Index

### Language Reference (`docs/lang/`) — what Aether does and how to use it

| Document | Description |
|----------|-------------|
| [STRINGS.md](docs/lang/STRINGS.md) | Literals, indexing, slicing, interpolation, methods |
| [DESTRUCTURING.md](docs/lang/DESTRUCTURING.md) | Array and dict destructuring, rest, rename, defaults |
| [STRUCT.md](docs/lang/STRUCT.md) | User-defined types with fields and methods |
| [ERROR_HANDLING.md](docs/lang/ERROR_HANDLING.md) | try/catch/throw with stack traces |
| [ASYNC.md](docs/lang/ASYNC.md) | async fn, await, .then(), Promise.all/race/allSettled, I/O pool |
| [ITERATORS.md](docs/lang/ITERATORS.md) | Iterator protocol, built-in and custom iterators |
| [MODULE_SYSTEM.md](docs/lang/MODULE_SYSTEM.md) | import, from…import, stdlib modules |
| [STDLIB.md](docs/lang/STDLIB.md) | range, map, filter, reduce, math, string, testing |
| [HTTP.md](docs/lang/HTTP.md) | http_get(), http_post() |
| [JSON.md](docs/lang/JSON.md) | json_parse(), json_stringify() |
| [TIME.md](docs/lang/TIME.md) | clock(), sleep() |
| [REPL.md](docs/lang/REPL.md) | Interactive REPL and file execution |
| [CONFIGURATION.md](docs/lang/CONFIGURATION.md) | Env vars and runtime configuration builtins |
| [TCP.md](docs/lang/TCP.md) | tcp_listen(), tcp_connect(), server/client lifecycle events |

### Developer Docs (`docs/dev/`) — how Aether is built

| Document | Description |
|----------|-------------|
| [DESIGN.md](docs/dev/DESIGN.md) | Complete language specification |
| [ARCHITECTURE.md](docs/dev/ARCHITECTURE.md) | System architecture and roadmap |
| [DEVELOPMENT.md](docs/dev/DEVELOPMENT.md) | Development guidelines and best practices |
| [TESTING.md](docs/dev/TESTING.md) | TDD workflow, running tests, debugging |
| [BACKLOG.md](docs/dev/BACKLOG.md) | Prioritised feature backlog |
| [LEXER.md](docs/dev/LEXER.md) | Lexer implementation |
| [PARSER.md](docs/dev/PARSER.md) | Parser implementation |
| [INTERPRETER.md](docs/dev/INTERPRETER.md) | Interpreter / evaluator sub-modules |
| [MEMORY_MANAGEMENT.md](docs/dev/MEMORY_MANAGEMENT.md) | Memory model, Rc-based GC, and design rationale |
| [EVENT_LOOP.md](docs/dev/EVENT_LOOP.md) | Event loop internals: on_ready, event_loop, queue controls |

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
| Extend static checker | `src/checker.rs` | `tests/checker_test.rs` |
| Extend formatter | `src/formatter.rs` | `tests/fmt_test.rs` |

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
- `Value::array(vec)` — create Rc<RefCell>-wrapped array (reference semantics)
- `Value::dict(vec)` — create Rc<RefCell>-wrapped dict (reference semantics)
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
4. **Static checker** — add any new builtin/stdlib names to `BUILTINS` in `src/checker.rs`; add match arms for any new AST variants in `check_stmt`/`check_expr`; verify with `cargo run -- check examples/<feature>_demo.ae`
5. **Memory check** — run `cargo test --test gc_test`; for new `Value` variants also run `leaks --atExit`
6. **Code quality** — `cargo fmt && cargo clippy && cargo test -- --test-threads=1`

Full details: **[DEVELOPMENT.md — Post-Feature Checklist](docs/dev/DEVELOPMENT.md#post-feature-checklist)**

## Project Status

**Phase**: 5 complete — language is fully functional with async I/O and rich stdlib.

### Completed Feature Summary

| Area | Features |
|------|---------|
| **Core language** | int, float, string, bool, null, array, dict, set; all operators; let, if/else, while, for, break, continue, return |
| **Operators** | arithmetic, comparison, logical, bitwise `& \| ^ ~ << >>`, power `**`, ternary `?:`, null coalesce `??`, optional chain `?.` |
| **Pattern matching** | `match` statement — literals, wildcard `_`, binding, or-patterns `\|`, enum variant patterns |
| **Destructuring** | `let [a, b, ...rest] = arr`, `let {host, port: p = 5432} = dict` — array/dict, rest, rename, defaults |
| **Functions** | declarations, expressions, closures, optional params, recursion (depth limit 100) |
| **Strings** | indexing, interpolation `${expr}`, slicing `str[1:3]`, spread `[...arr]`, upper/lower/trim/split |
| **Collections** | array (push/pop/sort/concat/slice/spread), dict (keys/values/contains), set (union/intersection/difference/subset); reference semantics for array/dict/struct; `==` is identity; `.equals()` depth-1 structural; `copy()` depth-1 shallow clone; `id()` for object identity |
| **Error handling** | try/catch/throw; `e.message`, `e.stack_trace`; stack frames include filename and line number |
| **Modules** | `import mod`, `from mod import fn`, `import mod as alias`; filesystem + embedded stdlib |
| **Structs** | fields, methods, `self` binding, mutable fields via RefCell; `.equals()` for depth-1 structural comparison |
| **Iterators** | `has_next()`, `next()`, for-in over array/dict/set/string/iterator |
| **Async/await** | `async fn`, `await expr`, Promise caching; `Promise.all`, `Promise.race`, `Promise.allSettled` |
| **I/O thread pool** | `set_workers(n)`, `AETHER_IO_WORKERS` env var; async `http_get`, `sleep`, `read_file`, `write_file`, `http_post` |
| **Event loop** | `on_ready(promise, callback)`, `event_loop()`; callback-based async; chained callbacks |
| **Null safety** | `??` null coalescing (short-circuit), `?.` optional member/method chaining |
| **JSON** | `json_parse()`, `json_stringify()` via serde_json |
| **HTTP** | `http_get(url)`, `http_post(url, body)` via reqwest (blocking or async) |
| **Time** | `clock()` (Unix epoch float), `sleep(secs)` |
| **TCP** | `tcp_listen(addr[, opts])`, `tcp_connect(addr)`; server events: `on_listen/connect/message/disconnect/error/timeout`, `accept()`, `close()`; client events: `on_connect/message/disconnect/error/timeout`, `start()`, `close()`, `write(data)`; event-driven via mio (single I/O thread, ~8–260 KB per connection); use array/dict for mutable closure state |
| **Standard library** | range, enumerate, map, filter, reduce, find, every, some, abs, min, max, sum, clamp, sign, join, repeat, reverse, starts_with, ends_with, first, last, chunk, partition, zip_longest, uniq_by, contains, index_of, replace, count, pad_left, pad_right, strip_prefix, strip_suffix, is_alpha, is_digit, is_space, pi, e, tau, factorial, trunc, degrees, radians, hypot, exp, sin, cos, tan |
| **Number/string conversions** | `hex(n)`, `oct(n)`, `bin(n)`, `int(s, base)`, `base64_encode(s)`, `base64_decode(s)` |
| **Testing framework** | assert_eq, assert_true/false/null, expect_error, test, test_summary |
| **REPL** | rustyline with history (`~/.aether_history`), tab-completion, `_help`/`_env`/`_exit`, multi-line input (`>>` / `..`) |
| **Configuration** | `AETHER_IO_WORKERS`, `AETHER_CALL_DEPTH`, `HOME` (see [CONFIGURATION.md](docs/lang/CONFIGURATION.md)) |
| **Tooling** | `aether ast` (AST printer — indented tree or JSON), `aether fmt` (formatter), `aether test` (test runner), `aether check` (undefined variable linter) |

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
| Phase 5 Sprint 6: Tooling (fmt, test, check, REPL multi-line) | ~1112 |

### Test Coverage (2026-05-23)

- **Total**: ~1112 tests passing (134 unit + ~978 integration)
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

**Integration tests (~725):**

| Suite | Count |
|-------|-------|
| `stdlib_collections_test` | 39 |
| `parser_tests` | 54 |
| `integration_test` | 30 |
| `null_coalesce_test` | 31 |
| `operators_test` | 31 |
| `dict_test` | 27 |
| `stdlib_math_test` | 26 |
| `json_test` | 25 |
| `stdlib_string_test` | 24 |
| `set_test` | 24 |
| `iterator_test` | 22 |
| `array_methods_test` | 22 |
| `async_test` | 21 |
| `destructure_test` | 20 |
| `clippy_fix_regression_test` | 20 |
| `stdlib_testing_test` | 19 |
| `string_indexing_test` | 16 |
| `event_loop_test` | 15 |
| `slice_test` | 15 |
| `enum_test` | 14 |
| `struct_test` | 14 |
| `io_pool_test` | 14 |
| `match_test` | 13 |
| `module_test` | 13 |
| `function_expr_test` | 13 |
| `error_context_test` | 11 |
| `time_test` | 10 |
| `error_handling_test` | 10 |
| `labeled_loop_test` | 10 |
| `string_interp_test` | 9 |
| `stdlib_test` | 9 |
| `spread_test` | 9 |
| `multiline_string_test` | 9 |
| `string_methods_test` | 8 |
| `member_access_test` | 8 |
| `args_test` | 8 |
| `gc_test` | 6 |
| `io_test` | 5 |
| `http_test` | 5 (ignored — network) |
| `closure_leak_test` | 4 |
| `debugger_test` | 5 |
| `file_io_test` | 2 |
| `small_recursion_test` | 2 |
| `recursion_limit_test` | 2 |
| `fmt_test` | 26 |
| `checker_test` | 27 |
| `test_runner_test` | 10 |

### Backlog

See **[docs/dev/BACKLOG.md](docs/dev/BACKLOG.md)** for the full prioritised backlog (~30 features across 6 tiers).

Top-of-backlog highlights: `format()`, variadic args, enums/tuples, named/default params, UDP server support.

## Development Resources

- **[DEVELOPMENT.md](docs/dev/DEVELOPMENT.md)** — guidelines, TDD workflow, file-size limits (max 1000 lines), code organisation
- **[TESTING.md](docs/dev/TESTING.md)** — comprehensive testing guide with examples
- **[ARCHITECTURE.md](docs/dev/ARCHITECTURE.md)** — system design and roadmap
- **[BACKLOG.md](docs/dev/BACKLOG.md)** — feature backlog
- **[CONFIGURATION.md](docs/lang/CONFIGURATION.md)** — all knobs and env vars
- Component docs: LEXER.md, PARSER.md, INTERPRETER.md, REPL.md, STDLIB.md, MEMORY_MANAGEMENT.md

## Documentation
- gh-pages branch is used for the GitHub Pages website
- Update docs in gh-pages when adding new user-facing features

## Rules

Before starting any task, check `.claude/rules/index.md` for the matching rule file and follow it.

Rules live in `.claude/rules/`. Each file covers one action. Load the file for the task at hand.
