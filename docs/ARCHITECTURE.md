---
layout: default
title: Aether Architecture & Roadmap
---

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
> - **[DEVELOPMENT.md](DEVELOPMENT.html)** - TDD workflow, testing strategy, code style, common pitfalls

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
```

### Core Components

| Component | Status | Purpose | Details |
|-----------|--------|---------|---------|
| **Lexer** | ✅ Complete | Tokenization | See [LEXER.md](LEXER.html) |
| **Parser** | ✅ Complete | Syntax analysis | See [PARSER.md](PARSER.html) |
| **Interpreter** | ✅ Complete | AST execution | See [INTERPRETER.md](INTERPRETER.html) |
| **REPL** | ✅ Complete | Interactive mode | See [REPL.md](REPL.html) |
| **Standard Library** | ✅ Complete | Core functions | See [STDLIB.md](STDLIB.html) |
| **Garbage Collection** | ✅ Complete | Memory management | See [GC_DESIGN.md](GC_DESIGN.html) |

### Project Structure

```
aether/
├── docs/              # Comprehensive documentation
├── stdlib/            # Standard library (written in Aether!)
├── examples/          # Example programs
├── tests/             # Integration tests (234 tests)
└── src/
    ├── lexer/         # Tokenization (14 tests)
    ├── parser/        # Parsing (53 tests)
    ├── interpreter/   # Execution (32 tests)
    └── repl/          # Interactive mode
```

## Current Status

### Phase 5 Complete ✅

**Development Time**: ~15 hours across 5 phases
**Test Coverage**: 333 tests passing (1 known recursion stack-overflow bug)
**Code Quality**: 0 clippy warnings

### What's Implemented

**Core Language** ✅
- Dynamic typing with runtime type checking
- First-class functions with closures and function expressions
- Arrays, dicts, and indexing
- Automatic memory management (Rc-based GC)
- Member access syntax (obj.property)
- C-like syntax (curly braces, no semicolons)
- String interpolation: `"Hello ${name}"`
- String indexing: `str[0]`

**Control Flow** ✅
- if/else conditionals
- while loops
- for-in loops (iteration over arrays)
- break/continue statements
- return statements

**Error Handling** ✅
- `try { ... } catch(e) { ... }` - structured exception handling
- `throw value` - throw any value as an error
- Error propagation across function calls

**Module System** ✅
- `import module` - namespace import
- `from module import fn1, fn2` - selective import
- `import module as alias` - aliased import
- User-defined `.ae` modules from filesystem

**Built-in Functions** ✅
- I/O: `print()`, `println()`, `input()`, `read_file()`, `write_file()`
- Type introspection: `type()`, `len()`
- Type conversions: `int()`, `float()`, `str()`, `bool()`

**Collection Methods** ✅
- Arrays: `push()`, `pop()`, `length`
- Dicts: `keys()`, `values()`, `contains()`
- Strings: `upper()`, `lower()`, `trim()`, `split()`, `length`

**Standard Library** ✅ (35+ functions, written in Aether)
- **Core**: `range()`, `enumerate()`
- **Collections**: `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`
- **Math**: `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`
- **String**: `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`
- **Testing**: `assert_eq()`, `assert_true()`, `assert_false()`, `assert_null()`, `assert_not_null()`, `expect_error()`, `test()`, `test_summary()`

**Development Tools** ✅
- Interactive REPL with line editing and history
- File execution mode
- Comprehensive error messages
- 333 automated tests

### Test Coverage

```
Total: 333 tests passing ✅ (1 known stack-overflow in recursion limit test)

Unit Tests (99):
├── Lexer: 14 tests
├── Parser: 53 tests
├── Interpreter: 17 tests
└── Built-ins: 15 tests

Integration Tests (234):
├── Core features: 29 tests
├── Member access: 8 tests
├── Array methods: 8 tests
├── String methods: 8 tests
├── String indexing: 16 tests
├── String interpolation: 9 tests
├── Function expressions: 13 tests
├── Closures: 3 tests
├── Dict literals: 10 tests
├── Error handling: 10 tests
├── Module system: 13 tests
├── IO builtins: 5 tests
├── Stdlib core: 9 tests
├── Stdlib testing: 19 tests
├── Stdlib collections: 24 tests
├── Stdlib math: 26 tests
└── Stdlib string: 24 tests
```

## Roadmap

### Phase 4: Advanced Language Features ✅ Complete

- ✅ Function expressions: `fn(x) { return x * 2 }`
- ✅ String indexing: `text[0]`
- ✅ String interpolation: `"Hello ${name}"`
- ✅ Module system: `import`, `from ... import`, aliases
- ✅ Error handling: `try/catch/throw`
- ✅ Dict literals: `{"key": value}`
- ✅ IO builtins: `input()`, `read_file()`, `write_file()`

### Phase 5: Stdlib Expansion ✅ Complete (Base)

- ✅ Testing framework stdlib module
- ⏳ `json` module — json_parse(), json_stringify() (requires Rust builtins)
- ⏳ `time` module — clock(), sleep() (requires Rust builtins)
- ⏳ `http` module — http_get(), http_post() (requires reqwest dependency)
- ⏳ User-defined types / structs
- ⏳ Iterator protocol
- ⏳ Async/await support

### Phase 6: Performance Optimization

**Compiler Improvements**
- Bytecode compilation (instead of tree-walking)
- Constant folding
- Dead code elimination
- Tail call optimization

**Runtime Optimization**
- JIT compilation for hot paths
- Better garbage collection (generational GC)
- String interning
- Inline caching

**Benchmarking**
- Performance benchmark suite
- Memory profiling tools
- Regression testing

### Phase 7: Community & Adoption

**Documentation & Learning**
- Official website with playground
- Tutorial series (beginner to advanced)
- API reference
- Cookbook (common patterns)

**Community Building**
- GitHub discussions
- Discord/Slack community
- Contribution guidelines
- RFC process for major changes

**Real-World Usage**
- Example applications
- Case studies
- Community showcase
- Plugin ecosystem

## Design Principles

### Core Philosophy

1. **Simplicity First**
   - Start with straightforward implementations
   - Optimize for readability over performance (initially)
   - Progressive complexity as needed

2. **Test-Driven Development**
   - Write tests before implementation
   - Maintain 100% test success rate
   - Tests serve as documentation

3. **User Empowerment**
   - Stdlib in Aether (users can read and extend)
   - Clear error messages
   - Predictable behavior

4. **Pragmatic Evolution**
   - Ship working features quickly
   - Iterate based on usage
   - No premature optimization

### Technical Decisions

**Why Tree-Walking Interpreter?**
- Faster to implement and iterate
- Easier to debug
- Good enough performance for Phase 1-3
- Can optimize later with bytecode

**Why Rust?**
- Memory safety without GC overhead (for interpreter itself)
- Strong type system catches bugs early
- Excellent tooling (cargo, clippy)
- Fast enough for production use

**Why Rc for GC?**
- Simple reference counting
- Predictable memory behavior
- Good enough for single-threaded interpreter
- Can upgrade to mark-and-sweep if cycles become an issue

**Why Stdlib in Aether?**
- Validates language expressiveness ("dogfooding")
- User-readable implementations
- Easy to extend and customize
- Proves the language works for real code

## Resources

### Documentation

**Core Implementation:**
- **[DESIGN.md](DESIGN.html)** - Complete language specification
- **[DEVELOPMENT.md](DEVELOPMENT.html)** - Development guidelines and best practices
- **[LEXER.md](LEXER.html)** - Tokenization implementation
- **[PARSER.md](PARSER.html)** - Syntax analysis implementation
- **[INTERPRETER.md](INTERPRETER.html)** - Execution engine implementation
- **[REPL.md](REPL.html)** - Interactive mode implementation
- **[STDLIB.md](STDLIB.html)** - Standard library design
- **[GC_DESIGN.md](GC_DESIGN.html)** - Garbage collection architecture

**Language Features:**
- **[STRUCT.md](STRUCT.html)** - User-defined types with fields and methods
- **[ERROR_HANDLING.md](ERROR_HANDLING.html)** - Try/catch/throw exception handling
- **[STRING_FEATURES.md](STRING_FEATURES.html)** - String indexing, interpolation, slicing
- **[JSON.md](JSON.html)** - JSON parsing and serialization
- **[TIME.md](TIME.html)** - Time functions (clock, sleep)
- **[HTTP.md](HTTP.html)** - HTTP client functions
- **[MODULE_SYSTEM.md](MODULE_SYSTEM.html)** - Import and module loading

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

**Last Updated**: April 17, 2026
**Current Phase**: Phase 5 Complete ✅ (base)
**Next Phase**: Phase 5 continued - JSON, Time, HTTP modules
