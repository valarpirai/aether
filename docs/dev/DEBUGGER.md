# Debugger

The debugger pauses program execution at a `debugger` statement and drops into an interactive REPL where you can inspect variables, evaluate expressions, and step through code.

---

## Trigger

```aether
fn main() {
    let x = 10
    debugger        // execution pauses here
    let y = x + 5
    print(y)
}
```

When the interpreter reaches `debugger`, it prints source context and waits for commands.

---

## Source Context Display

On each pause the debugger shows 2 lines above and 2 lines below the current line:

```
[debugger] Paused at examples/foo.ae:4

   2: fn main() {
   3:     let x = 10
>  4:     debugger
   5:     let y = x + 5
   6:     print(y)

(dbg)
```

When the source file is not available (stdlib, REPL):

```
[debugger] Paused at <repl>:4
[source not available]

(dbg)
```

---

## Commands

| Command | What it does |
|---------|-------------|
| `c` / `continue` | Resume execution until the next `debugger` or end of program |
| `n` / `next` | Run the current line and pause at the next line in the same scope (step over) |
| `s` / `step` | Run the current line and pause at the next line anywhere, including inside called functions (step into) |
| `bt` / `backtrace` | Print the call stack |
| `vars` | Print all variables in the current scope |
| `q` / `quit` | Exit the program immediately |
| any expression | Evaluate the expression against the current scope and print the result |

---

## Step Mode Semantics

### `next` — step over

Pauses at the next `Stmt::Line` marker where call depth ≤ depth at pause time. A function call on the current line executes fully before the next pause.

### `step` — step into

Pauses at the very next `Stmt::Line` marker regardless of depth. If the current line calls a function, the debugger pauses at its first statement.

---

## Architecture

### DebugState

Tracks whether and how to pause at the next line marker.

```
enum StepMode {
    Running,          // no debugger active
    Paused,           // at a breakpoint, waiting for command
    Step,             // pause at the very next Stmt::Line
    Next(usize),      // pause at the next Stmt::Line where depth <= stored depth
}
```

### Stmt::Debugger

Sets `StepMode::Paused` and calls `trigger_debugger()`.

### Stmt::Line hook

Every `Stmt::Line(n)` already updates `current_line`. The debugger check is added there:

- `Step` → set `Paused`, call `trigger_debugger()`
- `Next(d)` and `current_depth <= d` → set `Paused`, call `trigger_debugger()`
- Otherwise → do nothing (normal execution continues)

### trigger_debugger()

Reads source context, prints it, then loops reading commands from stdin until the user enters `c`, `n`, `s`, or `q`.

---

## Implementation Files

| File | Change |
|------|--------|
| `src/lexer/token.rs` | Add `TokenKind::Debugger` |
| `src/lexer/scanner.rs` | Map `"debugger"` keyword |
| `src/parser/ast.rs` | Add `Stmt::Debugger` |
| `src/parser/parse.rs` | Parse `debugger` statement |
| `src/interpreter/evaluator/mod.rs` | Add `DebugState`, `trigger_debugger()`, source context reader |
| `src/interpreter/evaluator/statements.rs` | Handle `Stmt::Debugger`; add step check to `Stmt::Line` |
| `tests/debugger_test.rs` | Automated tests for non-interactive paths |
| `examples/debugger_demo.ae` | Demo program |
