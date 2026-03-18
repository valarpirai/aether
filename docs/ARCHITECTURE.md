# Aether Architecture & Roadmap

This document provides an overview of the Aether interpreter architecture, implemented features, and pending work.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Module Structure](#module-structure)
- [Data Flow](#data-flow)
- [Implemented Features](#implemented-features)
- [Pending Features](#pending-features)
- [Phase Breakdown](#phase-breakdown)

## Architecture Overview

Aether is a **tree-walking interpreter** written in Rust. The interpreter follows a classic three-stage architecture:

```
Source Code (.ae)
      ↓
   [Lexer]  ──→  Tokens
      ↓
   [Parser] ──→  Abstract Syntax Tree (AST)
      ↓
[Interpreter] ──→  Execution / Output
```

### Design Principles

1. **Simplicity First**: Start with a straightforward tree-walking interpreter
2. **Test-Driven Development**: Write tests before implementation
3. **Incremental Development**: Complete one feature at a time
4. **Type Safety**: Leverage Rust's type system for correctness
5. **Clear Error Messages**: Provide helpful feedback to users

## Module Structure

### Current Implementation

```
aether/
├── Cargo.toml               # Project configuration
├── README.md                # User-facing documentation
├── CLAUDE.md                # Development guide for Claude Code
├── docs/
│   ├── DESIGN.md           # Language specification
│   ├── DEVELOPMENT.md      # Development guidelines
│   └── ARCHITECTURE.md     # This file
└── src/
    ├── main.rs             # CLI entry point
    ├── lib.rs              # Library exports
    │
    ├── lexer/              # Lexical analysis
    │   ├── mod.rs          # Module exports
    │   ├── token.rs        # Token definitions
    │   ├── scanner.rs      # Tokenization logic
    │   └── lexer_tests.rs  # 14 tests ✅
    │
    ├── parser/             # Syntax analysis
    │   ├── mod.rs          # Module exports
    │   ├── ast.rs          # AST node definitions
    │   ├── parse.rs        # Recursive descent parser
    │   └── parser_tests.rs # 23 tests ✅
    │
    └── interpreter/        # Execution (TODO)
        ├── mod.rs          # Module exports
        ├── value.rs        # Runtime values
        ├── environment.rs  # Variable scoping
        └── evaluator.rs    # AST evaluation
```

### Module Responsibilities

#### 1. Lexer (Complete ✅)
**Purpose**: Convert source code into a stream of tokens

**Components**:
- `Token`: Represents a single lexical unit (keyword, operator, literal, etc.)
- `Scanner`: Reads source code character-by-character and produces tokens
- `LexerError`: Custom error types for lexical errors

**Key Features**:
- All token types (integers, floats, strings, keywords, operators)
- String escape sequences (`\n`, `\t`, `\\`, `\"`)
- Single-line (`//`) and multi-line (`/* */`) comments
- Proper error handling with line/column information

#### 2. Parser (Complete - Core Features ✅)
**Purpose**: Convert tokens into an Abstract Syntax Tree

**Components**:
- `Expr`: Expression AST nodes (literals, binary ops, unary ops, etc.)
- `Stmt`: Statement AST nodes (let, if, while, for, return, etc.)
- `Program`: Root AST node containing all statements
- `Parser`: Recursive descent parser with operator precedence
- `ParseError`: Custom error types for syntax errors

**Key Features**:
- ✅ Expression parsing with proper precedence
- ✅ Statement parsing (declarations, control flow)
- ⏳ Function declarations (pending)
- ⏳ Function calls (pending)
- ⏳ Arrays and indexing (pending)
- ⏳ Assignment statements (pending)

#### 3. Interpreter (Not Started ⏳)
**Purpose**: Execute the AST and produce results

**Planned Components**:
- `Value`: Runtime value representation (int, float, string, bool, null, array, dict, function)
- `Environment`: Variable scoping and storage
- `Evaluator`: Tree-walking interpreter that evaluates expressions and executes statements
- `RuntimeError`: Custom error types for runtime errors

**Planned Features**:
- Variable storage and retrieval
- Expression evaluation
- Statement execution
- Function call stack
- Scope management
- Built-in functions

## Data Flow

### Compilation Pipeline

```
┌─────────────┐
│ Source Code │  "let x = 1 + 2"
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Lexer     │  Tokenization
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Tokens    │  [Let, Identifier("x"), Equal, Integer(1), Plus, Integer(2)]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  Syntax Analysis
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     AST     │  Stmt::Let("x", Expr::Binary(...))
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Interpreter │  Execution
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Output    │  x = 3
└─────────────┘
```

### Error Handling Flow

```
┌─────────────┐
│ Lexer Error │ → LexerError → Display to user
└─────────────┘

┌─────────────┐
│ Parse Error │ → ParseError → Display to user
└─────────────┘

┌─────────────┐
│Runtime Error│ → RuntimeError → Display to user with stack trace
└─────────────┘
```

## Implemented Features

### ✅ Phase 1: Core Interpreter (Partial)

#### Lexer (Complete)
- [x] Token definitions for all language constructs
- [x] Integer literals (`42`)
- [x] Float literals (`3.14`)
- [x] String literals with escape sequences (`"hello\n"`)
- [x] Boolean literals (`true`, `false`)
- [x] Null literal (`null`)
- [x] Identifiers and keywords
- [x] All operators (`+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`)
- [x] Delimiters (`(`, `)`, `{`, `}`, `[`, `]`, `,`, `.`, `:`)
- [x] Single-line comments (`//`)
- [x] Multi-line comments (`/* */`)
- [x] Error handling with line/column information
- [x] **Tests**: 14 passing

#### Parser (Partial)
**Expressions:**
- [x] Primary expressions (literals, identifiers)
- [x] Grouped expressions `(expr)`
- [x] Unary expressions (`-expr`, `!expr`)
- [x] Binary arithmetic (`+`, `-`, `*`, `/`, `%`)
- [x] Comparison operators (`<`, `>`, `<=`, `>=`, `==`, `!=`)
- [x] Logical operators (`&&`, `||`)
- [x] Proper operator precedence

**Statements:**
- [x] Expression statements
- [x] Variable declarations (`let x = value`)
- [x] Block statements (`{ ... }`)
- [x] If statements (`if (cond) { ... }`)
- [x] If-else statements (`if (cond) { ... } else { ... }`)
- [x] While loops (`while (cond) { ... }`)
- [x] For-in loops (`for item in iterable { ... }`)
- [x] Return statements (`return expr`)
- [x] Break statements (`break`)
- [x] Continue statements (`continue`)

- [x] **Tests**: 23 passing

## Pending Features

### 🚧 Phase 1: Core Interpreter (Remaining)

#### Parser (In Progress)
- [ ] Function declarations (`fn name(params) { body }`)
- [ ] Function calls (`function(args)`)
- [ ] Array literals (`[1, 2, 3]`)
- [ ] Array indexing (`array[index]`)
- [ ] Dictionary literals (`{"key": value}`)
- [ ] Member access (`object.property`)
- [ ] Assignment statements (`x = value`, `x += value`)
- [ ] Compound assignments (`+=`, `-=`, `*=`, `/=`)

#### Interpreter (Not Started)
- [ ] Runtime value types
- [ ] Expression evaluation
- [ ] Statement execution
- [ ] Variable environment (scoping)
- [ ] Function call stack
- [ ] Basic type checking
- [ ] Error reporting

#### REPL (Not Started)
- [ ] Interactive prompt
- [ ] Line editing support (rustyline)
- [ ] Command history
- [ ] Multi-line input
- [ ] Debug commands (`_tokens`, `_ast`, `_env`)

### ⏳ Phase 2: Basic Features
- [ ] Function closures
- [ ] Nested scopes
- [ ] String interpolation evaluation
- [ ] Range expressions (`range(0, 10)`)
- [ ] Type conversion functions (`int()`, `float()`, `str()`, `bool()`)
- [ ] Basic I/O (`print()`, `println()`, `input()`)

### ⏳ Phase 3: Collections & Built-ins
- [ ] Array methods (`push()`, `pop()`, `length`)
- [ ] Dictionary methods (`keys()`, `values()`, `get()`)
- [ ] String methods (`upper()`, `lower()`, `length`)
- [ ] Set operations
- [ ] Built-in functions (`len()`, `type()`)
- [ ] Collection iteration in for loops

### ⏳ Phase 4: Module System
- [ ] Import statements (`import module`)
- [ ] Module namespaces
- [ ] Standard library modules
- [ ] Module search paths
- [ ] Circular import detection

### ⏳ Phase 5: Advanced Features (Future)
- [ ] Error handling (try/catch or Result types)
- [ ] Pattern matching
- [ ] Async/await
- [ ] Package manager
- [ ] Optimizations (constant folding, etc.)

## Phase Breakdown

### Phase 1: Core Interpreter ✅ 70% Complete

**Goal**: Build a working interpreter that can execute simple Aether programs

**Completed**:
- ✅ Lexer (100%)
- ✅ Parser - Expressions (100%)
- ✅ Parser - Statements (100%)

**In Progress**:
- 🚧 Parser - Advanced features (40%)
  - Need: function declarations, calls, arrays, assignments

**Remaining**:
- ⏳ Interpreter (0%)
- ⏳ REPL (0%)

**Estimated Effort**: 2-3 more development sessions

### Phase 2: Basic Features ⏳ 0% Complete

**Goal**: Implement core language features (functions, scoping, basic built-ins)

**Key Deliverables**:
- Working functions with closures
- Proper variable scoping
- String interpolation
- Type conversions
- Basic I/O

**Estimated Effort**: 3-4 development sessions

### Phase 3: Collections & Built-ins ⏳ 0% Complete

**Goal**: Complete collection support and standard library

**Key Deliverables**:
- All collection types working
- Collection methods implemented
- Standard built-in functions
- Comprehensive test suite

**Estimated Effort**: 3-4 development sessions

### Phase 4: Module System ⏳ 0% Complete

**Goal**: Add module/import system

**Key Deliverables**:
- Import mechanism
- Module resolution
- Standard library organization
- Example programs using modules

**Estimated Effort**: 2-3 development sessions

## Testing Strategy

### Current Test Coverage

```
Total: 37 tests passing ✅

Lexer Tests (14):
├── Token creation
├── All literal types
├── All operators
├── Keywords and identifiers
├── String escapes
├── Comments (single/multi-line)
└── Error handling

Parser Tests (23):
├── Primary expressions (6)
├── Unary expressions (2)
├── Binary operators (6)
├── Operator precedence (1)
├── Variable declarations (2)
├── Control flow (4)
└── Loops (2)

Interpreter Tests (0):
└── To be implemented
```

### Testing Principles

1. **Unit Tests**: Test each component in isolation
2. **Integration Tests**: Test complete programs end-to-end
3. **Error Tests**: Verify error messages and recovery
4. **Edge Cases**: Test boundary conditions
5. **TDD Approach**: Write tests before implementation

## Performance Considerations

### Current Focus: Correctness

The initial implementation prioritizes correctness and clarity over performance:
- Tree-walking interpretation (no bytecode optimization)
- Simple recursive descent parser
- Straightforward environment implementation

### Future Optimizations

Once the interpreter is working correctly, consider:
1. **Bytecode compilation**: Compile AST to bytecode
2. **Constant folding**: Evaluate constant expressions at compile time
3. **Tail call optimization**: For recursive functions
4. **JIT compilation**: For hot code paths
5. **Better garbage collection**: Generational GC or reference counting

## Dependencies

### Current Dependencies
```toml
[dependencies]
# None - using only Rust standard library
```

### Planned Dependencies
```toml
[dependencies]
clap = "4.0"        # CLI argument parsing
rustyline = "12.0"  # REPL line editing
colored = "2.0"     # Terminal colors

[dev-dependencies]
criterion = "0.5"   # Benchmarking (future)
```

## Development Workflow

### Adding a New Feature

1. **Design**: Document the feature in DESIGN.md
2. **Test**: Write tests in `<module>_tests.rs`
3. **Implement**: Write minimal code to pass tests
4. **Verify**: Run `cargo test` and `cargo clippy`
5. **Document**: Update ARCHITECTURE.md and CLAUDE.md
6. **Commit**: Create focused, atomic commits

### Quality Checks

Before committing:
```bash
cargo fmt                     # Format code
cargo clippy -- -D warnings   # Check for issues
cargo test                    # Run all tests
cargo build --release         # Verify release build
```

## Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development guidelines including:
- Code organization
- Testing strategies
- Error handling patterns
- Code style guidelines
- Debugging techniques

## Resources

- **Language Design**: [DESIGN.md](DESIGN.md)
- **Development Guide**: [DEVELOPMENT.md](DEVELOPMENT.md)
- **Main README**: [../README.md](../README.md)

### External Resources
- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- [Rust Programming Language Book](https://doc.rust-lang.org/book/)
