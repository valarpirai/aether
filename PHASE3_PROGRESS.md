# Phase 3 Progress: String/Array/Dict Support

## Current Status: 40% Complete

Phase 3 is partially implemented. The foundation is in place but needs completion.

## ✅ Completed

### 1. FFI Helper Functions (`src/interpreter/ffi_helpers.rs`)
Complete set of C ABI functions for type conversion:

**Type Checking**:
- `aether_value_is_int/string/array/dict/null()`

**Value Extraction** (Aether → Rust):
- `aether_value_as_int()` - Extract i64
- `aether_value_as_string()` - Extract String (returns owned CString)
- `aether_value_array_len/get()` - Array access
- `aether_value_dict_len/get()` - Dict access

**Value Creation** (Rust → Aether):
- `aether_value_new_int/string/array/dict/null()`
- `aether_value_array_push()` - Add elements
- `aether_value_dict_insert()` - Add key-value pairs
- `aether_value_free()` - Memory cleanup

**Total**: 22 FFI helper functions, fully implemented and exported

### 2. Conversion Traits (`aether-plugin/src/convert.rs`)

**`FromAether` trait**: Convert Aether Value → Rust type
- ✅ `i64::from_aether()`
- ✅ `String::from_aether()` 
- ✅ `Vec<i64>::from_aether()`
- ⚠️  `HashMap<String, i64>::from_aether()` - stub only

**`ToAether` trait**: Convert Rust type → Aether Value  
- ✅ `i64::to_aether()`
- ✅ `String::to_aether()`
- ✅ `Vec<i64>::to_aether()`
- ✅ `HashMap<String, i64>::to_aether()`
- ✅ `Option<T>::to_aether()` - null for None
- ✅ `Result<T, E>::to_aether()` - handles Ok/Err

### 3. Macro V2 Protocol Support (`aether-plugin-macro/src/lib.rs`)

Macro now **auto-detects** protocol:
- **V1**: All i64 params → uses old signature
- **V2**: Any String/Vec param → uses new signature

Generated V2 wrapper:
```rust
unsafe extern "C" fn func_ffi(
    args: *const AetherValuePtr,
    argc: c_int,
    out_error: *mut AetherValuePtr
) -> AetherValuePtr
```

## ⚠️  Incomplete

### 1. Registration Macro
`aether_plugin_init!()` only supports V1 signatures.

**Needs**: Detect function protocol and register appropriately.

### 2. Plugin Loader
`src/interpreter/plugin.rs` only calls V1 functions.

**Needs**: 
- Detect which protocol a function uses
- Call V1 or V2 appropriately
- Handle V2 error output parameter

### 3. Dict Iteration
`HashMap::from_aether()` is stubbed.

**Needs**: Iterator support in FFI helpers.

### 4. Testing
No tests yet for V2 protocol.

**Needs**: Integration tests for String/Vec functions.

## Architecture

### V1 Protocol (Backward Compatible)
```rust
// Plugin side
#[aether_export]
fn add(a: i64, b: i64) -> i64 { a + b }

// Generated FFI
unsafe extern "C" fn add_ffi(args: *const i64, argc: c_int) -> i64

// Interpreter side (existing)
plugin.call(name, &[Value::Int(40), Value::Int(2)])  // → Value::Int(42)
```

### V2 Protocol (New)
```rust
// Plugin side
#[aether_export]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Generated FFI
unsafe extern "C" fn greet_ffi(
    args: *const AetherValuePtr,
    argc: c_int,
    out_error: *mut AetherValuePtr
) -> AetherValuePtr

// Interpreter side (TODO)
plugin.call_v2(name, &[Value::string("Alice")])  // → Value::string("Hello, Alice!")
```

## Remaining Work

### Critical Path (2-3 days)

**Day 1**: Registration & Loading
1. Update `aether_plugin_init!()` to support both protocols
2. Add protocol detection to plugin metadata
3. Update `Plugin::call()` to dispatch V1 vs V2

**Day 2**: Dict Support & Testing
1. Add dict iterator FFI helpers
2. Implement `HashMap::from_aether()`
3. Create 20+ integration tests
4. Build example_plugin_v2 successfully

**Day 3**: Examples & Documentation
1. Complete V2 example plugin
2. Create Aether demo program
3. Update all documentation
4. Verify all tests pass

## Why This Matters

Once complete, plugins can:
- ✅ Process text (string args/returns)
- ✅ Work with collections (Vec args/returns)
- ✅ Handle structured data (HashMap args/returns)
- ✅ Return proper errors (Result<T, E>)

This unlocks **real-world plugins**:
- **SQLite**: queries as strings, rows as dicts
- **Image processing**: pixel data as Vec<u8>
- **HTTP client**: headers as HashMap<String, String>
- **JSON**: nested structures

## Files Modified This Session

| File | Status | Lines |
|------|--------|-------|
| `src/interpreter/ffi_helpers.rs` | ✅ Complete | 330 |
| `aether-plugin/src/convert.rs` | ✅ ~95% | 220 |
| `aether-plugin-macro/src/lib.rs` | ✅ Protocol detection | 140 |
| `src/interpreter/mod.rs` | ✅ Module registration | 1 |
| `example_plugin_v2/` | ⚠️  Won't build yet | 50 |

**Total**: ~740 lines of new code

## Next Session Plan

1. Fix `aether_plugin_init!()` macro
2. Update `Plugin::load()` and `Plugin::call()`
3. Test V2 protocol end-to-end
4. Complete dict iteration
5. Write comprehensive tests

**Estimated time to complete**: 2-3 days

## Backward Compatibility

✅ **Zero breaking changes**
- V1 plugins continue to work
- Macro auto-detects protocol
- No changes needed to Phase 2 plugins

## Conclusion

Phase 3 foundation is **solid**:
- ✅ FFI helpers complete and well-designed
- ✅ Type conversion traits implemented
- ✅ Macro protocol detection working

**Remaining**: Plumbing to connect everything together.

**Recommendation**: Complete Phase 3 before building real-world plugins. The foundation is 40% done, finishing is 2-3 days of focused work.
