# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aether is a general-purpose programming language implementation written in Rust. The project is currently in initial setup phase.

### Language Characteristics
- **Execution**: Interpreted (tree-walking interpreter)
- **Typing**: Dynamic with runtime type checking
- **Memory**: Automatic memory management (garbage collection)
- **Syntax**: C-like with curly braces, no semicolons
- **File Extension**: `.ae`
- **Entry Point**: Required `main()` function

### Key Features
- Primitive types: `int`, `float`, `string` (UTF-8), `bool`, `null`
- Collections: `array`, `dict`, `set` (unique, unordered)
- First-class functions with closures and function expressions
- Block-scoped variables using `let` keyword
- Range-based and for-each loops
- String interpolation: `"Hello ${name}"`
- String indexing: `str[0]`
- Error handling: `try/catch/throw`
- Module system: `import`, `from ... import`
- REPL support

## Documentation Index

| Document | Description | Status |
|----------|-------------|--------|
| **Language & Project** | | |
| [DESIGN.md](docs/DESIGN.md) | Complete language specification (types, syntax, features) | ✅ Complete |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, roadmap, and feature checklist | ✅ Complete |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Development guidelines and best practices | ✅ Complete |
| [TESTING.md](docs/TESTING.md) | Testing guide: TDD workflow, running tests, debugging | ✅ Complete |
| [CONFIGURATION.md](docs/CONFIGURATION.md) | All env vars, runtime builtins, and compile-time constants | ✅ Complete |
| **Components** | | |
| [LEXER.md](docs/LEXER.md) | Lexer implementation (tokenization, 14 tests) | ✅ Complete |
| [PARSER.md](docs/PARSER.md) | Parser implementation (recursive descent, 53 tests) | ✅ Complete |
| [INTERPRETER.md](docs/INTERPRETER.md) | Interpreter implementation (82 tests, 2 ignored) | ✅ Complete |
| [REPL.md](docs/REPL.md) | REPL and file execution | ✅ Complete |
| **Features** | | |
| [STRUCT.md](docs/STRUCT.md) | User-defined types with fields and methods | ✅ Complete |
| [ERROR_HANDLING.md](docs/ERROR_HANDLING.md) | Try/catch/throw exception handling | ✅ Complete |
| [STRING_FEATURES.md](docs/STRING_FEATURES.md) | Indexing, interpolation, slicing, spread | ✅ Complete |
| [JSON.md](docs/JSON.md) | JSON parsing and serialization (25 tests) | ✅ Complete |
| [TIME.md](docs/TIME.md) | Time functions: clock(), sleep() (10 tests) | ✅ Complete |
| [HTTP.md](docs/HTTP.md) | HTTP client: http_get(), http_post() | ✅ Complete |

## Quick Reference for Claude Code

### Where to Add New Features

| Task | Primary File | Test File |
|------|-------------|-----------|
| Add token type | `src/lexer/token.rs` | `src/lexer/lexer_tests.rs` |
| Add syntax/AST node | `src/parser/ast.rs` | `src/parser/parser_tests.rs` |
| Add built-in function | `src/interpreter/builtins.rs` | `tests/integration_test.rs` |
| Add stdlib function | `stdlib/*.ae` | `tests/stdlib_test.rs` |
| Add GC-managed type | `src/interpreter/value.rs` (use Rc) | - |
| Add member method | `src/interpreter/evaluator.rs` (eval_member_access) | - |

### Key Helper Functions
- `Value::string(s)` - Create Rc-wrapped string
- `Value::array(vec)` - Create Rc-wrapped array
- `Value::dict(map)` - Create Rc-wrapped dict
- `Value::set(hashset)` - Create Rc-wrapped set
- `Value::is_truthy()` - Boolean coercion for conditionals
- `Value::is_hashable()` - Check if value can be in a set
- `Environment::with_parent()` - Create nested scope

### Stdlib Module Locations
- **Core**: `stdlib/core.ae` (range, enumerate)
- **Collections**: `stdlib/collections.ae` (map, filter, reduce, find, every, some)
- **Math**: `stdlib/math.ae` (abs, min, max, sum, clamp, sign)
- **String**: `stdlib/string.ae` (join, repeat, reverse, starts_with, ends_with)
- **Testing**: `stdlib/testing.ae` (assert_eq, assert_true, assert_false, assert_null, assert_not_null, expect_error, test, test_summary)

### Built-in vs Stdlib Decision Tree

**Built-in (Rust)** if:
- Requires interpreter internals (type(), len())
- Performance critical (operators, indexing)
- Core I/O operations (print, println)

**Stdlib (Aether)** if:
- Can be written in Aether
- Built on existing primitives
- User-modifiable logic (map, filter, range)

**Rule of thumb**: If you can write it in Aether, put it in stdlib!

## Development Commands

Once the Rust project is initialized, refer [DEVELOPMENT.md](docs/DEVELOPMENT.md) and use these commands:

### Build
```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Test
```bash
cargo test           # Run all tests
cargo test -- --test-threads=1  # IMPORTANT: Reduces memory pressure (recommended)
cargo test -- --nocapture # Show println! output during tests
cargo test -- --nocapture --test-threads=1 # With output, sequential execution
```

### Run
```bash
cargo run            # Run the main binary
cargo run -- [args]  # Run with arguments
```

### Code Quality
```bash
cargo fmt            # Format code
cargo clippy         # Run linter
```

## Project Status

**Current Phase**: Phase 5 - Stdlib Expansion & Polish

### Completed
- ✅ Language design specification (see `docs/DESIGN.md`)
- ✅ Type system design
- ✅ Syntax and grammar decisions
- ✅ Development environment setup
- ✅ **Lexer (Complete)** - Full tokenization with 14 tests
  - All token types (integers, floats, strings, keywords, operators)
  - String escape sequences
  - Single-line and multi-line comments
  - Error handling with proper error types
- ✅ **Parser (Complete)** - Recursive descent parser with 53 tests
  - All expressions (literals, identifiers, operators)
  - Proper operator precedence
  - All statements (let, blocks, if/else, loops)
  - Function declarations
  - Function calls
  - Arrays and indexing
  - Assignment statements
  - Member access syntax

- ✅ **Interpreter (Complete)** - Tree-walking interpreter with 82 tests
  - Value types (int, float, string, bool, null, array, function)
  - Environment with lexical scoping
  - Expression evaluation (all operators)
  - Statement execution (let, assign, if/else, return)
  - Function declarations and calls
  - Closures
  - Type checking and error handling
  - Note: 2 loop tests ignored (infinite loop bugs to fix later)

- ✅ **Integration Tests** - 20 end-to-end tests
  - Complete programs from source to execution
  - Error handling verification
  - Functions, closures, arrays
  - All features working together

- ✅ **REPL** - Interactive interpreter
  - Line editing with history (rustyline)
  - Special commands (_help, _env, _exit)
  - File execution mode (aether file.ae)
  - REPL mode (just 'aether')

### Phase 1 Status: ✅ COMPLETE!

All planned features for Phase 1 have been implemented and tested.

### Phase 2 Status: ✅ COMPLETE! (Essential Features)

**Sprint 1**: Loop fixes + Core I/O
- ✅ Fixed while/for loops (tests un-ignored)
- ✅ Implemented print() and println()

**Sprint 2**: Type System Built-ins
- ✅ Implemented type(), len()
- ✅ Type conversions: int(), float(), str(), bool()

**Sprint 3**: Member Access (TDD)
- ✅ array.length, string.length properties
- ✅ Proper error handling for undefined properties

**Sprint 4**: Collection Methods (TDD)
- ✅ Array methods: push(), pop()
- ✅ String methods: upper(), lower(), trim(), split()

### Phase 3 Status: ✅ COMPLETE! (Standard Library)

**Sprint 1**: ✅ Stdlib Foundation (+9 tests)
- ✅ Embedded module system (compiled into binary)
- ✅ stdlib/core.ae with range() and enumerate()
- ✅ Optional function parameters support
- ✅ Stdlib auto-loads at startup

**Sprint 2**: ✅ Collections Module (+24 tests)
- ✅ map(), filter(), reduce()
- ✅ find(), every(), some()
- ✅ All written in Aether!

**Sprint 3**: ✅ Math & String Utilities (+50 tests)
- ✅ Math: abs(), min(), max(), sum(), clamp(), sign()
- ✅ String: join(), repeat(), reverse(), starts_with(), ends_with()
- ✅ Function overloading (min/max with 2 args or array)

### Phase 4 Status: ✅ COMPLETE! (Advanced Language Features)

**Sprint 1**: ✅ Function Expressions & Recursion (+16 tests)
- ✅ Function expressions: `fn(params) { body }` - assignable and passable
- ✅ Recursive functions with stack overflow protection (limit: 100 calls)
- ✅ Closures fully working (memory leak fixed)

**Sprint 2**: ✅ String Indexing & Interpolation (+25 tests)
- ✅ String indexing: `str[0]`, `str[i]` - direct character access
- ✅ String interpolation: `"Hello ${name}"`, `"${a + b}"`

**Sprint 3**: ✅ Module System (+13 tests)
- ✅ `import module` - namespace import
- ✅ `from module import fn1, fn2` - selective import
- ✅ `import module as alias` - aliased import
- ✅ User-defined `.ae` modules from filesystem

**Sprint 4**: ✅ Error Handling (+10 tests)
- ✅ `try { ... } catch(e) { ... }` - structured exception handling
- ✅ `throw value` - throw any value as an error
- ✅ Error propagation across function calls

**Sprint 5**: ✅ Dict Literals & IO Builtins (+15 tests)
- ✅ Dict literals: `{"key": value, ...}`
- ✅ Dict methods: `keys()`, `values()`, `contains()`
- ✅ IO builtins: `input()`, `read_file()`, `write_file()`

### What Works Now
- ✅ Full lexer, parser, and interpreter
- ✅ All expressions and statements
- ✅ Functions with closures, optional parameters, and function expressions
- ✅ Arrays with methods (push, pop, length, contains, sort, concat)
- ✅ Dicts with literals and methods (keys, values, contains, size/length)
- ✅ Sets with methods (add, remove, contains, clear, to_array, union, intersection, difference, is_subset)
- ✅ Strings with methods (upper, lower, trim, split) and indexing
- ✅ String interpolation: `"Hello ${name}"`
- ✅ Member access (obj.property)
- ✅ Error handling (try/catch/throw)
- ✅ Module system (import, from...import, aliases)
- ✅ Interactive REPL with history
- ✅ File execution
- ✅ Built-in functions (print, println, input, read_file, write_file, type, len, conversions)
- ✅ **Complete Standard Library** - Written in Aether!
  - Core: range(), enumerate()
  - Collections: map(), filter(), reduce(), find(), every(), some()
  - Math: abs(), min(), max(), sum(), clamp(), sign()
  - String: join(), repeat(), reverse(), starts_with(), ends_with()
  - Testing: assert_eq(), assert_true(), assert_false(), assert_null(), assert_not_null(), expect_error(), test(), test_summary()
- ✅ Structs with fields, methods, and `self` binding
- ✅ **485 tests passing** (99 unit + 386 integration, 1 ignored)

### Completed Milestones
1. ✅ Phase 1: Core Interpreter (102 tests)
2. ✅ Phase 2: Essential Features (+45 tests → 147 total)
3. ✅ Phase 3: Standard Library (+83 tests → 230 total)
4. ✅ Phase 4: Advanced Language Features (+84 tests → 314 total)
5. ✅ Phase 5 Sprint 1: Testing Framework (+19 tests → 333 total)
6. ✅ Phase 5 Sprint 2: Advanced Types (+87 tests → 420 total)

**Development Time**: ~15 hours total across 5 phases

### Future Work (Phase 5 continued)
1. ✅ `json_parse()`, `json_stringify()` — via serde_json
2. ✅ `clock()`, `sleep()` — Unix epoch float, thread sleep
3. ✅ `http` module — http_get(), http_post() via reqwest (blocking)
4. ✅ User-defined structs — fields, methods, `self` binding, mutation via RefCell
5. ✅ Iterator protocol — array/dict/set/string iterators, has_next/next, for-in
6. ✅ Async/await — Promise-based, async fn, await, AsyncFunctionExpr (Phase 1)
7. ✅ I/O thread pool — configurable workers, async http/sleep/file builtins, Promise.all (Phase 2)
8. ✅ Runtime error context — line numbers, call-stack traces, `e.message` / `e.stack_trace` in catch blocks
9. ✅ `AETHER_CALL_DEPTH` env var — configurable recursion depth limit at startup

### Backlog
- ✅ Array slice syntax: `arr[1:3]`
- ✅ Array spread operator: `[...arr1, ...arr2]`
- ✅ Array `sort()` method
- ✅ Array `concat()` method
- Bytecode compiler (deferred — Phase 6)
- String formatting / `format()` builtin
- Multi-line strings / heredocs
- Named/keyword arguments

### Test Coverage (Last Updated: 2026-04-28)

- **Total**: ~536 tests passing ✅ (1 ignored, 1 known stack-overflow bug in recursion limit test; 5 http tests ignored — require network)
- **Code Quality**: 5 clippy warnings (mutable key type in HashSet - acceptable)

**Breakdown by Category:**

**Unit Tests (99):**
- Lexer: 14 tests
- Parser: 53 tests
- Interpreter: 17 tests
- Built-ins: 15 tests

**Integration Tests (386):**
- Core features: 29 tests
- Member access: 8 tests
- Array methods: 22 tests
- String methods: 8 tests
- String indexing: 16 tests
- String interpolation: 9 tests
- Function expressions: 13 tests
- Closures: 4 tests
- **Dict literals & methods: 27 tests** (10 existing + 17 new) ✨
- Error handling: 10 tests
- Module system: 13 tests
- IO builtins: 5 tests
- Stdlib core: 9 tests
- Stdlib testing: 19 tests
- Stdlib collections: 38 tests
- Stdlib math: 26 tests
- Stdlib string: 24 tests
- Slice syntax: 15 tests
- Spread operator: 9 tests
- JSON builtins: 25 tests
- Time builtins: 10 tests
- Structs: 14 tests
- **Set type: 24 tests** ✨
- GC tests: 7 tests
- **Async/await: 21 tests** ✨
- **I/O pool (Phase 2): 14 tests** ✨

## Development Resources

For contributing or extending Aether, see:
- **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Guidelines, TDD workflow, code organization
- **[TESTING.md](docs/TESTING.md)** - Comprehensive testing guide with examples
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and roadmap
- Component docs: LEXER.md, PARSER.md, INTERPRETER.md, REPL.md, STDLIB.md, GC_DESIGN.md

## Documentation
- Update the docs in the gh-pages branch
- gh-pages is used for deploying github pages website
