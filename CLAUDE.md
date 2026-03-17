# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aether is a general-purpose programming language implementation written in Rust. The project is currently in initial setup phase.

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

This is a new repository. When implementing:
1. Initialize with `cargo init` if not already done
2. Consider typical language implementation components: lexer, parser, AST, interpreter/compiler
3. Follow Rust best practices and idioms