# Rule: Write Tests

## When to follow this rule

You are implementing any feature, fixing any bug, or adding any function.

Write the test first. Then write the code.

## What every test file needs

Create `tests/<feature>_test.rs`. Cover three cases:

1. **Happy path** — normal input, expected output.
2. **Edge cases** — empty input, zero, null, boundary values.
3. **Error cases** — wrong type, wrong arg count, out of bounds.

## How to initialize the interpreter in tests

Use `Evaluator::new_without_stdlib()` for unit-level tests. It is 760x faster than `Evaluator::new()`.

Use `Evaluator::new()` only when the test calls stdlib functions.

## How to run tests

```bash
# Always use --test-threads=1
cargo test -- --test-threads=1

# Single file
cargo test --test <feature>_test -- --test-threads=1

# Show output
cargo test -- --test-threads=1 --nocapture
```

See `docs/dev/TESTING.md` for test patterns and examples.
