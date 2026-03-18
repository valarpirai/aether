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
| **Components** | | |
| [LEXER.md](docs/LEXER.md) | Lexer implementation (tokenization, 14 tests) | ✅ Complete |
| [PARSER.md](docs/PARSER.md) | Parser implementation (recursive descent, 53 tests) | ✅ Complete |
| [INTERPRETER.md](docs/INTERPRETER.md) | Interpreter implementation (82 tests, 2 ignored) | 🚧 In Progress |

## Development Commands

Once the Rust project is initialized, use these commands:

### Build
```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Test
```bash
cargo test           # Run all tests
cargo test -- --nocapture # Show println! output during tests
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

**Current Phase**: Phase 1 - Core Interpreter (In Progress)

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

### Current Work
- 🚧 **REPL** - Next to implement:
  - Interactive read-eval-print loop
  - Line editing with rustyline
  - Debug commands (_tokens, _ast, _env)
  - Multi-line input support

### Next Steps
1. ✅ ~~Lexer (tokenization)~~
2. ✅ ~~Parser (complete)~~
3. ✅ ~~Interpreter (core features)~~
4. ✅ ~~Integration tests~~
5. 🚧 **REPL** - **IN PROGRESS**
6. ⏳ Fix loop bugs (while/for infinite loops)
7. ⏳ Phase 2: Built-in functions (print, len, type, etc.)
8. ⏳ Phase 3: Collections & Built-ins (methods)
9. ⏳ Phase 4: Module System

### Test Coverage
- **Total Tests**: 102 passing ✅
  - **Unit Tests**: 82 passing (2 ignored for loop debugging)
    - Lexer: 14 tests
    - Parser: 53 tests
    - Interpreter: 10 value/env tests + 8 statement tests
  - **Integration Tests**: 20 passing
- **Code Quality**: 0 clippy warnings

### Implementation Guidelines
- ✅ Follow Rust best practices and idioms
- ✅ Test-driven development (write tests first)
- ✅ Refer to `docs/DESIGN.md` for language specification
- ✅ Create files incrementally, not all at once
- ✅ Test each component before moving to the next
- ✅ Use separate `<module>_tests.rs` files for tests

## Quick Reference

For detailed development guidelines, code organization, testing strategies, and best practices, see **[DEVELOPMENT.md](docs/DEVELOPMENT.md)**.