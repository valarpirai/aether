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
