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
- Collections: `array`, `dict`, `set`
- First-class functions with closures
- Block-scoped variables using `let` keyword
- Range-based and for-each loops
- String interpolation: `"Hello ${name}"`
- REPL support

## Documentation Index

| Document | Description | Status |
|----------|-------------|--------|
| **Language & Project** | | |
| [DESIGN.md](docs/DESIGN.md) | Complete language specification (types, syntax, features) | ✅ Complete |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System architecture, roadmap, and feature checklist | ✅ Complete |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Development guidelines and best practices | ✅ Complete |
| [TESTING.md](docs/TESTING.md) | Testing guide: TDD workflow, running tests, debugging | ✅ Complete |
| **Components** | | |
| [LEXER.md](docs/LEXER.md) | Lexer implementation (tokenization, 14 tests) | ✅ Complete |
| [PARSER.md](docs/PARSER.md) | Parser implementation (recursive descent, 53 tests) | ✅ Complete |
| [INTERPRETER.md](docs/INTERPRETER.md) | Interpreter implementation (82 tests, 2 ignored) | ✅ Complete |
| [REPL.md](docs/REPL.md) | REPL and file execution | ✅ Complete |

## Quick Reference for Claude Code

### Where to Add New Features

| Task | Primary File | Test File |
|------|-------------|-----------|
| Add token type | `src/lexer/token.rs` | `src/lexer/lexer_tests.rs` |
| Add syntax/AST node | `src/parser/ast.rs` | `src/parser/parser_tests.rs` |
| Add built-in function | `src/interpreter/builtins.rs` | `tests/integration_tests.rs` |
| Add stdlib function | `stdlib/*.ae` | `tests/stdlib_tests.rs` |
| Add GC-managed type | `src/interpreter/value.rs` (use Rc) | - |
| Add member method | `src/interpreter/evaluator.rs` (eval_member_access) | - |

### Key Helper Functions
- `Value::string(s)` - Create Rc-wrapped string
- `Value::array(vec)` - Create Rc-wrapped array
- `Value::is_truthy()` - Boolean coercion for conditionals
- `Environment::with_parent()` - Create nested scope

### Stdlib Module Locations
- **Core**: `stdlib/core.ae` (range, enumerate)
- **Collections**: `stdlib/collections.ae` (map, filter, reduce, find, every, some)
- **Math**: `stdlib/math.ae` (abs, min, max, sum, clamp, sign)
- **String**: `stdlib/string.ae` (join, repeat, reverse, starts_with, ends_with)

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

**Current Phase**: Phase 4 - Advanced Language Features

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

### What Works Now
- ✅ Full lexer, parser, and interpreter
- ✅ All expressions and statements
- ✅ Functions with closures and optional parameters
- ✅ Arrays with methods (push, pop, length)
- ✅ Strings with methods (upper, lower, trim, split)
- ✅ Member access (obj.property)
- ✅ Interactive REPL with history
- ✅ File execution
- ✅ Built-in functions (print, println, type, len, conversions)
- ✅ **Complete Standard Library** - Written in Aether!
  - Core: range(), enumerate()
  - Collections: map(), filter(), reduce(), find(), every(), some()
  - Math: abs(), min(), max(), sum(), clamp(), sign()
  - String: join(), repeat(), reverse(), starts_with(), ends_with()
- ✅ **230 tests passing** (94 unit + 136 integration)

### Completed Milestones
1. ✅ Phase 1: Core Interpreter (102 tests)
2. ✅ Phase 2: Essential Features (+45 tests → 147 total)
3. ✅ Phase 3: Standard Library (+83 tests → 230 total)

**Development Time**: ~10 hours total across 3 phases

### Future Work (Phase 4+)
1. ⏳ Function expressions (inline anonymous functions)
2. ⏳ Module system (import/from statements)
3. ⏳ User-defined modules (load .ae files from filesystem)
4. ⏳ Error handling (try/catch or Result types)
5. ⏳ String indexing (direct character access)
6. ⏳ Stdlib expansion (io, json, http, time, testing framework)

### Test Coverage (Last Updated: 2026-03-22)

- **Total**: 230 tests passing ✅ (100% success rate)
- **Code Quality**: 0 clippy warnings

**Breakdown by Category:**

**Unit Tests (94):**
- Lexer: 14 tests
- Parser: 53 tests
- Interpreter: 17 tests
- Built-ins: 10 tests

**Integration Tests (136):**
- Core features: 29 tests
- Member access: 8 tests
- Array methods: 8 tests
- String methods: 8 tests
- Stdlib core: 9 tests
- Stdlib collections: 24 tests
- Stdlib math: 26 tests
- Stdlib string: 24 tests

## Development Resources

For contributing or extending Aether, see:
- **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Guidelines, TDD workflow, code organization
- **[TESTING.md](docs/TESTING.md)** - Comprehensive testing guide with examples
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and roadmap
- Component docs: LEXER.md, PARSER.md, INTERPRETER.md, REPL.md, STDLIB.md, GC_DESIGN.md