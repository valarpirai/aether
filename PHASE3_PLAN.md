# Phase 3: String/Array/Dict Support - Implementation Plan

## Goal
Enable plugins to accept and return complex types: `String`, `Vec<T>`, `HashMap<String, T>`, and `Result<T, E>`.

## Current State (Phase 2)
- ✅ Integer-only: `fn(i64, i64) -> i64`
- ✅ Auto-generated FFI wrappers
- ✅ Proc macro eliminates boilerplate

## Target State (Phase 3)
```rust
#[aether_export]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[aether_export]
fn sort_numbers(nums: Vec<i64>) -> Vec<i64> {
    let mut sorted = nums;
    sorted.sort();
    sorted
}

#[aether_export]
fn lookup(dict: HashMap<String, i64>, key: String) -> Result<i64, String> {
    dict.get(&key).copied().ok_or("key not found".to_string())
}
```

## Architecture Changes

### 1. FFI Protocol V2

**Current (V1 - Integer only)**:
```rust
unsafe extern "C" fn func(args: *const i64, argc: c_int) -> i64
```

**New (V2 - Complex types)**:
```rust
// Opaque pointer to Aether Value
type AetherValuePtr = *const std::ffi::c_void;

unsafe extern "C" fn func(
    args: *const AetherValuePtr,
    argc: c_int,
    out_error: *mut AetherValuePtr
) -> AetherValuePtr
```

### 2. Type Conversion Layer

Create `aether-plugin/src/convert.rs`:

```rust
pub trait FromAether: Sized {
    unsafe fn from_aether(ptr: AetherValuePtr) -> Result<Self, String>;
}

pub trait ToAether {
    unsafe fn to_aether(self) -> AetherValuePtr;
}

// Implementations for:
// - i64, f64, bool
// - String
// - Vec<T> where T: FromAether
// - HashMap<String, T> where T: FromAether
// - Option<T>
// - Result<T, E>
```

### 3. Macro Updates

Extend `#[aether_export]` to:
1. Detect parameter types
2. Generate appropriate `from_aether()` calls
3. Generate appropriate `to_aether()` call for return
4. Handle `Result<T, E>` → error output parameter

## Implementation Steps

### Step 1: Add FFI Helpers (1 day)
- [ ] Add `AetherValuePtr` type alias
- [ ] Add `from_aether_*` helper functions in Rust interpreter
- [ ] Add `to_aether_*` helper functions in Rust interpreter
- [ ] Expose these as `extern "C"` functions for plugins

### Step 2: Update Plugin API (1 day)
- [ ] Add `convert.rs` module to `aether-plugin`
- [ ] Implement `FromAether` trait for basic types
- [ ] Implement `ToAether` trait for basic types
- [ ] Add helper types (AetherString, AetherVec, etc.)

### Step 3: Extend Macro (1 day)
- [ ] Parse parameter types beyond `i64`
- [ ] Generate conversion code for each parameter
- [ ] Generate conversion code for return type
- [ ] Handle `Result<T, E>` error output

### Step 4: Update Examples & Tests (1 day)
- [ ] Create `example_plugin_v2/` with string/array/dict functions
- [ ] Add 20+ integration tests
- [ ] Update documentation

## Challenges & Solutions

### Challenge 1: Memory Management
**Problem**: Who owns the returned `Value`?  
**Solution**: Plugin creates boxed `Value`, transfers ownership to Aether. Aether wraps in `Rc`.

### Challenge 2: Lifetimes
**Problem**: `&str` vs `String` - plugins can't return borrowed data  
**Solution**: Always use owned types (`String`, `Vec`, `HashMap`)

### Challenge 3: Error Handling
**Problem**: How to propagate Rust `Result<T, E>` to Aether exceptions?  
**Solution**: Use output parameter for error, return null on error

## Breaking Changes

**None!** V1 protocol (integer-only) continues to work. Plugins opt into V2 by:
1. Using `String`/`Vec`/`HashMap` types
2. Macro detects and generates V2 code

## Timeline

- Day 1: FFI helpers + conversion infrastructure
- Day 2: Trait implementations + macro updates
- Day 3: Examples + tests
- Day 4: Documentation + polish

**Total**: 3-4 days

## Success Criteria

- [ ] String args/returns work
- [ ] Vec args/returns work
- [ ] HashMap args/returns work
- [ ] Result<T, E> → exceptions work
- [ ] All existing tests pass
- [ ] 20+ new tests for complex types
- [ ] Example plugin demonstrates all features
- [ ] Zero breaking changes to Phase 2 plugins

## Next Steps After Phase 3

With Phase 3 complete, we can build real-world plugins:
- SQLite (strings for queries, dicts for rows)
- Image processing (byte arrays)
- HTTP client (headers as dicts)
- JSON processing (nested dicts/arrays)
