# Aether Development Guidelines

This document provides comprehensive development guidelines for contributing to the Aether programming language interpreter.

## Table of Contents
- [Code Organization](#code-organization)
- [Testing Strategy](#testing-strategy)
- [Error Handling](#error-handling)
- [Code Style](#code-style)
- [Incremental Development](#incremental-development)
- [Code Quality](#code-quality)
- [Performance](#performance)
- [Debugging](#debugging)
- [Documentation](#documentation)
- [Dependencies](#dependencies)

## Code Organization

### Module Structure

```
src/
├── main.rs                # Entry point, CLI handling
├── lib.rs                 # Library exports
├── lexer/
│   ├── mod.rs             # Lexer module exports
│   ├── token.rs           # Token type definitions
│   ├── scanner.rs         # Tokenization logic
│   └── lexer_tests.rs     # Lexer tests (14 tests) ✅
├── parser/
│   ├── mod.rs             # Parser module exports
│   ├── ast.rs             # AST node definitions
│   ├── parse.rs           # Recursive descent parser
│   └── parser_tests.rs    # Parser tests (53 tests) ✅
└── interpreter/           # ✅ Complete
    ├── mod.rs             # Interpreter module exports
    ├── value.rs           # Runtime value types (Rc-wrapped)
    ├── environment.rs     # Variable scoping
    ├── evaluator.rs       # Expression evaluation
    ├── builtins.rs        # Built-in functions
    ├── stdlib.rs          # Stdlib module loader
    ├── interpreter_tests.rs # Interpreter tests (17 tests) ✅
    └── builtins_tests.rs  # Built-in tests (15 tests) ✅
```

**Test File Convention**: Use `<module>_tests.rs` naming pattern for test files.

### Module Responsibilities

- **Lexer**: Converts source code into tokens
- **Parser**: Converts tokens into an Abstract Syntax Tree
- **Interpreter**: Evaluates the AST and executes code
- Each module should be independently testable

## Feature Implementation Decision Tree

### Should this be a Built-in (Rust) or Stdlib (Aether)?

When adding new functionality to Aether, use this decision tree:

**Built-in (Rust)** if it:
- Requires access to interpreter internals (e.g., `type()`, `len()`)
- Is performance-critical (e.g., operators, indexing, loops)
- Needs unsafe code or FFI (e.g., file I/O, system calls)
- Is a core I/O operation (e.g., `print()`, `println()`)
- Implements primitive operations that can't be expressed in Aether

**Standard Library (Aether)** if it:
- Can be built using existing primitives
- Contains user-modifiable logic
- Benefits from being readable by users
- Is a higher-level utility (e.g., `map()`, `filter()`, `range()`)
- Could be implemented by a user in their own code

**Examples**:
- ✅ Built-in: `print()`, `len()`, `type()`, `array.push()`, arithmetic operators
- ✅ Stdlib: `map()`, `filter()`, `range()`, `abs()`, `join()`, `reverse()`

**Rule of thumb**: If you can write it in Aether without accessing interpreter internals, put it in the standard library!

### Decision Process

```
Does it need interpreter internals? ──Yes──> Built-in (Rust)
           │
           No
           │
Is it performance-critical (10x+ difference)? ──Yes──> Built-in (Rust)
           │
           No
           │
Can users understand/modify it? ──Yes──> Stdlib (Aether)
           │
           No
           │
        Built-in (Rust)
```

## Testing Strategy

### Test-Driven Development

1. **Write tests first** before implementing features
2. **Test each component in isolation** before integration
3. **Follow the red-green-refactor cycle**:
   - Red: Write a failing test
   - Green: Write minimal code to pass the test
   - Refactor: Improve code while keeping tests green

### Test Organization

Tests are organized in separate `<module>_tests.rs` files:

```rust
// In src/lexer/lexer_tests.rs
use super::scanner::Scanner;
use super::token::{Token, TokenKind};

#[test]
fn test_tokenize_integer() {
    let mut scanner = Scanner::new("42");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens.len(), 2); // integer + EOF
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
}
```

Module configuration:
```rust
// In src/lexer/mod.rs
#[cfg(test)]
mod lexer_tests;
```

### Test Coverage Goals

- **Lexer**: Test all token types, edge cases, error conditions
- **Parser**: Test valid syntax, operator precedence, error recovery
- **Interpreter**: Test all operations, type checking, runtime errors
- **Integration**: Test complete programs end-to-end

### Running Tests

```bash
cargo test                    # Run all tests
cargo test lexer              # Run lexer tests only
cargo test -- --nocapture     # Show output during tests
cargo test -- --test-threads=1 # Run tests sequentially
```

## Error Handling

### Use Result Types

```rust
// Good: Return Result for operations that can fail
pub fn parse(tokens: &[Token]) -> Result<Expr, ParseError> {
    // parsing logic
}

// Bad: Using unwrap() or panic!() in production code
pub fn parse(tokens: &[Token]) -> Expr {
    tokens.first().unwrap() // Don't do this!
}
```

### Custom Error Types

Define clear, specific error types:

```rust
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch { expected: String, got: String },
    DivisionByZero,
}
```

### Error Messages

- **Be specific**: "Undefined variable 'x' at line 10" vs "Error"
- **Be helpful**: Suggest fixes when possible
- **Include context**: Line numbers, column numbers, surrounding code

## Code Style

### Rust Idioms

- Use `match` for exhaustive pattern matching
- Prefer `if let` for single-pattern matches
- Use iterators instead of explicit loops where appropriate
- Leverage the type system for safety

### Naming Conventions

- `snake_case` for functions and variables
- `PascalCase` for types and enums
- `SCREAMING_SNAKE_CASE` for constants
- Clear, descriptive names (e.g., `tokenize_number` vs `tn`)

### Documentation

```rust
/// Tokenizes a string of Aether source code into tokens.
///
/// # Arguments
/// * `input` - The source code to tokenize
///
/// # Returns
/// A vector of tokens or a lexer error
///
/// # Example
/// ```
/// let tokens = tokenize("let x = 42")?;
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexerError> {
    // implementation
}
```

## Incremental Development

### Build in Small Steps

1. **Don't create all files at once**
2. **Implement one feature completely** before moving to the next
3. **Verify each step works** before proceeding

### Phase 1 Example

1. Define basic token types (integers, operators)
2. Write tests for tokenizing integers
3. Implement integer tokenization
4. Write tests for tokenizing operators
5. Implement operator tokenization
6. Test complete tokenization of simple expressions
7. Only then move to the next feature

### Commit Frequency

- Commit after each working feature
- Keep commits focused and atomic
- Write clear commit messages explaining the "why"

## Code Quality

### Before Committing

```bash
cargo fmt               # Format code
cargo clippy            # Run linter
cargo test              # Run all tests
cargo build --release   # Ensure release build works
```

### Clippy Warnings

- Address all clippy warnings
- Use `#[allow(clippy::...)]` sparingly and with justification
- Prefer fixing the code over silencing warnings

### Code Review Checklist

- [ ] All tests pass
- [ ] New code has tests
- [ ] Error handling is robust
- [ ] Documentation is clear
- [ ] No clippy warnings
- [ ] Code follows Rust idioms
- [ ] Commit messages are descriptive

## Performance

### Start Simple

- **Correctness first**, performance second
- Profile before optimizing
- Don't prematurely optimize

### When Optimizing

- Use `cargo bench` for benchmarks
- Profile with `cargo flamegraph` or similar tools
- Document why optimizations are necessary

## Debugging

### Debug Output

```rust
// Use Debug trait for development
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

// Enable detailed debugging
println!("{:?}", token);  // Compact
println!("{:#?}", token); // Pretty-printed
```

### REPL for Testing

- Use the REPL to quickly test language features
- Add debug commands (e.g., `_tokens`, `_ast`) to inspect internals

## Documentation

### Keep Updated

- Update `docs/DESIGN.md` when making design changes
- Document design decisions and trade-offs
- Keep CLAUDE.md current with project status

### Code Comments

- Explain **why**, not **what**
- Document complex algorithms
- Add examples for non-obvious code

## Dependencies

### Minimize Dependencies

- Prefer standard library when possible
- Only add dependencies that provide significant value
- Review and understand dependencies before adding

### Useful Dependencies

Consider these for the interpreter:
- `clap` - Command-line argument parsing
- `rustyline` - REPL readline support
- `colored` - Terminal colors for errors

## Continuous Integration

### Future CI Setup

When setting up CI, include:
- Run tests on multiple platforms
- Check formatting (`cargo fmt --check`)
- Run clippy (`cargo clippy -- -D warnings`)
- Build documentation (`cargo doc --no-deps`)

## Common Pitfalls When Extending Aether

### 1. Forgetting Rc Wrappers

**Problem**: Creating values without using helper methods.

❌ **Wrong**:
```rust
Value::String("hello".to_string())  // Compilation error - expects Rc<String>
Value::Array(vec![Value::Int(1)])   // Compilation error - expects Rc<Vec<Value>>
```

✅ **Right**:
```rust
Value::string("hello".to_string())  // Uses helper method
Value::array(vec![Value::Int(1)])   // Uses helper method
```

**Why**: Strings and Arrays are wrapped in `Rc<T>` for garbage collection. Always use the helper methods `Value::string()` and `Value::array()`.

### 2. Not Using --test-threads=1

**Problem**: Running tests without limiting thread count causes memory pressure.

❌ **Wrong**:
```bash
cargo test  # May cause OOM on large test suites
```

✅ **Right**:
```bash
cargo test -- --test-threads=1  # Sequential execution, lower memory usage
```

**Why**: Parallel test execution can consume excessive memory (135 GB observed without GC). Sequential execution is safer.

### 3. Adding Built-ins Instead of Stdlib

**Problem**: Implementing features in Rust that could be written in Aether.

❌ **Wrong**: Adding `max()` function to `src/interpreter/builtins.rs`

✅ **Right**: Implementing `max()` in `stdlib/math.ae`

**Why**: Stdlib functions are:
- User-readable and modifiable
- Easier to maintain and test
- Prove the language is expressive enough

**Rule**: If it can be written in Aether, it belongs in stdlib!

### 4. Forgetting Optional Parameter Support

**Problem**: Not handling `null` for optional parameters.

❌ **Wrong**:
```aether
fn range(start, end) {
    // Assumes both parameters are always provided
    let i = start
    while (i < end) { ... }
}
```

✅ **Right**:
```aether
fn range(start, end) {
    // Handle single argument: range(n) -> range(0, n)
    if (end == null) {
        end = start
        start = 0
    }
    let i = start
    while (i < end) { ... }
}
```

**Why**: Aether supports optional parameters. Functions should handle `null` for missing arguments.

### 5. Not Testing Edge Cases

**Problem**: Only testing the happy path.

❌ **Insufficient**:
```rust
#[test]
fn test_division() {
    assert_eq!(eval("10 / 2"), Value::Int(5));
}
```

✅ **Comprehensive**:
```rust
#[test]
fn test_division() {
    assert_eq!(eval("10 / 2"), Value::Int(5));
    assert_eq!(eval("10.0 / 3.0"), Value::Float(3.333...));
    assert!(eval("10 / 0").is_err());  // Division by zero
    assert!(eval("10 / null").is_err()); // Type error
}
```

**Why**: Edge cases include:
- Empty arrays/strings
- Null values
- Type mismatches
- Division by zero
- Out of bounds access
- Negative numbers

Always test error conditions, not just success cases!

### 6. Mutating Rc-wrapped Values Incorrectly

**Problem**: Trying to mutate shared data.

❌ **Wrong**:
```rust
if let Value::Array(arr) = &mut value {
    arr.push(Value::Int(42));  // Error: Rc is not mutable
}
```

✅ **Right**:
```rust
if let Value::Array(arr) = &value {
    let mut new_arr = arr.as_ref().clone();  // Clone the Vec
    new_arr.push(Value::Int(42));
    value = Value::array(new_arr);  // Create new Rc-wrapped array
}
```

**Why**: `Rc<T>` provides shared ownership but not interior mutability. To modify, clone the data, modify it, and create a new `Rc`.

### 7. Not Using TDD

**Problem**: Writing implementation before tests.

❌ **Wrong**:
1. Write entire feature
2. Write tests later
3. Find bugs
4. Fix and repeat

✅ **Right** (TDD workflow):
1. Write failing test
2. Write minimal code to pass
3. Refactor
4. Repeat

**Why**: TDD ensures:
- Clear requirements before coding
- All code is tested
- Easier debugging (smaller changes)
- Better design (testable code)

## Learning Resources

### Rust Interpreter Resources

- "Crafting Interpreters" by Robert Nystrom
- "Writing An Interpreter In Go" by Thorsten Ball
- Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/

---

**Last Updated**: April 17, 2026
**Phase**: 5 Complete (base)
**Status**: Comprehensive development guidelines with TDD workflow and common pitfalls
