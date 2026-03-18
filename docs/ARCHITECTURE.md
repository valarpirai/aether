# Aether Architecture & Roadmap

This document provides a high-level overview of Aether's architecture, current status, and future roadmap.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Current Status](#current-status)
- [Roadmap](#roadmap)
- [Design Principles](#design-principles)
- [Resources](#resources)

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
| **Lexer** | ✅ Complete | Tokenization | See [LEXER.md](LEXER.md) |
| **Parser** | ✅ Complete | Syntax analysis | See [PARSER.md](PARSER.md) |
| **Interpreter** | ✅ Complete | AST execution | See [INTERPRETER.md](INTERPRETER.md) |
| **REPL** | ✅ Complete | Interactive mode | See [REPL.md](REPL.md) |
| **Standard Library** | ✅ Complete | Core functions | See [STDLIB.md](STDLIB.md) |
| **Garbage Collection** | ✅ Complete | Memory management | See [GC_DESIGN.md](GC_DESIGN.md) |

### Project Structure

```
aether/
├── docs/              # Comprehensive documentation
├── stdlib/            # Standard library (written in Aether!)
├── examples/          # Example programs
├── tests/             # Integration tests (136 tests)
└── src/
    ├── lexer/         # Tokenization (14 tests)
    ├── parser/        # Parsing (53 tests)
    ├── interpreter/   # Execution (51 tests)
    └── repl/          # Interactive mode
```

## Current Status

### Phase 3 Complete ✅

**Development Time**: ~10 hours across 3 phases
**Test Coverage**: 230 tests passing (100% success rate)
**Code Quality**: 0 clippy warnings

### What's Implemented

**Core Language** ✅
- Dynamic typing with runtime type checking
- First-class functions with closures
- Arrays and indexing
- Automatic memory management (Rc-based GC)
- Member access syntax (obj.property)
- C-like syntax (curly braces, no semicolons)

**Control Flow** ✅
- if/else conditionals
- while loops
- for-in loops (iteration over arrays)
- break/continue statements
- return statements

**Built-in Functions** ✅
- I/O: `print()`, `println()`
- Type introspection: `type()`, `len()`
- Type conversions: `int()`, `float()`, `str()`, `bool()`

**Collection Methods** ✅
- Arrays: `push()`, `pop()`, `length`
- Strings: `upper()`, `lower()`, `trim()`, `split()`, `length`

**Standard Library** ✅ (28+ functions, written in Aether)
- **Core**: `range()`, `enumerate()`
- **Collections**: `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`
- **Math**: `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`
- **String**: `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`

**Development Tools** ✅
- Interactive REPL with line editing and history
- File execution mode
- Comprehensive error messages
- 230 automated tests

### Test Coverage

```
Total: 230 tests passing ✅

Unit Tests (94):
├── Lexer: 14 tests
├── Parser: 53 tests
├── Interpreter: 17 tests
└── Built-ins: 10 tests

Integration Tests (136):
├── Core features: 29 tests
├── Member access: 8 tests
├── Array methods: 8 tests
├── String methods: 8 tests
├── Stdlib core: 9 tests
├── Stdlib collections: 24 tests
├── Stdlib math: 26 tests
└── Stdlib string: 24 tests
```

### Recent Achievements

**Garbage Collection** 🎉
- Implemented reference-counted GC using `Rc<T>`
- Fixed critical 135 GB memory leak
- Memory reduction: 99%+ in tests
- All 230 tests passing with GC enabled

**Standard Library Bootstrapping** 🎉
- Stdlib written in Aether, not Rust
- Validates language expressiveness
- Embedded in binary using `include_str!()`
- Zero deployment complexity

## Roadmap

### Phase 4: Advanced Language Features (Next)

**Priority 1: Function Expressions** (High impact)
- Inline anonymous functions: `fn(x) { return x * 2 }`
- Lambda syntax (optional): `|x| x * 2`
- Enables cleaner functional programming
- **Estimated**: 3-4 hours, ~25 tests

**Priority 2: String Indexing** (Quick win)
- Direct character access: `text[0]`
- Eliminates `split("")` workaround
- Simplifies stdlib implementation
- **Estimated**: 1-2 hours, ~10 tests

**Priority 3: Module System**
- Import statements: `import module` or `from module import func`
- User-defined modules (load .ae files)
- Namespace management
- **Estimated**: 6-8 hours, ~35 tests

**Priority 4: Error Handling**
- Try/catch or Result types
- Error propagation
- Custom error types
- **Estimated**: 4-5 hours, ~20 tests

### Phase 5: Ecosystem & Tooling

**Standard Library Expansion**
- I/O module (file operations)
- JSON module (parsing, serialization)
- HTTP module (requests, simple server)
- Time module (dates, durations)
- Testing framework (written in Aether!)

**Development Tools**
- VS Code extension (syntax highlighting, LSP)
- Language server protocol (LSP) implementation
- Debugger support
- Performance profiling tools

**Package Management**
- Package manifest format
- Dependency resolution
- Central package registry
- Version management

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
- **[DESIGN.md](DESIGN.md)** - Complete language specification
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Development guidelines and best practices
- **[LEXER.md](LEXER.md)** - Tokenization implementation
- **[PARSER.md](PARSER.md)** - Syntax analysis implementation
- **[INTERPRETER.md](INTERPRETER.md)** - Execution engine implementation
- **[REPL.md](REPL.md)** - Interactive mode implementation
- **[STDLIB.md](STDLIB.md)** - Standard library design
- **[GC_DESIGN.md](GC_DESIGN.md)** - Garbage collection architecture

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

**Last Updated**: March 18, 2026
**Current Phase**: Phase 3 Complete ✅
**Next Phase**: Phase 4 - Advanced Language Features
