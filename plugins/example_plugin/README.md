# Example Aether Plugin

This is a minimal example plugin demonstrating Aether's FFI system.

## What it provides

Four simple math functions:

- `add(a, b)` → a + b
- `multiply(a, b)` → a * b
- `power(base, exp)` → base^exp
- `is_even(n)` → 1 if even, 0 if odd

## Building

```bash
cargo build --release
```

This produces `target/release/libexample_plugin.dylib` (or `.so` on Linux, `.dll` on Windows).

## Using from Aether

```aether
let math = load_plugin("path/to/libexample_plugin.dylib")

let sum = math.add(40, 2)           // 42
let product = math.multiply(6, 7)    // 42
let powered = math.power(2, 10)      // 1024
let check = math.is_even(100)        // 1
```

See `../examples/plugin_demo.ae` for a complete example.

## Plugin Protocol (MVP)

This plugin uses the manual FFI protocol. The key parts:

### 1. Plugin Metadata

```rust
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const PluginFnPtr,
}
```

### 2. Initialization Function

```rust
#[no_mangle]
pub extern "C" fn aether_plugin_init() -> *const PluginMetadata {
    // Return metadata with function names and pointers
}
```

### 3. Function Signature

```rust
type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;
```

Functions take an array of i64 arguments and return i64. Error signaling: return `i64::MIN`.

## Current Limitations

- **Integer-only**: Args and return values must be `int`
- **Synchronous**: Functions block the interpreter
- **Manual**: No macro to generate boilerplate (Phase 2)

## Next Steps

Phase 2 will introduce the `aether-plugin` crate with `#[aether_export]` macro:

```rust
use aether_plugin::*;

#[aether_export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

Phase 3 will add support for `String`, `Vec`, and `HashMap` types.

See `../docs/dev/FFI_PLUGIN_SYSTEM.md` for the full design.
