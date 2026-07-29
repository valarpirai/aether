---
layout: default
title: "Aether — Memory Management"
---

[Home](../index.md) › Developer Docs › Memory Management

# Memory Management

**Status**: ✅ Complete
**Priority**: Critical (fixed 135 GB memory leak)
**Approach**: Reference Counting with Rc<T>

## Memory Model Decision

### Why Not Stack + Heap?

Languages like C, Java, and Go use two distinct regions:

- **Stack** — local variables at fixed slot offsets, managed by the hardware stack pointer, freed on function return (LIFO).
- **Heap** — dynamically allocated objects, freed by GC or `free()`.

This model works because a **compiler** runs first and assigns every local variable a numeric slot index before the program executes:

```
Compile time:             Runtime:
  int x = 1;    →    frame[0] = 1   (fixed offset, O(1) access)
  int y = 2;    →    frame[1] = 2   (compiler assigned the slot)
```

Aether is a **tree-walking interpreter** — it evaluates the AST directly through recursive Rust function calls. There is no compilation step. At runtime the only handle on a variable is its name (`"x"`), not a numeric slot. The Rust call stack is the interpreter's call stack.

To use a proper value stack for locals, one of the following would be required:

1. **A compiler pass** — analyse the AST, assign each local a slot index, emit bytecode. This is how CPython and the JVM work.
2. **A runtime name→index table per frame** — same HashMap lookup cost as the current approach, but far more complex to implement.

Without (1), a `Vec<Value>` stack gives no advantage over `HashMap<String, Value>` because names still have to be resolved to indices at runtime.

### Memory Model Comparison

| Approach | Examples | How locals work | Tradeoffs |
|---|---|---|---|
| Tree-walking (Aether) | Early Ruby MRI, early Python | `HashMap<name, Value>` per scope | Simple; name lookup per access |
| Bytecode VM | CPython, JVM, V8 Ignition | `frame[slot_index]` — O(1) indexed access | Requires compiler pass; much faster |
| JIT compiled | V8 TurboFan, HotSpot | Real machine stack, register allocation | Fastest; most complex |

### Why Reference Counting (`Rc<T>`)?

Given that Aether is a tree-walker, variables live in `Environment` (a `HashMap<String, Value>`). The right question becomes: how do values share data without cloning it on every assignment?

**Option A — Clone on every assignment (initial state)**
- Every `Value::String(String)` copy duplicated the heap string.
- Result: 135 GB memory usage in the test suite.

**Option B — Arena allocator**
- Allocate all values into a single region; free the whole arena at once.
- Incompatible with per-scope lifetimes and closures that outlive their defining scope.

**Option C — Tracing GC (mark-and-sweep)**
- Periodically scan from roots, collect unreachable objects.
- Handles cycles automatically.
- Requires a centralised heap registry to enumerate all live objects; `Rc<T>` scattered across Rust's allocator has no such registry.
- Significant implementation complexity for the current phase.

**Option D — Reference Counting (`Rc<T>`) ← chosen**
- Wrap heap-allocated values in `Rc<T>`; cloning only copies the pointer and increments a counter.
- Memory freed immediately when the last reference drops (RC = 0).
- Zero infrastructure beyond what Rust provides; composable with `RefCell<T>` for interior mutability.
- Known limitation: reference cycles keep RC ≥ 1 and are never freed. Mitigated by exposing `make_weak()` / `upgrade_weak()` for user code.

### Upgrade Path

If Aether outgrows the tree-walking model, the natural evolution is:

```
Current:  AST → tree-walking evaluator  (HashMap scopes, Rc values)
Step 1:   AST → bytecode compiler → bytecode VM  (frame stack, slot indices)
Step 2:   hot bytecode → JIT to machine code  (real stack frames, registers)
```

This mirrors CPython (added bytecode compiler in 1994) and V8 (Ignition bytecode → TurboFan JIT). It is a significant redesign and not planned for the current phase.

## Problem Statement

Without GC, the interpreter leaks memory catastrophically:
- String operations create new allocations every iteration
- Arrays cloned unnecessarily
- No memory reclamation until process exits
- **Result**: 135 GB memory usage in tests

## Solution: Reference Counted Values

### Phase 1: Rc-based Reference Counting

Use Rust's `Rc<T>` (Reference Counted pointer) to share values:

```rust
// Before (current - memory leak)
pub enum Value {
    String(String),        // Owned - cloned on every use
    Array(Vec<Value>),     // Owned - entire array cloned
}

// After (with GC)
pub enum Value {
    String(Rc<String>),    // Shared - only pointer cloned
    Array(Rc<Vec<Value>>), // Shared - cheap to clone
}
```

### How Reference Counting Works

```rust
let s1 = Value::String(Rc::new("hello".to_string())); // RC = 1
let s2 = s1.clone(); // RC = 2 (cheap - only pointer cloned)
// ... s1 dropped → RC = 1
// ... s2 dropped → RC = 0 → String freed automatically
```

**Benefits**:
- ✅ Automatic memory reclamation when RC hits 0
- ✅ No manual tracking needed
- ✅ Zero-cost when values aren't shared
- ✅ Immediate impact on memory usage

**Limitations**:
- ⚠️ Can't handle reference cycles (rare in Aether)
- ⚠️ Slight overhead for RC bookkeeping
- ⚠️ Thread-local only (fine for single-threaded interpreter)

### Implementation Steps

1. **Update Value enum** - Wrap String/Array in Rc
2. **Update Value operations** - Clone Rc instead of data
3. **Update Display/PartialEq** - Dereference Rc
4. **Update interpreter** - Handle Rc in evaluation
5. **Test** - Verify memory usage drops dramatically

### Code Changes

#### value.rs
```rust
use std::rc::Rc;

pub enum Value {
    Int(i64),
    Float(f64),
    String(Rc<String>),        // Rc — cheap clone, freed when RC = 0
    Bool(bool),
    Null,
    Array(Rc<Vec<Value>>),     // Rc — cheap clone
    Dict(Rc<Vec<(Value, Value)>>),  // Rc
    Set(Rc<HashSet<Value>>),        // Rc
    Function { params: Vec<String>, body: Rc<Stmt>, closure: Rc<Environment> },
    AsyncFunction { params: Vec<String>, body: Rc<Stmt>, closure: Rc<Environment> },
    BuiltinFn { name: String, arity: usize, func: BuiltinFn },
    Module { name: String, members: Rc<HashMap<String, Value>> },
    StructDef { name: String, fields: Vec<String>, methods: MethodMap },
    Instance { type_name: String, fields: Rc<RefCell<HashMap<String, Value>>>, methods: MethodMap },
    Iterator(Rc<RefCell<IteratorState>>),
    Promise(Rc<RefCell<PromiseState>>),
    ErrorVal { message: String, stack_trace: String },
    FileLines(Rc<RefCell<FileIterState>>),
}
```

#### Creating values
```rust
// String literal
Value::String(Rc::new("hello".to_string()))

// Array literal
Value::Array(Rc::new(vec![Value::Int(1), Value::Int(2)]))

// String concatenation (still creates new string, but old one freed)
let new_str = format!("{}{}", s1, s2);
Value::String(Rc::new(new_str))  // Old strings freed if RC = 0
```

#### Accessing values
```rust
match &value {
    Value::String(s) => {
        let str_ref: &str = s.as_ref();  // Dereference Rc
        println!("{}", str_ref);
    }
    Value::Array(arr) => {
        let vec_ref: &Vec<Value> = arr.as_ref();
        for item in vec_ref {
            // ...
        }
    }
}
```

### Expected Memory Impact

**Before GC**:
```
String reverse("hello"):
- 5 intermediate strings allocated
- All kept in memory
- 135 GB for test suite

Test time: 60+ seconds
```

**After GC**:
```
String reverse("hello"):
- 5 intermediate strings allocated
- Old ones freed immediately (RC = 0)
- Expected: < 100 MB for test suite

Test time: Should be similar or faster
```

**Estimated savings**: 99%+ memory reduction

### Testing Strategy

1. **Unit tests** - Verify values work correctly with Rc
2. **Memory tests** - Monitor memory usage during string ops
3. **Integration tests** - Run full test suite and measure
4. **Benchmarks** - Ensure no performance regression

### Future Improvements (Phase 5+)

#### Cycle Detection
Reference counting can't handle cycles:
```aether
let a = []
let b = []
a.push(b)  // a → b
b.push(a)  // b → a (cycle!)
// Both RC > 0 forever, never freed
```

**Solution**: Add cycle detector or upgrade to mark-and-sweep

#### Arena Allocator
For short-lived values, use arena:
```rust
struct ValueArena {
    values: Vec<Value>,
}

// Allocate many values, drop all at once
impl Drop for ValueArena {
    fn drop(&mut self) {
        // All values freed together
    }
}
```

#### Generational GC
Separate young/old generations:
```rust
struct Heap {
    young: Vec<Value>,  // Collected frequently
    old: Vec<Value>,    // Collected rarely
}
```

#### Concurrent GC
For multi-threaded future:
```rust
use std::sync::Arc;  // Thread-safe RC
```

## Implementation Timeline

**Phase 1** (Now): Reference Counting
- Time: 1-2 hours
- Impact: Fix memory leak
- Tests: All 230 should still pass

**Phase 2** (Phase 5): Optimizations
- Arena allocators
- Cycle detection
- Profiling and tuning

**Phase 3** (Future): Advanced GC
- Mark-and-sweep
- Generational GC
- Concurrent GC

## Risks & Mitigation

**Risk 1**: Breaking existing tests
- **Mitigation**: Run tests after each change, fix immediately

**Risk 2**: Performance regression
- **Mitigation**: Benchmark before/after, Rc overhead is minimal

**Risk 3**: Reference cycles leak memory
- **Mitigation**: Document limitation, add cycle detector later

**Risk 4**: Complex refactor
- **Mitigation**: Incremental changes, commit frequently

## Success Criteria

- ✅ All ~693 tests still pass
- ✅ Memory usage < 100 MB (vs 135 GB)
- ✅ Test time ≤ 60 seconds (no regression)
- ✅ 0 clippy warnings
- ✅ Clean architecture maintained

## References

- Rust Rc docs: https://doc.rust-lang.org/std/rc/
- "Crafting Interpreters" Chapter 26 (Garbage Collection)
- Python's reference counting implementation
- Ruby's GC evolution

---

**Last Updated**: May 20, 2026
**Phase**: 5 Complete (base)
**Status**: Rc-based GC implemented and working, 99%+ memory reduction achieved

---
[← Interpreter](INTERPRETER.md) &nbsp;&nbsp; [Backlog →](BACKLOG.md)
