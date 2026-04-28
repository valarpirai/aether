---
layout: default
title: Home
---

# Aether Programming Language

A general-purpose, dynamically typed programming language with automatic memory management, combining the familiarity of C-like syntax with the ease of use of modern interpreted languages.

## Features

- **Dynamic Typing** - Runtime type checking for flexibility
- **Automatic Memory Management** - Reference-counted garbage collection
- **First-Class Functions** - Functions with closures and recursion
- **User-Defined Types** - Structs with fields and methods
- **Error Handling** - Structured `try/catch/throw` exception handling
- **Module System** - Import and organize code across files
- **Rich Standard Library** - 35+ functions written in Aether itself
- **Interactive REPL** - Rapid development and testing

## Quick Example

```aether
// Functional programming with stdlib
fn square(x) { return x * x }
fn is_even(x) { return x % 2 == 0 }

fn main() {
    let numbers = range(1, 11)
    let squares = map(numbers, square)
    let even_squares = filter(squares, is_even)
    let total = sum(even_squares)
    
    println("Sum of even squares:", total)  // 220
}
```

## Status

- **Phase**: 5 Sprint 2 Complete ✅
- **Tests**: 505 passing (99 unit + 406 integration)
- **Code Quality**: 0 clippy warnings
- **Documentation**: 15 comprehensive guides

## Quick Start

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/aether.git
cd aether

# Build the project
cargo build --release

# Run the REPL
./target/release/aether

# Run a program
./target/release/aether examples/hello.ae
```

### Your First Program

Create `hello.ae`:

```aether
fn main() {
    println("Hello, Aether!")
    
    let name = "World"
    println("Hello, ${name}!")
}
```

Run it:

```bash
cargo run hello.ae
```

## Documentation

### Getting Started
- [Language Design](DESIGN.html) - Complete language specification
- [Architecture](ARCHITECTURE.html) - System design and roadmap
- [Development Guide](DEVELOPMENT.html) - Contributing and best practices
- [Testing Guide](TESTING.html) - Test-driven development workflow

### Core Implementation
- [Lexer](LEXER.html) - Tokenization (14 tests)
- [Parser](PARSER.html) - Recursive descent parsing (53 tests)
- [Interpreter](INTERPRETER.html) - Tree-walking interpreter (82 tests)
- [REPL](REPL.html) - Interactive mode
- [Standard Library](STDLIB.html) - Self-hosted stdlib
- [Garbage Collection](GC_DESIGN.html) - Rc-based memory management
- [Module System](MODULE_SYSTEM.html) - Import mechanism

### Language Features
- [Structs](STRUCT.html) - User-defined types with methods
- [Error Handling](ERROR_HANDLING.html) - Try/catch/throw
- [String Features](STRING_FEATURES.html) - Indexing, interpolation, slicing
- [JSON Support](JSON.html) - Parsing and serialization
- [Time Functions](TIME.html) - clock(), sleep()
- [HTTP Client](HTTP.html) - http_get(), http_post()

## What Works

✅ **Complete Interpreter** - Tree-walking interpreter with full language support  
✅ **505 Tests Passing** - 100% success rate  
✅ **Garbage Collection** - Reference-counted memory management  
✅ **Standard Library** - 35+ functions written in Aether  
✅ **Built-in Functions** - I/O, type introspection, conversions, JSON, time, HTTP  
✅ **Collection Types** - Arrays, dicts, and sets with comprehensive methods  
✅ **Structs** - User-defined types with fields and methods  
✅ **Error Handling** - Try/catch/throw exception handling  
✅ **Module System** - Import and organize code  
✅ **Interactive REPL** - Line editing with history

## Examples

Browse [example programs](https://github.com/yourusername/aether/tree/main/examples):

- `hello.ae` - Hello World
- `set_demo.ae` - Set operations
- `dict_demo.ae` - Dictionary usage
- `error_handling.ae` - Exception handling
- `shapes.ae` - Structs and methods
- `http_demo.ae` - HTTP requests
- `stdlib_demo.ae` - Standard library showcase

## Community

- **GitHub**: [github.com/yourusername/aether](https://github.com/yourusername/aether)
- **Issues**: [Report bugs and request features](https://github.com/yourusername/aether/issues)
- **Discussions**: [Join the conversation](https://github.com/yourusername/aether/discussions)

## License

MIT License - see [LICENSE](https://github.com/yourusername/aether/blob/main/LICENSE) for details

## Development Status

**Current Phase**: Phase 5 Sprint 2 Complete

### Recent Achievements
- ✅ User-defined structs with methods
- ✅ Set type with set operations
- ✅ Dict methods (keys, values, contains)
- ✅ Comprehensive documentation (15 guides)
- ✅ 505 tests passing
- ✅ 0 clippy warnings

### Up Next
- Iterator protocol
- Async/await support
- Performance optimizations
- Community building

---

**Last Updated**: 2026-04-28  
**Version**: 0.1.0  
**Status**: Active Development

[Get Started](DESIGN.html){: .btn .btn-primary}
[View on GitHub](https://github.com/yourusername/aether){: .btn .btn-outline}
