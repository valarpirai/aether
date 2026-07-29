---
layout: default
title: "Aether — Interpreter"
---

[Home](../index.md) › Developer Docs › Interpreter

# Aether Interpreter Documentation

## Overview

Tree-walking interpreter that executes AST nodes directly without compilation.

**Location**: `src/interpreter/`  
**Status**: ✅ Complete (134 unit tests, ~559 integration tests passing)

## Components

```
src/interpreter/
  mod.rs             — public re-exports
  value.rs           — Value enum (all runtime types)
  environment.rs     — variable scoping + RuntimeError enum
  builtins.rs        — built-in functions (print, len, http_get, …)
  stdlib.rs          — embedded stdlib loader
  io_pool.rs         — I/O thread pool + HttpOptions
  event_loop.rs      — EventLoopQueue for callback-based async
  evaluator/
    mod.rs           — Evaluator struct, constructors, public API
    expressions.rs   — eval_expr, eval_index, eval_slice, await_value
    statements.rs    — exec_stmt_internal (all Stmt variants)
    functions.rs     — eval_call, call_value, exec_async_body, try_submit_io_task
    members.rs       — eval_member, eval_method_call (collections + structs)
    modules.rs       — load_module, import resolution
    operators.rs     — eval_unary, eval_binary, arithmetic, comparison
```

## Value Types

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    String(Rc<String>),     // Rc for cheap clone / GC
    Bool(bool),
    Null,
    Array(Rc<Vec<Value>>),  // Rc for cheap clone / GC
    Dict(Rc<IndexMap<Value, Value>>),  // insertion-ordered, O(1) lookup, key: string/int/bool
    Set(Rc<HashSet<Value>>),
    Function {
        params: Vec<String>,
        body: Rc<Stmt>,
        closure: Rc<Environment>,
    },
    AsyncFunction {
        params: Vec<String>,
        body: Rc<Stmt>,
        closure: Rc<Environment>,
    },
    BuiltinFn { name: String, arity: usize, func: BuiltinFn },
    Module { name: String, members: Rc<HashMap<String, Value>> },
    StructDef { name: String, fields: Vec<String>, methods: MethodMap },
    Instance {
        type_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: MethodMap,
    },
    Iterator(Rc<RefCell<IteratorState>>),
    Promise(Rc<RefCell<PromiseState>>),
    ErrorVal { message: String, stack_trace: String },
    FileLines(Rc<RefCell<FileIterState>>),
}
```

### GC: Rc-based reference counting

- `String`, `Array`, `Dict`, `Set` use `Rc<T>` — clone is O(1), data freed when count drops to zero.
- `Dict` uses `IndexMap<Value, Value>`: O(1) average lookup and insert, insertion-order preserved.
- Mutable aggregate values (`Instance` fields, `Iterator` state, `Promise` state) use `Rc<RefCell<T>>`.
- Helper constructors: `Value::string(s)`, `Value::array(v)`, `Value::dict(pairs)`, `Value::set(h)`, `Value::promise(func, args)`, `Value::promise_io(rx)`.

See [MEMORY_MANAGEMENT.md](MEMORY_MANAGEMENT.md) for details.

## Value Construction Helpers

Never construct `Value` variants directly — these helpers set up the `Rc`
wrapping that the GC depends on.

| Helper | Purpose |
|--------|---------|
| `Value::string(s)` | Rc-wrapped string |
| `Value::array(vec)` | `Rc<RefCell>`-wrapped array (reference semantics) |
| `Value::dict(vec)` | `Rc<RefCell>`-wrapped dict (reference semantics) |
| `Value::set(hashset)` | Rc-wrapped set |
| `Value::promise(func, args)` | pending Promise |
| `Value::promise_io(rx)` | channel-backed I/O Promise |
| `Value::error_val(msg, stack, line)` | error object for catch blocks |

Predicates: `Value::is_truthy()` for boolean coercion in conditionals,
`Value::is_hashable()` to check set/dict key eligibility.

See [MEMORY_MANAGEMENT.md](MEMORY_MANAGEMENT.md) for the `Rc` cycle rules.

## Evaluator Struct

```rust
pub struct Evaluator {
    pub environment: Environment,
    call_depth: usize,
    max_call_depth: usize,           // from AETHER_CALL_DEPTH (default 100)
    module_cache: HashMap<String, Environment>,
    loading_stack: Vec<String>,      // circular import detection
    pub current_file: Option<PathBuf>,
    pub(crate) io_pool: Option<Arc<IoPool>>,
    pub(crate) event_loop_queue: EventLoopQueue,
    pub(crate) event_loop_timeout: Option<f64>,  // from AETHER_EVENT_LOOP_TIMEOUT
    pub current_line: usize,
    pub(crate) call_stack: Vec<StackFrame>,      // for stack traces
}
```

### Constructors

| Constructor | Use case |
|-------------|----------|
| `Evaluator::new()` | Full interpreter with stdlib |
| `Evaluator::new_without_stdlib()` | Fast test initialization (~760× faster) |

### Main Methods

- `eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError>`
- `exec_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError>`
- `call_main(&mut self) -> Result<(), RuntimeError>` — calls the `main()` function

## Environment

```rust
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}
```

**Operations**: `define(name, value)`, `get(name)`, `set(name, value)`, `with_parent(env)`

Block-scoped with lexical scope chain. Function closures capture `Rc<Environment>`.

## Runtime Errors

```rust
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeError { expected: String, got: String },
    DivisionByZero,
    IndexOutOfBounds { index: i64, length: usize },
    InvalidOperation(String),
    ArityMismatch { expected: usize, got: usize },
    StackOverflow { depth: usize, limit: usize },
    Thrown(String),  // user throw — caught by TryCatch
}
```

## Implemented Features

### Expressions
- All literals: int, float, string (with interpolation `${expr}`), bool, null
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `<`, `>`, `<=`, `>=`, `==`, `!=`
- Logical: `&&`, `||`, `!`
- Null coalescing: `??` (short-circuit)
- Optional chaining: `?.member`, `?.method(args)`
- Unary: `-`, `!`
- Array literals, indexing, slicing `arr[1:3]`, spread `[...arr]`
- Dict literals, indexing
- Set literals via `set([...])`
- Member access, method calls
- Function expressions: `fn(params) { body }`
- Async function expressions: `async fn(params) { body }`
- `await expr`
- String interpolation: `"Hello ${name}"`
- Struct instantiation: `Point { x: 1, y: 2 }`
- Optional member/call: `obj?.field`, `obj?.method(args)`

### Statements
- `let` declarations, assignment, compound assignment (`+=`, `-=`, …)
- `if`/`else`
- `while`, `for ... in`
- `break`, `continue` (with optional labels)
- `return`
- `fn` declarations, `async fn` declarations
- `struct` declarations
- `try`/`catch`/`finally`, `throw`
- `import`, `from ... import`, `import ... as`, `from ... import ... as`
- Labeled loops: `outer: while ...`

## Usage

```rust
use aether::interpreter::Evaluator;

let mut eval = Evaluator::new();
eval.execute_program(&program.statements)?;
eval.call_main()?;
```

For tests (no stdlib overhead):

```rust
let mut eval = Evaluator::new_without_stdlib();
```

---

**Last Updated**: 2026-04-29  
**Status**: Complete

---
[← Parser](PARSER.md) &nbsp;&nbsp; [Memory Management →](MEMORY_MANAGEMENT.md)
