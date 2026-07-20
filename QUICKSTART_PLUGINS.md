# Plugin System Quick Start

Access the entire Rust ecosystem from Aether programs!

## Try the Example

```bash
# Run the demo
cargo run -- examples/plugin_demo.ae
```

Output:
```
=== Aether Plugin System Demo ===

Loading plugin...
Plugin loaded: <plugin:4 functions>

--- Basic arithmetic ---
math.add(40, 2) = 42
math.multiply(6, 7) = 42

--- Power function ---
math.power(5, 2) = 25
math.power(2, 10) = 1024
...
```

## Write Your First Plugin

### 1. Create a Rust library

```bash
cargo new --lib my_plugin
cd my_plugin
```

### 2. Edit `Cargo.toml`

```toml
[lib]
crate-type = ["cdylib"]
```

### 3. Add dependencies (`Cargo.toml`)

```toml
[dependencies]
aether-plugin = { path = "../aether-plugin" }
```

### 4. Write your functions (`src/lib.rs`)

```rust
use aether_plugin::*;

#[aether_export]
fn greet(code: i64) -> i64 {
    code + 100  // Simple transformation
}

#[aether_export]
fn double(x: i64) -> i64 {
    x * 2
}

// Register all exported functions
aether_plugin_init!(greet, double);
```

**That's it!** No unsafe code, no manual FFI, just pure Rust.

### 5. Build

```bash
cargo build --release
# Produces target/release/libmy_plugin.dylib (or .so/.dll)
```

### 6. Use from Aether

```aether
let plugin = load_plugin("path/to/libmy_plugin.dylib")
let result = plugin.greet(42)  // 142
println(result)
```

## Current Limitations

- **Integer-only**: Functions take and return `int` values
- **Synchronous**: Functions are blocking

## Completed

**Phase 1** ✅: Core FFI (manual)  
**Phase 2** ✅: `#[aether_export]` macro (eliminates boilerplate!)

## What's Next

**Phase 3**: String, array, and dict support

**Phase 4**: Real-world examples (PostgreSQL, Redis, image processing)

## Learn More

- [User Guide](docs/lang/PLUGINS.md) — Complete documentation
- [Design Document](docs/dev/FFI_PLUGIN_SYSTEM.md) — Architecture details
- [Example Plugin](example_plugin/) — Full working example

## Use Cases

With plugins, Aether can leverage:

- 🗄️ **Databases**: PostgreSQL, MySQL, Redis, SQLite
- 🖼️ **Image processing**: resize, filter, convert
- 🔐 **Cryptography**: hashing, encryption, signing
- 🤖 **Machine learning**: ONNX runtime, inference
- ⚙️ **System tools**: Any Rust crate you need

The plugin system opens Aether to the **entire Rust ecosystem** without reimplementing everything as built-ins!
