# Plugin System - MVP Implementation Status

## ✅ Phase 1: Core FFI (COMPLETE)

### Implemented
- ✅ `libloading` integration in `src/interpreter/plugin.rs`
- ✅ `load_plugin(path)` built-in
- ✅ FFI type conversion: `Value` ↔ primitives
- ✅ Plugin value type with member access (`plugin.func_name`)
- ✅ `Value::Plugin` and `Value::PluginFn` variants
- ✅ Calling plugin functions from Aether
- ✅ Manual plugin protocol (no macro yet)

### Plugin API (Manual)

Plugins must export:

```c
#[repr(C)]
struct PluginMetadata {
    version: i32,
    function_count: i32,
    function_names: *const *const c_char,
    function_ptrs: *const PluginFnPtr,
}

#[no_mangle]
pub extern "C" fn aether_plugin_init() -> *const PluginMetadata {
    // Return metadata
}

type PluginFnPtr = unsafe extern "C" fn(*const *const Value, c_int) -> *mut FfiResult;

#[repr(C)]
struct FfiResult {
    is_ok: bool,
    value: Value,
    error_msg: *mut c_char,
}
```

### Next Steps

1. **Create example plugin** — handwritten math plugin (add, mul)
2. **Test with Aether program** — load and call functions
3. **Document manual plugin authoring**
4. **Phase 2**: Create `aether-plugin` crate with `#[aether_export]` macro

## File Changes

| File | Changes |
|------|---------|
| `Cargo.toml` | Added `libloading = "0.8"` dependency |
| `src/interpreter/plugin.rs` | **NEW** — Plugin loader, FFI bridge, registry |
| `src/interpreter/mod.rs` | Added `pub mod plugin` |
| `src/interpreter/value.rs` | Added `Plugin` and `PluginFn` variants |
| `src/interpreter/evaluator/members.rs` | Added plugin member access |
| `src/interpreter/evaluator/functions.rs` | Added `PluginFn` call handling |
| `src/interpreter/evaluator/mod.rs` | Registered `load_plugin` builtin |
| `src/interpreter/builtins.rs` | Added `builtin_load_plugin` |
| `docs/dev/FFI_PLUGIN_SYSTEM.md` | **NEW** — Full design document |
| `docs/dev/BACKLOG.md` | Added plugin system to Tier 5 |

## Known Limitations (MVP)

- Manual FFI is unsafe and verbose
- No automatic type conversion
- No proc macro yet (`#[aether_export]`)
- String/Vec/Dict not yet supported (primitives only)
- Synchronous only (no async plugin functions)
- No plugin search path

These will be addressed in Phase 2-4.
