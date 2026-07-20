# aether-plugin

Proc macro and runtime support for creating Aether FFI plugins with minimal boilerplate.

## Installation

Add to your plugin's `Cargo.toml`:

```toml
[dependencies]
aether-plugin = { path = "path/to/aether-plugin" }

[lib]
crate-type = ["cdylib"]
```

## Usage

```rust
use aether_plugin::*;

#[aether_export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[aether_export]
fn multiply(a: i64, b: i64) -> i64 {
    a * b
}

// Register all exported functions
aether_plugin_init!(add, multiply);
```

That's it! No manual FFI, no unsafe code, no boilerplate.

## What the Macro Does

The `#[aether_export]` macro automatically generates:

1. **FFI wrapper function** with C calling convention
2. **Argument extraction** from raw pointer array
3. **Arity checking** (returns error if wrong arg count)
4. **Type conversions** (currently i64 only)

## Example: Before and After

### Before (Manual FFI - 60+ lines)

```rust
use std::ffi::{c_char, c_int};

type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;

#[repr(C)]
pub struct PluginMetadata { /* ... */ }

static FUNC_NAMES: &[&[u8]] = &[b"add\0"];
static FUNC_PTRS: &[PluginFnPtr] = &[add_impl];

#[no_mangle]
pub extern "C" fn aether_plugin_init() -> *const PluginMetadata {
    let name_ptrs: Vec<*const c_char> = FUNC_NAMES
        .iter()
        .map(|s| s.as_ptr() as *const c_char)
        .collect();
    let names_box = Box::leak(name_ptrs.into_boxed_slice());
    Box::into_raw(Box::new(PluginMetadata {
        version: 1,
        function_count: FUNC_NAMES.len() as c_int,
        function_names: names_box.as_ptr(),
        function_ptrs: FUNC_PTRS.as_ptr(),
    }))
}

unsafe extern "C" fn add_impl(args: *const i64, argc: c_int) -> i64 {
    if argc != 2 { return i64::MIN; }
    let a = *args.offset(0);
    let b = *args.offset(1);
    a + b
}
```

### After (With Macro - 5 lines!)

```rust
use aether_plugin::*;

#[aether_export]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

aether_plugin_init!(add);
```

**12× less code, zero unsafe, pure Rust!**

## Building

```bash
cargo build --release
```

This produces a shared library:
- **Linux**: `target/release/libmyplugin.so`
- **macOS**: `target/release/libmyplugin.dylib`
- **Windows**: `target/release/myplugin.dll`

## Using from Aether

```aether
let plugin = load_plugin("path/to/libmyplugin.dylib")
let result = plugin.add(40, 2)  // 42
```

## Current Limitations

- **Integer-only**: Functions must take and return `i64`
- **Synchronous**: Functions are blocking
- **No Result<T, E>**: Error handling via `i64::MIN` sentinel

Phase 3 will add support for `String`, `Vec`, `HashMap`, and proper error types.

## Example Plugins

See:
- `example_plugin_macro/` - Full working example
- `examples/plugin_macro_demo.ae` - Aether usage example

## API Reference

### `#[aether_export]`

Marks a function for export to Aether.

**Requirements:**
- Function must be public or private (visibility doesn't matter)
- Parameters must be `i64`
- Return type must be `i64`

**Generates:**
- `{fn_name}_ffi` wrapper function
- Arity checking
- Type conversion

### `aether_plugin_init!(...)`

Generates the plugin initialization function.

**Usage:**
```rust
aether_plugin_init!(fn1, fn2, fn3);
```

Lists all functions marked with `#[aether_export]`.

**Generates:**
- `aether_plugin_init()` C export
- Function registry metadata
- Static function pointer table

## Architecture

```
┌─────────────────┐
│ Your Rust Code  │  #[aether_export]
│   fn add(...)   │  ─────────────┐
└─────────────────┘                │
                                   ▼
                        ┌──────────────────────┐
                        │ Generated FFI Code   │
                        │  - Type conversion   │
                        │  - Arity checking    │
                        │  - C calling conv    │
                        └──────────────────────┘
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │ Aether Interpreter   │
                        │  plugin.add(40, 2)   │
                        └──────────────────────┘
```

## License

MIT
