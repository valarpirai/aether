# Aether REPL

Interactive Read-Eval-Print Loop for the Aether interpreter.

**Source**: `src/main/java/com/aether/Repl.java`
**Line editing**: JLine 3 (replaces Rust's rustyline)

---

## Usage

```bash
# Start REPL
java --enable-preview -jar target/aether.jar

# Run a file
java --enable-preview -jar target/aether.jar examples/hello.ae
```

---

## Features

- Line editing — arrow keys, Ctrl+A/E, Ctrl+W
- Command history (up/down arrows, 500 entries)
- State persists across inputs — variables defined on one line survive to the next
- Non-null expression results are printed automatically
- Errors are reported and the REPL continues (no crash)

---

## Built-in Commands

| Command | Description |
|---------|-------------|
| `help`  | Show available commands |
| `env`   | List all defined variables and their values |
| Ctrl+D  | Exit |
| Ctrl+C  | Cancel current input line |

---

## Session Example

```
>> let name = "Aether"
>> "Hello ${name}!"
Hello Aether!
>> fn square(n) { return n * n }
>> square(9)
81
>> let nums = [1, 2, 3, 4, 5]
>> nums.length
5
>> env
name = Aether
square = <fn ["n"]>
nums = [1, 2, 3, 4, 5]
>> help
Commands:
  help  — show this message
  env   — list all defined variables
  Ctrl+D — exit
```

---

## File Execution

When given a file argument, the interpreter:

1. Parses and executes all top-level statements
2. If a `main` function is defined, calls it automatically
3. Script-style files (no `main`) work as-is

```aether
fn main() {
    println("Hello from Aether!")
}
```

```bash
java --enable-preview -jar target/aether.jar hello.ae
# Hello from Aether!
```

---

**Last Updated**: April 17, 2026
**Status**: Complete
