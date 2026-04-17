# Aether REPL Documentation

## Overview

Interactive Read-Eval-Print Loop for executing Aether code.

**Location**: `src/repl.rs` and `src/main.rs`
**Status**: ✅ Complete

## Usage

### Start REPL
```bash
aether
```

### Run File
```bash
aether examples/hello.ae
```

## Features

- **Line editing** with rustyline (arrow keys, Ctrl+A/E, etc.)
- **Command history** (up/down arrows)
- **Multi-line support** (TODO)
- **Special commands** starting with `_`

## Special Commands

| Command | Description |
|---------|-------------|
| `_help` | Show help message |
| `_env` | Display current environment (variables) |
| `_exit` | Exit REPL |
| `_tokens` | Show tokens (TODO) |
| `_ast` | Show AST (TODO) |

## Controls

- **Ctrl+D** - Exit REPL
- **Ctrl+C** - Cancel current input
- **Up/Down** - Navigate history
- **Ctrl+A** - Start of line
- **Ctrl+E** - End of line

## Examples

```
>> let x = 42
>> x + 10
52
>> fn double(n) { return n * 2 }
>> double(21)
42
>> _env
Current environment: ...
>> _exit
Goodbye!
```

## Implementation

```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>>
```

Uses `rustyline::DefaultEditor` for line editing and maintains a persistent `Evaluator` across inputs.

---

**Last Updated**: April 17, 2026
**Status**: Fully functional with history support
