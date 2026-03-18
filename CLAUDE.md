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

## Documentation

- **[DESIGN.md](docs/DESIGN.md)** - Complete language design specification covering:
  - Type system and operators
  - Control flow structures
  - Function syntax and scoping rules
  - Built-in functions and methods
  - Module system design
  - Implementation phases
  - Example programs

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Architecture overview and roadmap covering:
  - Interpreter architecture (lexer → parser → interpreter)
  - Module structure and responsibilities
  - Data flow and compilation pipeline
  - Implemented features (detailed checklist)
  - Pending features and phase breakdown
  - Test coverage and quality metrics

- **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Development guidelines covering:
  - Code organization and module structure
  - Testing strategy and TDD workflow
  - Error handling patterns
  - Code style and Rust idioms
  - Incremental development process
  - Quality assurance and CI/CD

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
- ✅ **Parser (Complete - Expressions & Statements)** - Recursive descent parser with 23 tests
  - Primary expressions (literals, identifiers, grouped)
  - Unary expressions (`-`, `!`)
  - Binary arithmetic operators (`+`, `-`, `*`, `/`, `%`)
  - Comparison operators (`<`, `>`, `<=`, `>=`, `==`, `!=`)
  - Logical operators (`&&`, `||`)
  - Proper operator precedence
  - AST node definitions
  - Variable declarations (`let x = value`)
  - Block statements (`{ ... }`)
  - If/else conditionals
  - While loops
  - For loops (for-in style)
  - Return/break/continue statements

### Current Work
- 🚧 **Parser (Advanced Features)** - Next to implement:
  - Assignment statements
  - Function declarations
  - Function calls
  - Arrays and indexing
  - Member access (dot notation)

### Next Steps
1. ✅ ~~Lexer (tokenization)~~
2. ✅ ~~Parser (expressions)~~
3. ✅ ~~Parser (statements)~~
4. 🚧 Parser (arrays, function calls, assignments) - **IN PROGRESS**
5. ⏳ Tree-walking interpreter
6. ⏳ REPL
7. ⏳ Phase 2: Basic Features (variables, functions, control flow)
8. ⏳ Phase 3: Collections & Built-ins
9. ⏳ Phase 4: Module System

### Test Coverage
- **Total Tests**: 37 passing ✅
- **Lexer Tests**: 14 (in `src/lexer/lexer_tests.rs`)
- **Parser Tests**: 23 (in `src/parser/parser_tests.rs`)
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