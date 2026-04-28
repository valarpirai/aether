# Aether Interpreter Documentation

## Overview

Tree-walking interpreter that executes AST nodes.

**Location**: `src/interpreter/`
**Status**: ✅ Complete (99 unit tests, 234 integration tests passing)

## Components

- `value.rs` - Runtime value types
- `environment.rs` - Variable scoping
- `evaluator.rs` - Expression evaluation & statement execution
- `interpreter_tests.rs` - 82 tests (80 passing, 2 ignored)

## Value Types

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Function { params: Vec<String> },
}
```

### Methods
- `is_truthy()` - For conditionals
- `type_name()` - For error messages

### GC Implementation Notes

Strings and Arrays use `Rc<T>` (Reference Counted pointers) for automatic memory management:

- **Creating Values**: Use helper methods
  - `Value::string(s)` - Create Rc-wrapped string
  - `Value::array(vec)` - Create Rc-wrapped array
- **Accessing Values**: Dereference with `.as_ref()` or pattern matching
  ```rust
  match &value {
      Value::String(s) => { let str_ref: &str = s.as_ref(); }
      Value::Array(arr) => { let vec_ref: &Vec<Value> = arr.as_ref(); }
  }
  ```
- **Cloning**: Cheap operation (only clones Rc pointer, not the actual data)
- **Memory Reclaim**: Automatic when reference count reaches zero

**See**: [GC_DESIGN.md](GC_DESIGN.md) for detailed implementation

## Environment

```rust
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}
```

### Operations
- `define(name, value)` - Create variable
- `get(name)` - Lookup variable (searches parent scopes)
- `set(name, value)` - Update variable
- `with_parent(env)` - Create nested scope

## Evaluator

```rust
pub struct Evaluator {
    pub environment: Environment,
}
```

### Main Methods
- `eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError>`
- `exec_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError>`
- `execute_program(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError>`

## Implemented Features

### ✅ Expressions
- All literals (int, float, string, bool, null)
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `<`, `>`, `<=`, `>=`, `==`, `!=`
- Logical: `&&`, `||`, `!`
- Unary: `-`, `!`
- String concatenation
- Array literals
- Array indexing
- Variables

### ✅ Statements
- Let declarations
- Assignments (simple & compound)
- Blocks
- If/else
- While loops
- For loops
- Break and continue
- Expression statements
- Function declarations
- Function calls with closures
- Return statements
- Member access (obj.property)

## Runtime Errors

```rust
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeError { expected: String, got: String },
    DivisionByZero,
    IndexOutOfBounds { index: i64, length: usize },
    InvalidOperation(String),
    ArityMismatch { expected: usize, got: usize },
}
```

## Examples

### Expression Evaluation
```rust
let mut eval = Evaluator::new();
let expr = Expr::Binary(
    Box::new(Expr::Integer(10)),
    BinaryOp::Add,
    Box::new(Expr::Integer(20))
);
let result = eval.eval_expr(&expr)?; // Value::Int(30)
```

### Variable Usage
```rust
eval.environment.define("x".to_string(), Value::Int(42));
let expr = Expr::Identifier("x".to_string());
let result = eval.eval_expr(&expr)?; // Value::Int(42)
```

### Statement Execution
```rust
let stmt = Stmt::Let("x".to_string(), Expr::Integer(42));
eval.exec_stmt(&stmt)?;
assert_eq!(eval.environment.get("x")?, Value::Int(42));
```

## Usage

```rust
use aether::interpreter::Evaluator;

let mut eval = Evaluator::new();
eval.execute_program(&program.statements)?;
```

---

**Last Updated**: April 17, 2026
**Phase**: 5 Complete (base)
**Status**: 99 unit tests, 234 integration tests passing
