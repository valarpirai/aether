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

The complete language design specification is available in `docs/DESIGN.md`. This document covers:
- Type system and operators
- Control flow structures
- Function syntax and scoping rules
- Built-in functions and methods
- Module system design
- Implementation phases
- Example programs

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

**Current Phase**: Design Complete, Implementation Not Started

### Completed
- ✅ Language design specification (see `docs/DESIGN.md`)
- ✅ Type system design
- ✅ Syntax and grammar decisions
- ✅ Control flow structures
- ✅ Built-in functions specification

### Next Steps
When implementing, follow this order:
1. Initialize with `cargo init` if not already done
2. Implement Phase 1: Core Interpreter
   - Lexer (tokenization)
   - Parser (AST generation)
   - Basic tree-walking interpreter
   - REPL
3. Implement Phase 2: Basic Features (primitives, variables, functions, control flow)
4. Implement Phase 3: Collections & Built-ins
5. Implement Phase 4: Module System

### Implementation Guidelines
- Follow Rust best practices and idioms
- Refer to `docs/DESIGN.md` for language specification
- Create files incrementally, not all at once
- Test each component before moving to the next