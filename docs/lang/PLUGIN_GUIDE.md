# Plugin Guide: Wrapping Rust Libraries for Aether

This guide builds an Aether plugin. It also shows how to wrap a crate from the
Rust ecosystem: a database, a cipher, an image library. For the reference of
helpers and supported types, see [PLUGINS.md](PLUGINS.md).

At the end you have a shared library. Aether loads it with `load_plugin`. You
call its functions as methods.

## How plugins work

A plugin is a Rust `cdylib`. That is a `.dylib`, a `.so`, or a `.dll`. At load
time Aether calls an init function the plugin exports. It reads a table of names
and pointers. Then Aether code calls each function as a method.

```
Aether program            plugin (.dylib)              Rust crate
--------------            ---------------              ----------
load_plugin(path)   -->   aether_plugin_init_v2   -->  (your functions)
plugin.fn(args)     -->   fn_ffi wrapper          -->  redis::Client, etc.
```

The `#[aether_export]` macro writes the wrapper for each function. The wrapper
converts the arguments from Aether values into Rust types. It calls your
function. It converts the result back. You write ordinary Rust.

## Type mapping

Only these types cross the boundary. Choose your signatures from this set.

| Aether type | Rust type |
|-------------|-----------|
| `int` | `i64` |
| `string` | `String` |
| `array` of ints | `Vec<i64>` |
| `array` of strings | `Vec<String>` |
| `dict` of string→int | `HashMap<String, i64>` |

A function that uses only `i64` runs on the V1 protocol. A function that uses
`String`, `Vec`, or `HashMap` runs on V2. The macro picks the protocol. Aether
detects it at load time.

Return `Result<T, String>` when a call can fail. `Err` reaches Aether as a
catchable error. `Ok(v)` returns `v`. Never panic across the boundary. A panic
there is undefined behaviour.

## Step 1 — Create the crate

Plugins live under `plugins/`. Make a new crate there:

```
plugins/myplugin/
  Cargo.toml
  .cargo/config.toml
  src/lib.rs
```

`plugins/myplugin/Cargo.toml`:

```toml
[package]
name = "myplugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
aether-plugin = { path = "../../aether-plugin" }
# Add the crate you are wrapping, e.g.:
# redis = "0.27"
```

Add the crate to the workspace `members` in the root `Cargo.toml`:

```toml
members = [".", "aether-plugin", "aether-plugin/aether-plugin-macro",
           "plugins/example_plugin_v2", "plugins/redis_plugin", "plugins/myplugin"]
```

## Step 2 — Allow undefined symbols (macOS)

The plugin calls FFI helpers named `aether_value_*`. Those helpers live in the
main `aether` binary. They are not in the plugin. On macOS you must tell the
plugin to resolve them at runtime. Create `plugins/myplugin/.cargo/config.toml`:

```toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]
```

The main binary is built with `-rdynamic`. See the repo-root
`.cargo/config.toml`. That flag exports the symbols. On Linux `-rdynamic` on the
host binary is enough. You do not need this per-plugin config there.

## Step 3 — Write the functions

`plugins/myplugin/src/lib.rs`:

```rust
use aether_plugin::*;

#[aether_export]
fn shout(text: String) -> String {
    text.to_uppercase()
}

#[aether_export]
fn add_all(nums: Vec<i64>) -> i64 {
    nums.iter().sum()
}

aether_plugin_init_v2!(shout, add_all);
```

Use `aether_plugin_init!` if every function is `i64`-only.

## Step 4 — Build

Build from inside the plugin directory. That way its `.cargo/config.toml`
applies:

```bash
cd plugins/myplugin
cargo build --release
```

This makes `target/release/libmyplugin.dylib`. The workspace shares one
`target/`. Do not build from the repo root. The root applies `-rdynamic`, and
the plugin fails to link. Always build a plugin from its own directory.

## Step 5 — Use from Aether

```aether
fn main() {
    let p = load_plugin("./target/release/libmyplugin.dylib")
    println(p.shout("hello"))        // HELLO
    println(p.add_all([1, 2, 3]))    // 6
}
```

Catch failures with `try/catch`. A function that returns `Err` raises a
catchable error:

```aether
try {
    p.some_fallible_call("bad input")
} catch (e) {
    println("plugin error:", e.message)
}
```

## Case study: wrapping the `redis` crate

The full plugin is in `plugins/redis_plugin/`. The demo is
`examples/redis_plugin_demo.ae`. It shows the pattern most real bindings need. A
resource must outlive a single call. Here that resource is a live connection.

The boundary is scalar-only. You cannot hand a `redis::Connection` to Aether. So
the plugin keeps its connections in a process-global registry. It hands Aether
an integer **handle id** instead:

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use redis::Commands;

static CONNECTIONS: Mutex<Option<HashMap<i64, redis::Connection>>> = Mutex::new(None);
static NEXT_ID: Mutex<i64> = Mutex::new(1);

#[aether_export]
fn conn_open(url: String) -> Result<i64, String> {
    let client = redis::Client::open(url).map_err(|e| format!("open: {e}"))?;
    let connection = client.get_connection().map_err(|e| format!("connect: {e}"))?;
    // ... store `connection` under a fresh id, return the id ...
}

#[aether_export]
fn conn_get(id: i64, key: String) -> Result<String, String> {
    // ... look up the connection by id and reuse it ...
}
```

Every command takes the id as its first argument. Every command reuses the same
connection. The socket opens once, not once per call.

### Giving it method syntax

Passing the id by hand is clumsy. Wrap it in an Aether struct. Then you write
`conn.get(key)`:

```aether
struct RedisConn {
    plugin
    id

    fn get(self, key)         { return self.plugin.conn_get(self.id, key) }
    fn set(self, key, value)  { return self.plugin.conn_set(self.id, key, value) }
    fn close(self)            { return self.plugin.conn_close(self.id) }
}

fn redis_connect(url) {
    let plugin = load_plugin("./target/release/libredis_plugin.dylib")
    return RedisConn { plugin: plugin, id: plugin.conn_open(url) }
}
```

```aether
let conn = redis_connect("redis://127.0.0.1/")
conn.set("k", "v")
conn.get("k")     // reuses the same connection
conn.close()      // release it when done
```

### Lifecycle caveat

The connection lives until you call `conn.close()`. There is no opaque-handle
type yet. A connection does not close when the struct goes out of scope. So
close it yourself. A `finally` block is the safe place. A closed or unknown id
raises a catchable error. It never aliases another connection, because ids are
never reused.

## Limitations

- Element types are scalar. Arrays hold `int` or `string`. Dicts map `string` to `int`. Nested collections do not cross the boundary.
- Calls are synchronous. A plugin call blocks the interpreter thread.
- There is no native handle type. Use the integer-handle pattern above. Close the resource yourself.

## Checklist

1. Crate under `plugins/`, `crate-type = ["cdylib"]`, depends on `aether-plugin`.
2. Added to workspace `members` in the root `Cargo.toml`.
3. `.cargo/config.toml` with `dynamic_lookup` (macOS).
4. Functions annotated `#[aether_export]`, signatures use only boundary types, fallible ones return `Result<T, String>`.
5. Registered with `aether_plugin_init_v2!` (or `aether_plugin_init!` for int-only).
6. Built from inside the plugin directory.
