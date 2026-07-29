# Rule: Add a Stdlib Function

## When to follow this rule

You are adding a function that can be written in Aether using existing primitives.

## Steps

1. Pick the right module: `stdlib/core.ae`, `stdlib/collections.ae`, `stdlib/math.ae`, or `stdlib/string.ae`.
2. Write the function in Aether. Handle `null` for optional args — Aether passes `null` for omitted parameters.
3. Add its name to the `BUILTINS` slice in `src/checker.rs`.
4. Add a test in `tests/stdlib_<module>_test.rs`.
5. Add the function name and signature to the quick-reference table in
   `docs/lang/STDLIB.md`, plus a detail entry under the module's section.
6. Add the function name to the feature table in `docs/dev/ARCHITECTURE.md`
   (Feature Summary).

## Check your work

```bash
cargo run -- check examples/<feature>_demo.ae
cargo test --test checker_test -- --test-threads=1
cargo test --test stdlib_<module>_test -- --test-threads=1
```

See `docs/dev/DEVELOPMENT.md` for the builtin-vs-stdlib decision tree.
