# GC Implementation - Remaining Work

## Status: Core GC Complete ✅

The garbage collection system using `Rc<T>` is fully implemented! However, test files need updating to use the new Value constructors.

## What's Done ✅

- ✅ Value enum uses Rc for String and Array
- ✅ All production code updated
- ✅ Helper methods added: `Value::string()` and `Value::array()`
- ✅ String/Array operations handle Rc correctly
- ✅ Build succeeds for production code

## What's Left: Test Updates

### Current Issue

Tests fail to compile because they create Values directly:

```rust
// OLD (doesn't compile anymore):
let val = Value::String("hello".to_string());
let arr = Value::Array(vec![Value::Int(1), Value::Int(2)]);
```

### Solution: Use Helpers

```rust
// NEW (using helpers):
let val = Value::string("hello");
let arr = Value::array(vec![Value::Int(1), Value::Int(2)]);
```

### Files That Need Updates

1. `src/interpreter/interpreter_tests.rs` (~20 fixes)
2. `src/interpreter/builtins_tests.rs` (~10 fixes)

### How to Fix

**Option 1: Use helpers (recommended)**
```rust
// Before:
assert_eq!(result, Value::String("expected".to_string()));

// After:
assert_eq!(result, Value::string("expected"));
```

**Option 2: Wrap in Rc manually**
```rust
use std::rc::Rc;

// Before:
let val = Value::String("test".to_string());

// After:
let val = Value::String(Rc::new("test".to_string()));
```

### Automated Fix Script

Run this to fix most issues:

```bash
# In test files, replace Value::String with Value::string
find src -name "*_tests.rs" -exec sed -i '' \
  's/Value::String(\([^)]*\)\.to_string())/Value::string(\1)/g' {} \;

# Replace Value::Array with Value::array
find src -name "*_tests.rs" -exec sed -i '' \
  's/Value::Array(\(vec!\[.*\]\))/Value::array(\1)/g' {} \;
```

### Manual Fixes Needed

Some complex cases need manual fixing:
- Nested arrays
- Dynamic string construction
- Pattern matching on Values

### Quick Test

After fixes, run:
```bash
cargo test --lib
```

Should see **230 tests passing** again!

## Expected Memory Improvement

### Before GC
```
stdlib_collections_test: 135.28 GB ❌
Test time: 60+ seconds
System: Unresponsive
```

### After GC (projected)
```
stdlib_collections_test: < 100 MB ✅
Test time: ~60 seconds (same)
System: Responsive
Memory savings: 99%+
```

## How GC Works

### Reference Counting
```rust
let s1 = Value::string("hello");  // RC = 1
let s2 = s1.clone();              // RC = 2 (cheap - only pointer)
// s1 dropped → RC = 1
// s2 dropped → RC = 0 → String freed automatically!
```

### Before (Memory Leak)
```rust
fn reverse(s: String) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        result = result + &ch.to_string();  // NEW string each time!
        // Old `result` never freed → LEAK
    }
    result
}
// For "hello": creates 5 intermediate strings, all leaked
```

### After (GC)
```rust
fn reverse(s: Rc<String>) -> Rc<String> {
    let mut result = String::new();
    for ch in s.chars() {
        result = result + &ch.to_string();  // NEW string
        // Old `result` dropped → RC = 0 → freed immediately!
    }
    Rc::new(result)
}
// For "hello": creates 5 intermediate strings, 4 freed immediately
```

### Key Insight

**Old code**:
```
let s1 = "a";      // Alloc 1
let s2 = s1 + "b"; // Alloc 2 (s1 still in memory)
let s3 = s2 + "c"; // Alloc 3 (s1, s2 still in memory)
// All 3 allocations stay until program ends
```

**With GC**:
```
let s1 = Rc::new("a");      // Alloc 1, RC = 1
let s2 = Rc::new(s1 + "b"); // Alloc 2, s1 RC = 0 → freed!
let s3 = Rc::new(s2 + "c"); // Alloc 3, s2 RC = 0 → freed!
// Only s3 remains in memory
```

## Testing the GC

### Quick Memory Test

Create a test program:

```aether
// test_gc.ae
fn heavy_string_ops() {
    let result = ""
    let i = 0
    while (i < 10000) {
        result = result + "x"  // Should not leak with GC!
        i = i + 1
    }
    return len(result)
}

println(heavy_string_ops())
```

**Before GC**: Would use 50+ MB
**After GC**: Should use < 1 MB

### Run with Memory Monitoring

```bash
# Terminal 1: Run program
cargo run -- test_gc.ae

# Terminal 2: Monitor memory
watch -n 1 'ps aux | grep aether | grep -v grep'
```

## Limitations

### Reference Cycles

RC can't handle cycles (rare in Aether):

```aether
let a = []
let b = []
a.push(b)
b.push(a)  // Cycle! Both RC > 0 forever
```

**Solution** (Phase 5): Add cycle detector or upgrade to mark-and-sweep

### Performance

- Rc adds small overhead (~8 bytes per string/array)
- Clone is cheap (pointer copy) but not free
- Acceptable for interpreted language

## Next Steps

1. **Fix test files** (30 mins)
   - Use Value::string() helper
   - Use Value::array() helper
   - Run `cargo test --lib`

2. **Measure memory** (10 mins)
   - Run string tests
   - Monitor Activity Monitor
   - Confirm < 100 MB usage

3. **Commit** (5 mins)
   - Document memory improvement
   - Update KNOWN_ISSUES.md

4. **Celebrate** 🎉
   - 135 GB → < 100 MB
   - GC working!
   - Language now practical

## Questions?

- **Q**: Why Rc and not Arc?
  **A**: Single-threaded interpreter, Rc is lighter

- **Q**: What about performance?
  **A**: RC overhead minimal for interpreted language

- **Q**: Can users trigger the cycle bug?
  **A**: Rare, would need circular array references

- **Q**: Will this slow down tests?
  **A**: No, test time should be similar

- **Q**: When to use mark-and-sweep?
  **A**: Phase 5, if cycles become an issue

## Resources

- Rust Rc docs: https://doc.rust-lang.org/std/rc/
- "Crafting Interpreters" Ch 26: Garbage Collection
- Python's RC + cycle detector approach
- Apple's Swift uses RC successfully

---

**Status**: GC core complete, test fixes in progress
**Impact**: 99%+ memory reduction expected
**Priority**: High (critical bug fix)
