# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## Project Overview

Aether is a general-purpose programming language implemented in Rust — a
tree-walking interpreter with a rich standard library, async I/O, structs, and a
module system.

- **Execution**: interpreted (tree-walking)
- **Typing**: dynamic, with runtime type checking
- **Memory**: Rc-based reference counting
- **Syntax**: C-like, curly braces, no semicolons
- **File extension**: `.ae`, entry point `main()`

Current phase, feature inventory, and test counts:
**[ARCHITECTURE.md — Current Status](docs/dev/ARCHITECTURE.md#current-status)**.

## Rules

Before starting any task, check `.claude/rules/index.md` for the matching rule
file and follow it. Rules live in `.claude/rules/`; each file covers one action.
Load the file for the task at hand.

## Where to Add Features

See **[DEVELOPMENT.md — Where to Add New Features](docs/dev/DEVELOPMENT.md#where-to-add-new-features)**
for the task-to-file map.

### Evaluator sub-module layout

```
src/interpreter/evaluator/
  mod.rs          — Evaluator struct, constructors, public API, call_main
  expressions.rs  — eval_expr, eval_index, eval_slice, await_value
  statements.rs   — exec_stmt_internal (all Stmt variants)
  functions.rs    — eval_call, call_value, exec_async_body, try_submit_io_task
  members.rs      — eval_member, eval_method_call (collections + structs)
  modules.rs      — load_module, import_from, resolve_module_path
  operators.rs    — eval_unary, eval_binary, arithmetic, comparison
```

### Built-in vs stdlib

**Built-in (Rust)** if it needs interpreter internals (`type()`, `len()`,
`await`), is performance-critical (operators, indexing), or does native I/O
(`print`, `read_file`, `http_get`, `sleep`).

**Stdlib (Aether)** if it can be written in Aether on existing primitives, or is
user-modifiable logic (`map`, `filter`, `range`).

**Rule of thumb**: if you can write it in Aether, put it in stdlib. Full decision
tree in [DEVELOPMENT.md](docs/dev/DEVELOPMENT.md#feature-implementation-decision-tree).

### Value construction

Never construct `Value` variants directly — use the helpers
(`Value::string`, `Value::array`, `Value::dict`, `Value::promise_io`, …). Table in
[INTERPRETER.md](docs/dev/INTERPRETER.md#value-construction-helpers).

## Development Commands

```bash
# Build
cargo build               # debug
cargo build --release     # optimised

# Test — always use --test-threads=1
cargo test -- --test-threads=1
cargo test -- --test-threads=1 --nocapture              # show output
cargo test --test error_context_test -- --test-threads=1 # single file
cargo test --test gc_test -- --test-threads=1            # memory / GC

# macOS leak check (spot-check after adding new Value variants)
leaks --atExit -- ./target/debug/aether examples/<feature>_demo.ae

# Run
cargo run -- examples/error_context.ae
AETHER_IO_WORKERS=4 cargo run -- examples/concurrent_io.ae

# Code quality
cargo fmt && cargo clippy
```

Use `Evaluator::new_without_stdlib()` in tests that don't call stdlib functions —
it is ~760× faster than `Evaluator::new()`.

## Post-Feature Checklist

1. **Tests** — `tests/<feature>_test.rs` with happy path, edge cases, error cases
2. **Example** — `examples/<feature>_demo.ae` covering all new functions/syntax
3. **Docs** — update the relevant component doc and BACKLOG.md
4. **Static checker** — add new builtin/stdlib names to `BUILTINS` in
   `src/checker.rs`; add match arms for new AST variants in
   `check_stmt`/`check_expr`; verify with `cargo run -- check examples/<feature>_demo.ae`
5. **Memory check** — `cargo test --test gc_test`; for new `Value` variants also
   run `leaks --atExit`
6. **Code quality** — `cargo fmt && cargo clippy && cargo test -- --test-threads=1`

Full details: **[DEVELOPMENT.md — Post-Feature Checklist](docs/dev/DEVELOPMENT.md#post-feature-checklist)**

## Documentation Index

### Language Reference (`docs/lang/`) — what Aether does and how to use it

| Document | Description |
|----------|-------------|
| [STRINGS.md](docs/lang/STRINGS.md) | Literals, indexing, slicing, interpolation, methods |
| [FORMAT.md](docs/lang/FORMAT.md) | format() — `{}` placeholders, width, alignment, precision |
| [DESTRUCTURING.md](docs/lang/DESTRUCTURING.md) | Array and dict destructuring, rest, rename, defaults |
| [STRUCT.md](docs/lang/STRUCT.md) | User-defined types with fields and methods |
| [ERROR_HANDLING.md](docs/lang/ERROR_HANDLING.md) | try/catch/finally/throw with stack traces |
| [ASYNC.md](docs/lang/ASYNC.md) | async fn, await, .then(), Promise combinators, I/O pool |
| [ITERATORS.md](docs/lang/ITERATORS.md) | Iterator protocol, built-in and custom iterators |
| [MODULE_SYSTEM.md](docs/lang/MODULE_SYSTEM.md) | import, from…import, stdlib modules |
| [BUILTINS.md](docs/lang/BUILTINS.md) | Built-in (Rust) function reference — print, len, file I/O, sockets |
| [STDLIB.md](docs/lang/STDLIB.md) | Stdlib (Aether) function reference — map, filter, math, string |
| [EXAMPLES.md](docs/lang/EXAMPLES.md) | Worked examples by topic, mirrors `examples/` |
| [HTTP.md](docs/lang/HTTP.md) | http_get(), http_post() |
| [JSON.md](docs/lang/JSON.md) | json_parse(), json_stringify() |
| [CSV.md](docs/lang/CSV.md) | csv_parse(), csv_stringify() |
| [TIME.md](docs/lang/TIME.md) | clock(), sleep() |
| [RANDOM.md](docs/lang/RANDOM.md) | random(), rand_int(n) |
| [PLUGINS.md](docs/lang/PLUGINS.md) | load_plugin() — FFI for Rust shared libraries |
| [PLUGIN_GUIDE.md](docs/lang/PLUGIN_GUIDE.md) | Step-by-step: write a plugin, wrap a Rust crate |
| [REPL.md](docs/lang/REPL.md) | Interactive REPL and file execution |
| [CONFIGURATION.md](docs/lang/CONFIGURATION.md) | Env vars and runtime configuration builtins |
| [TCP.md](docs/lang/TCP.md) | tcp_listen(), tcp_connect(), lifecycle events |

### Developer Docs (`docs/dev/`) — how Aether is built

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/dev/ARCHITECTURE.md) | System architecture, current status, roadmap |
| [DESIGN.md](docs/dev/DESIGN.md) | Complete language specification |
| [DEVELOPMENT.md](docs/dev/DEVELOPMENT.md) | Where to add features, checklist, guidelines |
| [TESTING.md](docs/dev/TESTING.md) | TDD workflow, running tests, debugging |
| [BACKLOG.md](docs/dev/BACKLOG.md) | Prioritised feature backlog |
| [LEXER.md](docs/dev/LEXER.md) | Lexer implementation |
| [PARSER.md](docs/dev/PARSER.md) | Parser implementation |
| [INTERPRETER.md](docs/dev/INTERPRETER.md) | Interpreter / evaluator sub-modules |
| [MEMORY_MANAGEMENT.md](docs/dev/MEMORY_MANAGEMENT.md) | Memory model, Rc-based GC, rationale |
| [EVENT_LOOP.md](docs/dev/EVENT_LOOP.md) | Event loop internals: on_ready, event_loop |
| [ASYNC_IO.md](docs/dev/ASYNC_IO.md) | IoPool, EventLoopQueue, TCP dispatch loop |
| [TCP_UDP.md](docs/dev/TCP_UDP.md) | TCP/UDP implementation: mio I/O loop, channels |
| [FFI_PLUGIN_SYSTEM.md](docs/dev/FFI_PLUGIN_SYSTEM.md) | Plugin protocol, type mapping, status |
| [DEBUGGER.md](docs/dev/DEBUGGER.md) | Debugger implementation |
| [GITHUB_PAGES.md](docs/dev/GITHUB_PAGES.md) | Publishing the docs site |

## Documentation Maintenance

- The `gh-pages` branch serves the GitHub Pages website.
- Update `gh-pages` when adding user-facing features.
- Each doc owns one topic. Link between docs instead of duplicating content.
