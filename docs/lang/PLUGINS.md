# FFI / Plugin System

Load compiled Rust shared libraries (`.so`/`.dylib`/`.dll`) from Aether programs to access the Rust ecosystem.

For a step-by-step tutorial on writing a plugin and wrapping an existing crate, see [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md).

## Loading a Plugin

```aether
let plugin = load_plugin("path/to/libexample.dylib")
```

## Calling Plugin Functions

Plugin functions are called like methods:

```aether
let result = plugin.function_name(arg1, arg2)
```

## Example

```aether
// Load a math plugin
let math = load_plugin("examples/plugins/libexample_plugin.dylib")

// Call plugin functions
let sum = math.add(40, 2)              // 42
let product = math.multiply(6, 7)       // 42
let powered = math.power(2, 10)         // 1024
let check = math.is_even(100)           // 1 (true)

// Use in expressions
let result = math.add(
    math.multiply(3, 4),
    math.power(2, 3)
)  // 20
```

## Supported Types

Plugin functions map Aether values to Rust types through two protocols. The
protocol is chosen automatically by the registration macro and detected at load
time — you do not select it manually.

| Aether type | Rust type |
|-------------|-----------|
| `int` | `i64` |
| `string` | `String` |
| `array` of ints | `Vec<i64>` |
| `dict` of string→int | `HashMap<String, i64>` |

`int`-only functions use the V1 protocol; any function that uses `String`,
`Vec`, or `HashMap` uses the V2 protocol. Both can coexist in one plugin.

## Current Limitations

- **Element types are scalar**: arrays hold `int`, dicts map `string` to `int`; nested collections are not yet supported.
- **Synchronous**: plugin functions are blocking.

## Creating a Plugin (Easy Way - With Macro)

### 1. Setup

```toml
# Cargo.toml
[dependencies]
aether-plugin = { path = "path/to/aether-plugin" }

[lib]
crate-type = ["cdylib"]
```

### 2. Write Functions

```rust
// src/lib.rs
use aether_plugin::*;

#[aether_export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[aether_export]
fn factorial(n: i64) -> i64 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

// Register all exported functions
aether_plugin_init!(add, factorial);
```

### 3. Build

```bash
cargo build --release
# Produces target/release/libmyplugin.dylib (or .so/.dll)
```

That's it! **No manual FFI, no unsafe code, zero boilerplate.**

### String, Array, and Dict Functions (V2)

Functions that take or return `String`, `Vec<i64>`, or `HashMap<String, i64>`
register with `aether_plugin_init_v2!` instead of `aether_plugin_init!`.

```rust
use aether_plugin::*;
use std::collections::HashMap;

#[aether_export]
fn to_upper(s: String) -> String {
    s.to_uppercase()
}

#[aether_export]
fn sort_array(mut nums: Vec<i64>) -> Vec<i64> {
    nums.sort();
    nums
}

#[aether_export]
fn sum_values(scores: HashMap<String, i64>) -> i64 {
    scores.values().sum()
}

aether_plugin_init_v2!(to_upper, sort_array, sum_values);
```

Call them from Aether like any other plugin method:

```aether
let p = load_plugin("target/release/libmyplugin.dylib")
p.to_upper("hello")                          // "HELLO"
p.sort_array([5, 2, 8, 1])                   // [1, 2, 5, 8]
p.sum_values({"a": 10, "b": 20})             // 30
```

## Creating a Plugin (Hard Way - Manual FFI)

For advanced use cases or understanding how it works under the hood, see the manual FFI approach in [FFI_PLUGIN_SYSTEM.md](../dev/FFI_PLUGIN_SYSTEM.md) and `example_plugin/`.

## Error Handling

Plugin functions that fail return an error:

```aether
try {
    plugin.function_with_wrong_args(1, 2, 3)
} catch (e) {
    println("Plugin error:", e.message)
}
```

Common errors:
- **Wrong arity**: Function called with incorrect number of arguments
- **Type error**: Argument type does not match the function signature
- **Function not found**: Plugin doesn't export the requested function

## Use Cases

Plugins enable Aether to leverage the entire Rust ecosystem:

- **Database drivers**: PostgreSQL, MySQL, Redis, SQLite
- **Image processing**: Resize, filter, format conversion
- **Cryptography**: Hashing, encryption, signing
- **Machine learning**: ONNX runtime, model inference
- **System integration**: Any Rust crate you need

## Completed Phases

**Phase 1** ✅: Core FFI (manual)  
**Phase 2** ✅: `aether-plugin` crate with `#[aether_export]` proc macro  
**Phase 3** ✅: String, array, and dict support (V2 protocol)

## Future Phases

**Phase 4**: Async plugin functions

See the [full design document](../dev/FFI_PLUGIN_SYSTEM.md) for details.
