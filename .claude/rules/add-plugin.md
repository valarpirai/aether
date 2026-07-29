# Rule: Write a Plugin or Change the FFI Protocol

## When to follow this rule

You are writing a new Aether plugin, or changing the FFI protocol that plugins
speak to the interpreter.

If you are adding a normal Rust function to the interpreter itself, stop. Use
`add-builtin.md` instead.

## Layout

Plugins are standalone crates under `plugins/<name>/`, each depending on the
`aether-plugin` SDK crate by relative path:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
aether-plugin = { path = "../../aether-plugin" }
```

The interpreter side lives in `src/interpreter/plugin.rs` (loading, dispatch) and
`src/interpreter/ffi_helpers.rs` (value marshalling).

## Writing a new plugin

1. Create `plugins/<name>/` with the `Cargo.toml` above.
2. Write the exported functions in `src/lib.rs`.
3. Register them with `aether_plugin_init_v2!(...)` — V2 is the current protocol
   and supports `String`, `Vec<i64>`, `Vec<String>`, and `HashMap<String, i64>`.
   Use `aether_plugin_init!(...)` only for an int-only V1 plugin.
4. Build it: `cargo build -p <name>` produces
   `target/debug/lib<name>.{dylib,so,dll}`.
5. Add `examples/<name>_demo.ae` that calls `load_plugin()` on that path.
6. Add tests in `tests/plugin_v2_test.rs` (or `tests/plugin_test.rs` for V1).

## Changing the protocol

The protocol version is declared in the `PluginMetadata` returned by the init
macros in `aether-plugin/src/lib.rs`. The interpreter auto-detects V1 vs V2 at
load time by which symbol the library exports.

Any protocol change touches all four of these — keep them in step:

| Concern | File |
|---|---|
| Metadata struct, init macros | `aether-plugin/src/lib.rs` |
| Type conversion (`FromAether`/`IntoAether`) | `aether-plugin/src/convert.rs` |
| Loading and symbol lookup | `src/interpreter/plugin.rs` |
| Value marshalling | `src/interpreter/ffi_helpers.rs` |

Adding a newly supported type means adding a conversion in `convert.rs` **and**
the matching marshalling arm in `ffi_helpers.rs`. A type handled on only one side
fails at runtime, not at compile time.

Do not break V1. Existing V1 plugins must keep loading.

## Docs

Update `docs/dev/FFI_PLUGIN_SYSTEM.md` for protocol or type-mapping changes, and
`docs/lang/PLUGIN_GUIDE.md` for anything that changes how a plugin author writes
a plugin.

## Check your work

```bash
cargo build -p <name>
cargo test --test plugin_v2_test -- --test-threads=1
cargo test --test plugin_test -- --test-threads=1
cargo test --test plugin_macro_test -- --test-threads=1
cargo run -- examples/<name>_demo.ae
```

Plugins load a shared library into the process — a panic across the FFI boundary
aborts the interpreter. Return `Result` and let the SDK convert it into an Aether
error rather than unwrapping.
