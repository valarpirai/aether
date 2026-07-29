# FFI / Plugin System Design

## Overview

Allow Aether programs to load and call functions from compiled Rust shared libraries (`.so`/`.dylib`/`.dll`), enabling access to the entire Rust ecosystem without rewriting every library as a built-in.

## Architecture

```
┌─────────────┐         ┌──────────────┐        ┌─────────────┐
│  Aether     │ calls   │   Plugin     │  uses  │   Rust      │
│  Program    │────────>│   Wrapper    │───────>│   Library   │
│  (.ae)      │         │   (FFI)      │        │   (crate)   │
└─────────────┘         └──────────────┘        └─────────────┘
      │                        │
      │ load_plugin("lib.so")  │
      └────────────────────────┘
```

### Components

1. **Plugin Loader** (`src/interpreter/plugin.rs`) — dynamic library loading via `libloading`
2. **FFI Bridge** — `Value` ↔ Rust type conversion layer
3. **Plugin API Crate** (`aether-plugin/`) — derive macro for plugin authors
4. **Registry** — track loaded plugins per interpreter instance

## Plugin Author Experience

```rust
// my_plugin/src/lib.rs
use aether_plugin::*;

#[aether_export]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[aether_export]
fn add_numbers(a: i64, b: i64) -> Result<i64, String> {
    Ok(a + b)
}

// Builds to: libmy_plugin.so / libmy_plugin.dylib / my_plugin.dll
```

```aether
// main.ae
let plugin = load_plugin("libmy_plugin.so")
print(plugin.greet("Aether"))           // "Hello, Aether!"
print(plugin.add_numbers(40, 2))        // 42
```

## Type Mapping

Implemented conversions live in `aether-plugin/src/convert.rs` as `FromAether`
(argument) and `ToAether` (return) impls.

| Aether Type | Rust Type | Direction | Notes |
|-------------|-----------|-----------|-------|
| `int` | `i64` | both | Direct copy |
| `string` | `String` | both | Clone from `Rc<String>` |
| `array` of ints | `Vec<i64>` | both | Clone; non-int elements are a type error |
| `array` of strings | `Vec<String>` | both | Clone |
| `dict` of string→int | `HashMap<String, i64>` | both | String keys only |
| `null` | `Option<T>` | return only | `None` → `null` |
| — | `Result<T, E>` | return only | `Err` becomes a catchable Aether error |

Not yet crossing the boundary — no `FromAether`/`ToAether` impl exists:

| Type | Reason |
|------|--------|
| `float` / `f64` | No impl; add alongside `i64` when needed |
| `bool` | No impl; pass as `int` |
| Nested `array`/`dict` | Element type is fixed at one level |
| `Option<T>` as an argument | Only the return direction is implemented |
| Function / closure | Closure lifetime cannot outlive the call |
| Promise | Plugins have no evaluator access, so cannot await |
| Struct | No representation at the boundary |

## Plugin Registration Protocol

Every plugin exports one init symbol returning a `repr(C)` metadata table
(`src/interpreter/plugin.rs`):

```rust
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const *const c_void, // cast per protocol
}
```

The exported symbol name selects the calling convention:

| Symbol | Protocol | Function pointer signature |
|--------|----------|---------------------------|
| `aether_plugin_init_v2` | V2 | `fn(*const AetherValuePtr, c_int, *mut AetherValuePtr) -> AetherValuePtr` |
| `aether_plugin_init` | V1 | `fn(*const i64, c_int) -> i64` |

`Plugin::load` probes `aether_plugin_init_v2` first and falls back to
`aether_plugin_init`. A `version` field other than 1 or 2 is rejected. Plugin
authors never write this table: `aether_plugin_init!(f, g, ...)` emits it.

The `#[aether_export]` macro generates one `<name>_ffi` wrapper per function:
1. Arity check — mismatch becomes an error, not a crash
2. Argument conversion via `FromAether`, return conversion via `ToAether`
3. Protocol selection — all-`i64` signatures compile to V1, anything else to V2
4. `Result<T, E>` routing — `Err` goes through `out_error` as a catchable error

V1 signals failure by returning `i64::MIN`, so a V1 function cannot legitimately
return that value. V2 has a dedicated error channel and no such reserved value;
prefer V2 for anything non-trivial.

## Memory Safety

**Ownership model:**
- Aether → Plugin: Copy primitives, clone strings/arrays
- Plugin → Aether: Transfer ownership of return value
- No shared `Rc` — plugins can't hold Aether values across calls

**Lifetime constraints:**
- No Aether value survives a call — nothing is shared, everything is copied
- No access to interpreter internals (Environment, Evaluator)
- No async — plugins are synchronous and block the calling thread

**Plugin-owned state:** a plugin may keep its own state in a process-global
registry and hand Aether an integer handle. `plugins/redis_plugin` does this:
`conn_open(url)` stores a live connection and returns its id, command functions
take that id, `conn_close(id)` drops it. The state lives in the plugin, not in
Aether, so the boundary stays scalar-only. Ids are monotonic and never reused,
so a stale handle fails cleanly instead of aliasing another connection. Wrap the
handle in an Aether struct to give callers a method-style API — see
`examples/redis_plugin_demo.ae`.

**ABI stability:**
- Use `repr(C)` for all FFI boundary types
- Pin `aether-plugin` crate version in plugin `Cargo.toml`
- Breaking changes require version bump

## Implementation Status

### Phase 1: Core FFI — done
- [x] `libloading` integration in `src/interpreter/plugin.rs`
- [x] `load_plugin(path)` built-in
- [x] `Value::Plugin` and `Value::PluginFn` variants, plugin member access
- [x] V1 protocol: `fn(*const i64, c_int) -> i64`
- [x] Manual plugin with handwritten registration (`plugins/example_plugin`)

### Phase 2: Plugin API Crate — done
- [x] `aether-plugin/` and `aether-plugin/aether-plugin-macro/` workspace members
- [x] `#[aether_export]` proc macro generating the FFI wrapper
- [x] `aether_plugin_init!` generating the metadata table
- [x] Example: `plugins/example_plugin_macro`

### Phase 3: Complex Types — done
- [x] V2 protocol with a dedicated error channel (`out_error`)
- [x] `String`, `Vec<i64>`, `Vec<String>`, `HashMap<String, i64>`
- [x] 21 C-ABI conversion helpers in `src/interpreter/ffi_helpers.rs`
- [x] `Result<T, E>` → catchable Aether error
- [x] `Option<T>` → `null` (return direction)
- [x] Example: `plugins/example_plugin_v2`

### Phase 4: Ecosystem Integration — partial
- [x] Real-world plugin: `plugins/redis_plugin` wrapping the `redis` crate
- [x] Handle-based persistent state (process-global registry, integer handles)
- [ ] More example plugins: `postgres`, `image`
- [ ] Plugin search path: `./plugins`, `~/.aether/plugins` — today `load_plugin`
      takes an explicit path only
- [ ] Version compatibility check beyond the `version != 1 && != 2` gate

### Phase 5: Not started
- [ ] Async plugin functions (plugin call on the I/O pool, returning a Promise)
- [ ] `float` and `bool` at the boundary
- [ ] Nested collection types
- [ ] Sandboxing / capability system

### Tests

`tests/plugin_test.rs` (12), `tests/plugin_v2_test.rs` (19),
`tests/plugin_macro_test.rs` (10). They load prebuilt `.dylib` files from
`examples/plugins/`, so rebuild those after changing a plugin crate.

## Security Considerations

**Sandboxing:**
- Plugins run with same privileges as interpreter (no sandbox)
- Can access filesystem, network, spawn processes
- Trust model: only load plugins you trust

**Future hardening:**
- Plugin capability system (declare needed permissions)
- WASM plugins for untrusted code
- Process isolation (plugin runs in child process)

## Example Use Case

The pattern for wrapping a stateful crate — open a resource, return a handle,
take the handle on later calls. From `plugins/redis_plugin/src/lib.rs`:

```rust
use aether_plugin::*;
use redis::Commands;

static CONNECTIONS: Mutex<Option<HashMap<i64, redis::Connection>>> = Mutex::new(None);

#[aether_export]
fn conn_open(url: String) -> Result<i64, String> {
    let client = redis::Client::open(url).map_err(|e| format!("open client: {e}"))?;
    let connection = client.get_connection().map_err(|e| format!("connect: {e}"))?;
    // store `connection` under a fresh id, return the id
}

#[aether_export]
fn conn_get(id: i64, key: String) -> Result<String, String> {
    with_conn(id, |c| c.get(&key).map_err(|e| e.to_string()))
}
```

```aether
let redis = load_plugin("examples/plugins/libredis_plugin.dylib")
let id = redis.conn_open("redis://127.0.0.1/")
redis.conn_set(id, "greeting", "hello")
print(redis.conn_get(id, "greeting"))
redis.conn_close(id)
```

A row-returning query is harder: `Vec<HashMap<String, String>>` has no
`ToAether` impl, so a table must be flattened — return `Vec<String>` of JSON
rows and `json_parse` each on the Aether side, or expose column accessors.

## Open Questions

1. **Thread safety** — a plugin called from the I/O pool must be thread-safe.
   Not enforced today; plugin calls are synchronous on the calling thread.
2. **Async plugins** — Phase 5. Would submit the call to the I/O pool and return
   a Promise.
3. **Plugin unloading** — `Library` is held for the process lifetime. Dropping it
   would invalidate function pointers still referenced by `Value::PluginFn`.
4. **Version mismatch** — only the `version` field is checked. A plugin built
   against a different `aether-plugin` with the same version number is undefined
   behaviour.
5. **Cross-platform** — developed and tested on macOS (`.dylib`). Linux
   (`.so`) and Windows (`.dll`) are untested.
6. **Nested types** — one level of collection only; no `Vec<Vec<i64>>` or
   dict-of-dict.

## References

- Python C extension API
- Lua C API (`lua_State`, stack-based)
- Ruby native extensions (`rb_define_method`)
- WebAssembly Interface Types (future sandboxing)
