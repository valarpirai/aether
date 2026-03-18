# Aether Programming Language

A general-purpose, dynamically typed programming language with automatic memory management.

## Overview

Aether combines the familiarity of C-like syntax with the ease of use of modern interpreted languages. It features:

- **Dynamic typing** with runtime type checking
- **Automatic memory management** through garbage collection
- **C-like syntax** with curly braces, no semicolons
- **First-class functions** with closures
- **Block-scoped variables** using the `let` keyword
- **Modern control flow** with range-based and for-each loops
- **String interpolation** with `"Hello ${name}"` syntax
- **Interactive REPL** for rapid development

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/aether.git
cd aether

# Build the project
cargo build

# Run the interpreter
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Example Program

Create a file `example.ae`:

```aether
fn fibonacci(n) {
    if (n <= 1) {
        return n
    }

    let a = 0
    let b = 1

    for i in range(2, n + 1) {
        let temp = a + b
        a = b
        b = temp
    }

    return b
}

fn main() {
    println("Fibonacci Sequence")

    for i in range(0, 10) {
        let result = fibonacci(i)
        println("fib(${i}) = ${result}")
    }
}
```

Run it with:
```bash
aether example.ae
```

## Documentation

- **[Language Design](docs/DESIGN.md)** - Complete language specification
- **[Architecture & Roadmap](docs/ARCHITECTURE.md)** - System architecture, features, and roadmap
- **[Development Guidelines](docs/DEVELOPMENT.md)** - Code organization and best practices
- **[Project Guide](CLAUDE.md)** - Quick reference for contributors

## Development Status

**Current Phase**: Phase 1 - Core Interpreter (70% Complete)

### Completed ✅
- ✅ Language design specification ([DESIGN.md](docs/DESIGN.md))
- ✅ Development environment setup
- ✅ Project structure with comprehensive documentation
- ✅ **Lexer (100%)** - Full tokenization with 14 tests
  - All token types (literals, keywords, operators)
  - String escape sequences
  - Comments (single/multi-line)
  - Error handling
- ✅ **Parser (80%)** - Recursive descent parser with 23 tests
  - All expressions with proper precedence
  - Variable declarations (`let`)
  - Control flow (if/else, while, for)
  - Return/break/continue statements

### In Progress 🚧
- 🚧 **Parser (Advanced)** - Remaining features:
  - Function declarations
  - Function calls
  - Arrays and indexing
  - Assignment statements
- 🚧 **Interpreter** - Starting soon
- 🚧 **REPL** - Starting soon

### Test Coverage
- **37 tests passing** (14 lexer + 23 parser)
- **0 clippy warnings**
- **100% passing rate**

### Up Next ⏳
- ⏳ Complete parser (functions, arrays, assignments)
- ⏳ Tree-walking interpreter
- ⏳ Interactive REPL
- ⏳ Phase 2: Functions and closures
- ⏳ Phase 3: Collections & built-ins
- ⏳ Phase 4: Module system

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed roadmap and feature checklist.

## Contributing

Contributions are welcome! Please see [CLAUDE.md](CLAUDE.md) for development guidelines.

### Development Workflow

1. Write tests first (TDD approach)
2. Implement the feature
3. Ensure all tests pass
4. Run `cargo fmt` and `cargo clippy`
5. Commit with clear messages

## License

MIT License - see [LICENSE](LICENSE) for details

## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
