---
layout: default
title: "Aether — REPL"
---

[Home](../index.html) › Language Reference › REPL

# REPL

The Aether REPL lets you run Aether code interactively, one expression or statement at a time.

## Starting the REPL

```bash
aether
```

## Running a file

```bash
aether examples/hello.ae
```

## CLI subcommands

| Command | What it does |
|---------|-------------|
| `aether ast <file>` | Print the AST as an indented tree |
| `aether ast --json <file>` | Print the AST as JSON |
| `aether ast --json --output out.json <file>` | Write AST JSON to a file |
| `aether fmt <file>` | Format an Aether source file in place |
| `aether fmt --check <file>` | Check formatting without writing; exits 1 if unformatted |
| `aether test [dir\|file]` | Discover and run `*_test.ae` files |
| `aether check <file>` | Check for undefined variables without running |

## Session example

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

## Multi-line input

The REPL automatically detects incomplete input and shows a `.. ` continuation prompt until the block is closed. Press **Ctrl+C** to cancel a multi-line block.

```
>> fn greet(name) {
..     return "Hello " + name
.. }
>> greet("Alice")
Hello Alice
>> if (x > 0) {
..     println("positive")
.. }
positive
```

Triple-quoted strings also continue across lines:

```
>> let msg = """
..     hello
..     world
.. """
```

## Special commands

| Command | What it does |
|---------|-------------|
| `_help` | Show available commands |
| `_env` | Display all variables in the current scope |
| `_exit` | Exit the REPL |

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Up / Down | Navigate command history |
| Ctrl+A | Move to start of line |
| Ctrl+E | Move to end of line |
| Ctrl+C | Cancel current input |
| Ctrl+D | Exit REPL |

## History

Command history is saved to `~/.aether_history` between sessions. If `HOME` is unset, history is not persisted.

---
[← Home](../index.html) &nbsp;&nbsp; [Strings →](STRINGS.html)
