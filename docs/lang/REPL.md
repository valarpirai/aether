---
layout: default
title: "Aether — REPL"
---

[Home](../index.md) › Language Reference › REPL

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
| `aether check [file\|dir]` | Check for undefined variables without running (default: current dir) |

## Entry point

Every Aether program file must define a `fn main()` function. Running a file without one is an error:

```bash
$ aether script.ae
Error: script.ae: no main() function defined. Every Aether program must have a main() function.
```

`aether check` reports a **warning** (not an error) when `main()` is absent:

```bash
$ aether check script.ae
script.ae:0: warning: no main() function defined
```

When checking a directory, exactly one file across the whole directory should define `main()`:

```bash
$ aether check src/
src/lib.ae: ok
src/main.ae: ok          # ← the one entry point
```

If zero or multiple files define `main()`, a warning is printed.

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
[← Home](../index.md) &nbsp;&nbsp; [Strings →](STRINGS.md)
