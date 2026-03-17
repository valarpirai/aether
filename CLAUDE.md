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

## Development Guidelines

### Code Organization

#### Module Structure
Organize code into clear, focused modules:

```
src/
├── main.rs           # Entry point, CLI handling, REPL
├── lexer/
│   ├── mod.rs        # Lexer module exports
│   ├── token.rs      # Token type definitions
│   └── scanner.rs    # Tokenization logic
├── parser/
│   ├── mod.rs        # Parser module exports
│   ├── ast.rs        # AST node definitions
│   └── parser.rs     # Parsing logic
├── interpreter/
│   ├── mod.rs        # Interpreter module exports
│   ├── value.rs      # Runtime value types
│   ├── environment.rs # Variable scoping
│   └── evaluator.rs  # Expression evaluation
└── lib.rs            # Library exports
```

#### Module Responsibilities
- **Lexer**: Converts source code into tokens
- **Parser**: Converts tokens into an AST
- **Interpreter**: Evaluates the AST and executes code
- Each module should be independently testable

### Testing Strategy

#### Test-Driven Development
1. **Write tests first** before implementing features
2. **Test each component in isolation** before integration
3. **Follow the red-green-refactor cycle**:
   - Red: Write a failing test
   - Green: Write minimal code to pass the test
   - Refactor: Improve code while keeping tests green

#### Test Organization
```rust
// In src/lexer/scanner.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_integer() {
        let input = "42";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Integer);
    }
}
```

#### Test Coverage Goals
- **Lexer**: Test all token types, edge cases, error conditions
- **Parser**: Test valid syntax, operator precedence, error recovery
- **Interpreter**: Test all operations, type checking, runtime errors
- **Integration**: Test complete programs end-to-end

#### Running Tests
```bash
cargo test                    # Run all tests
cargo test lexer              # Run lexer tests only
cargo test -- --nocapture     # Show output during tests
cargo test -- --test-threads=1 # Run tests sequentially
```

### Error Handling

#### Use Result Types
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

#### Custom Error Types
Define clear, specific error types:
```rust
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, usize),
    UnterminatedString(usize),
    InvalidNumber(String, usize),
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch { expected: String, got: String },
    DivisionByZero,
}
```

#### Error Messages
- **Be specific**: "Undefined variable 'x' at line 10" vs "Error"
- **Be helpful**: Suggest fixes when possible
- **Include context**: Line numbers, column numbers, surrounding code

### Code Style

#### Rust Idioms
- Use `match` for exhaustive pattern matching
- Prefer `if let` for single-pattern matches
- Use iterators instead of explicit loops where appropriate
- Leverage the type system for safety

#### Naming Conventions
- `snake_case` for functions and variables
- `PascalCase` for types and enums
- `SCREAMING_SNAKE_CASE` for constants
- Clear, descriptive names (e.g., `tokenize_number` vs `tn`)

#### Documentation
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

### Incremental Development

#### Build in Small Steps
1. **Don't create all files at once**
2. **Implement one feature completely** before moving to the next
3. **Verify each step works** before proceeding

#### Phase 1 Example
1. Define basic token types (integers, operators)
2. Write tests for tokenizing integers
3. Implement integer tokenization
4. Write tests for tokenizing operators
5. Implement operator tokenization
6. Test complete tokenization of simple expressions
7. Only then move to the next feature

#### Commit Frequency
- Commit after each working feature
- Keep commits focused and atomic
- Write clear commit messages explaining the "why"

### Code Quality

#### Before Committing
```bash
cargo fmt               # Format code
cargo clippy            # Run linter
cargo test              # Run all tests
cargo build --release   # Ensure release build works
```

#### Clippy Warnings
- Address all clippy warnings
- Use `#[allow(clippy::...)]` sparingly and with justification
- Prefer fixing the code over silencing warnings

#### Code Review Checklist
- [ ] All tests pass
- [ ] New code has tests
- [ ] Error handling is robust
- [ ] Documentation is clear
- [ ] No clippy warnings
- [ ] Code follows Rust idioms
- [ ] Commit messages are descriptive

### Performance Considerations

#### Start Simple
- **Correctness first**, performance second
- Profile before optimizing
- Don't prematurely optimize

#### When Optimizing
- Use `cargo bench` for benchmarks
- Profile with `cargo flamegraph` or similar tools
- Document why optimizations are necessary

### Debugging

#### Debug Output
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

#### REPL for Testing
- Use the REPL to quickly test language features
- Add debug commands (e.g., `_tokens`, `_ast`) to inspect internals

### Documentation

#### Keep Updated
- Update `docs/DESIGN.md` when making design changes
- Document design decisions and trade-offs
- Keep CLAUDE.md current with project status

#### Code Comments
- Explain **why**, not **what**
- Document complex algorithms
- Add examples for non-obvious code

### Dependencies

#### Minimize Dependencies
- Prefer standard library when possible
- Only add dependencies that provide significant value
- Review and understand dependencies before adding

#### Useful Dependencies
Consider these for the interpreter:
- `clap` - Command-line argument parsing
- `rustyline` - REPL readline support
- `colored` - Terminal colors for errors

### Continuous Integration

#### Future CI Setup
When setting up CI, include:
- Run tests on multiple platforms
- Check formatting (`cargo fmt --check`)
- Run clippy (`cargo clippy -- -D warnings`)
- Build documentation (`cargo doc --no-deps`)

### Learning Resources

#### Rust Interpreter Resources
- "Crafting Interpreters" by Robert Nystrom
- "Writing An Interpreter In Go" by Thorsten Ball
- Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/