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

| Aether Type | Rust Type | Notes |
|-------------|-----------|-------|
| `Int` | `i64` | Direct copy |
| `Float` | `f64` | Direct copy |
| `String` | `String` | Clone from `Rc<String>` |
| `Bool` | `bool` | Direct copy |
| `Null` | `Option<T>` | `None` |
| `Array` | `Vec<Value>` | Clone from `Rc<RefCell<Vec<Value>>>` |
| `Dict` | `HashMap<String, Value>` | Clone, string keys only |
| Function | ✗ | Not passable to plugins (closure lifetime) |
| Promise | ✗ | Plugins can't await (no evaluator access) |

## Plugin Registration Protocol

Every plugin exports a registration function:

```rust
#[no_mangle]
pub extern "C" fn aether_plugin_init() -> *const PluginMetadata {
    Box::into_raw(Box::new(PluginMetadata {
        version: 1,
        functions: vec![
            FunctionDescriptor {
                name: "greet".to_string(),
                func: greet_ffi_wrapper,
                arity: 1,
            },
            FunctionDescriptor {
                name: "add_numbers".to_string(),
                func: add_numbers_ffi_wrapper,
                arity: 2,
            },
        ],
    }))
}
```

The `#[aether_export]` macro generates:
1. FFI wrapper function (C ABI)
2. Type conversion boilerplate
3. Error handling (`Result<T, E>` → `RuntimeError`)

## Memory Safety

**Ownership model:**
- Aether → Plugin: Copy primitives, clone strings/arrays
- Plugin → Aether: Transfer ownership of return value
- No shared `Rc` — plugins can't hold Aether values across calls

**Lifetime constraints:**
- Plugin functions are `fn(args) -> result` — no stored state between calls
- No access to interpreter internals (Environment, Evaluator)
- No async — plugins are synchronous functions

**ABI stability:**
- Use `repr(C)` for all FFI boundary types
- Pin `aether-plugin` crate version in plugin `Cargo.toml`
- Breaking changes require version bump

## Implementation Phases

### Phase 1: Core FFI (MVP)
- [ ] `libloading` integration in `src/interpreter/plugin.rs`
- [ ] `load_plugin(path)` built-in
- [ ] FFI type conversion: `Value` ↔ primitives/String/Vec
- [ ] Plugin registry per `Evaluator`
- [ ] Manual plugin (no macro) — test with handwritten registration

### Phase 2: Plugin API Crate
- [ ] Create `aether-plugin/` workspace member
- [ ] `#[aether_export]` proc macro
- [ ] Auto-generate registration function
- [ ] Error handling: `Result<T, E>` → Aether exceptions

### Phase 3: Complex Types
- [ ] Dict ↔ `HashMap<String, Value>`
- [ ] `Option<T>` → null handling
- [ ] Custom error types (structs with message field)

### Phase 4: Stdlib Integration
- [ ] Example plugins: `postgres`, `redis`, `image`
- [ ] Plugin search path: `./plugins`, `~/.aether/plugins`
- [ ] Plugin versioning / compatibility checks

## Security Considerations

**Sandboxing:**
- Plugins run with same privileges as interpreter (no sandbox)
- Can access filesystem, network, spawn processes
- Trust model: only load plugins you trust

**Future hardening:**
- Plugin capability system (declare needed permissions)
- WASM plugins for untrusted code
- Process isolation (plugin runs in child process)

## Example Use Cases

### PostgreSQL Client

```rust
// aether-postgres/src/lib.rs
use aether_plugin::*;
use postgres::{Client, NoTls};

#[aether_export]
fn pg_connect(url: String) -> Result<i64, String> {
    let client = Client::connect(&url, NoTls)
        .map_err(|e| e.to_string())?;
    let handle = CONNECTIONS.lock().unwrap().insert(client);
    Ok(handle as i64)
}

#[aether_export]
fn pg_query(handle: i64, sql: String) -> Result<Vec<HashMap<String, Value>>, String> {
    // Execute query, convert rows to Aether dicts
    // ...
}
```

```aether
let db = load_plugin("libaether_postgres.so")
let conn = db.pg_connect("postgresql://localhost/mydb")
let rows = db.pg_query(conn, "SELECT * FROM users WHERE age > 21")
for row in rows {
    print(row["name"], row["email"])
}
```

### Image Processing

```rust
use aether_plugin::*;
use image::{ImageBuffer, Rgba};

#[aether_export]
fn load_image(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}

#[aether_export]
fn resize(data: Vec<u8>, width: i64, height: i64) -> Result<Vec<u8>, String> {
    // Use `image` crate to resize
    // ...
}
```

## Open Questions

1. **Thread safety** — what if plugin spawns threads? Document as "plugins must be thread-safe for I/O pool"
2. **Async plugins** — defer to Phase 5; for now plugins are synchronous
3. **Plugin unloading** — `dlclose` on drop or keep loaded forever?
4. **Version mismatch** — fail gracefully if plugin compiled with incompatible `aether-plugin` version
5. **Cross-platform** — test on Linux, macOS, Windows

## References

- Python C extension API
- Lua C API (`lua_State`, stack-based)
- Ruby native extensions (`rb_define_method`)
- WebAssembly Interface Types (future sandboxing)
