# Rule: Add a Value Variant

## When to follow this rule

You are adding a new variant to the `Value` enum in `src/interpreter/value.rs`.

## Steps

1. Add the variant to `Value` in `src/interpreter/value.rs`.
2. Use `Rc<T>` if the data needs reference semantics. Use helper methods — never construct raw variants.
3. Implement `Display` and `Debug` for the new variant.
4. Add match arms everywhere `Value` is exhaustively matched. The compiler lists every location.
5. Add a GC test in `tests/gc_test.rs`. The test creates and drops the value. It verifies no `Rc` cycle leaks.
6. Run `leaks --atExit -- ./target/debug/aether examples/<feature>_demo.ae` on macOS.

## Check your work

```bash
cargo build
cargo test --test gc_test -- --test-threads=1
leaks --atExit -- ./target/debug/aether examples/<feature>_demo.ae
```

See `docs/dev/MEMORY_MANAGEMENT.md` for `Rc` cycle rules and `Weak<T>` guidance.
