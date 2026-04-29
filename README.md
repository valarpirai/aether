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
// Functional programming with stdlib
fn square(x) { return x * x }
fn is_even(x) { return x % 2 == 0 }

fn main() {
    println("=== Functional Pipeline Demo ===")

    // Sum of squares of even numbers from 1-10
    let numbers = range(1, 11)
    let squares = map(numbers, square)
    let even_squares = filter(squares, is_even)
    let total = sum(even_squares)

    println("Numbers:", numbers)
    println("Squares:", squares)
    println("Even squares:", even_squares)
    println("Sum:", total)  // 220

    println()
    println("=== Text Processing ===")

    let words = ["hello", "beautiful", "world"]
    let sentence = join(words, " ")
    println("Original:", sentence)
    println("Uppercase:", sentence.upper())
    println("Reversed:", reverse(sentence))
}
```

Run it with:
```bash
cargo run example.ae
```

## Running Aether Programs

### Quick Start

```bash
# Run a program (development mode)
cargo run myprogram.ae

# Start interactive REPL
cargo run

# Build optimized binary
cargo build --release

# Run with optimized binary
./target/release/aether myprogram.ae
```

### REPL Mode (Interactive)

Start the REPL by running Aether without arguments:

```bash
cargo run
# or
./target/release/aether
```

**REPL Features:**
- Line editing with arrow keys
- Command history (up/down arrows)
- Multi-line support

**Special Commands:**
- `_help` - Show help information
- `_env` - Display environment variables
- `_exit` - Exit the REPL

**Example REPL Session:**
```
Welcome to Aether REPL v0.1.0
Type _help for more information, _exit to quit

>>> let x = 42
null
>>> x * 2
84
>>> fn greet(name) { return "Hello, " + name }
null
>>> greet("World")
Hello, World
>>> let nums = range(1, 6)
null
>>> map(nums, fn(x) { return x * x })
[1, 4, 9, 16, 25]
>>> _exit
```

### File Mode (Running Scripts)

Run Aether programs from files:

```bash
# Using cargo (development)
cargo run path/to/program.ae

# Using built binary
./target/release/aether path/to/program.ae
```

**Program Requirements:**
- Every program needs a `main()` function as the entry point
- Standard library functions are automatically available

**Example Program (`hello.ae`):**
```aether
fn main() {
    println("Hello, Aether!")

    let numbers = range(1, 11)
    let sum = sum(numbers)
    println("Sum of 1-10:", sum)
}
```

Run it:
```bash
cargo run hello.ae
```

### Example Programs

Try the included example programs:

```bash
# Simple hello world
cargo run examples/hello.ae

# Standard library demos
cargo run examples/stdlib_demo.ae       # Core functions (range, enumerate)
cargo run examples/collections_demo.ae  # map, filter, reduce
cargo run examples/math_demo.ae         # Math utilities
cargo run examples/string_demo.ae       # String operations

# Performance test
cargo run examples/gc_stress_test.ae    # GC stress test
```

### Building for Production

Create an optimized release build:

```bash
# Build with optimizations
cargo build --release

# Binary location
./target/release/aether

# Optional: Install globally
cargo install --path .

# Now run from anywhere
aether myprogram.ae
```

### Usage Summary

```
USAGE:
    aether              # Start interactive REPL
    aether <file.ae>    # Run an Aether program

EXAMPLES:
    aether                          # Interactive mode
    aether hello.ae                 # Run hello.ae
    aether examples/stdlib_demo.ae  # Run example
    aether /path/to/script.ae       # Run from any path
```

## Documentation

- **[Language Design](docs/DESIGN.md)** - Complete language specification
- **[Architecture & Roadmap](docs/ARCHITECTURE.md)** - System architecture, features, and roadmap
- **[Development Guidelines](docs/DEVELOPMENT.md)** - Code organization and best practices
- **[Project Guide](CLAUDE.md)** - Quick reference for contributors

## Development Status

**Current Phase**: Phase 5 Sprint 2 - Advanced Types (Complete ✅)

### What's Working
- ✅ **Complete Interpreter** - Tree-walking interpreter with full language support
- ✅ **485 Tests Passing** - 100% success rate (99 unit + 386 integration)
- ✅ **Garbage Collection** - Reference-counted memory management (Rc-based)
- ✅ **Standard Library** - 28+ functions written in Aether itself
  - Core: `range()`, `enumerate()`
  - Collections: `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()`
  - Math: `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()`
  - String: `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()`
- ✅ **Built-in Functions** - I/O, type introspection, conversions, JSON, time, HTTP
- ✅ **Collection Methods** - Array, Dict, Set with comprehensive operations
- ✅ **Set Type** - Unique collections with union, intersection, difference operations
- ✅ **Structs** - User-defined types with fields and methods
- ✅ **Interactive REPL** - Line editing with history
- ✅ **First-class Functions** - Functions with closures and recursion

### Test Coverage
- **485 tests passing** ✅
  - 99 unit tests
  - 386 integration tests
- **0 clippy warnings**
- **100% success rate**

### Recent Achievements
- 🎉 **Fixed 135 GB Memory Leak** - Implemented GC (99%+ memory reduction)
- 🎉 **Stdlib Bootstrapping** - Standard library written in Aether, not Rust
- 🎉 **Zero Deployment** - Stdlib embedded in binary using `include_str!()`

### Up Next (Phase 5 continued)
- ⏳ Iterator protocol
- ⏳ Async/await support
- ⏳ Performance optimizations
- ⏳ Feature documentation (structs, error handling, JSON, HTTP, etc.)

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed roadmap and [CLAUDE.md](CLAUDE.md) for complete project status.

## Contributing

Contributions are welcome! Please see [CLAUDE.md](CLAUDE.md) for development guidelines.

### Development Workflow

1. Write tests first (TDD approach)
2. Implement the feature
3. Ensure all tests pass
4. Run `cargo fmt` and `cargo clippy`
5. Commit with clear messages

## Code Quality

| Area | Score | Notes |
|------|-------|-------|
| `value.rs` | 9/10 | Clean enum design, correct Rc/RefCell/Weak, comprehensive Display |
| `operators.rs` | 9/10 | Short-circuit eval, numeric promotion, null coalescing |
| `io_pool.rs` | 9/10 | Thread-safe, primitives-only boundary, clean task types |
| `environment.rs` / `evaluator/mod.rs` | 8/10 | Solid; three clear sub-structs (CallContext, ModuleLoader, AsyncRuntime) |
| `builtins.rs` | 8/10 | Good coverage; structured typed errors; some HTTP option duplication |
| `evaluator/functions.rs` / `members.rs` | 7/10 | Some code duplication in call setup and array mutation |
| Parser (`parse.rs`) | 7/10 | Clean AST; no error recovery; ~60 unnecessary clones |
| Stdlib (`stdlib/*.ae`) | 6/10 | Missing `flatten`, `sqrt`, `pow`; no method-style `.split()`/`.trim()` |
| Error handling | 8/10 | 16 typed RuntimeError variants; stack traces with filenames and line numbers |
| Concurrency design | 5/10 | Callback-based; no cancellation; no composable async primitives |
| **Overall** | **7.6/10** | Solid interpreter; clean pipeline; room to grow in stdlib and concurrency |

## License

MIT License - see [LICENSE](LICENSE) for details

## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
