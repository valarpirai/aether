# Rule: Add a Built-in Function

## When to follow this rule

You are adding a new function that needs interpreter internals, native I/O, or can't be written in Aether.

If you can write it in Aether, stop. Use `add-stdlib.md` instead.

## Steps

1. Add the function to `src/interpreter/builtins.rs`.
2. Add its name to the `BUILTINS` slice in `src/checker.rs`.
3. Add a test in `tests/integration_test.rs`.
4. Add the function name to the feature table in `CLAUDE.md`.

## Check your work

```bash
cargo run -- check examples/<feature>_demo.ae
cargo test --test checker_test -- --test-threads=1
cargo test --test integration_test -- --test-threads=1
```

See `docs/dev/DEVELOPMENT.md` for the full decision tree.
