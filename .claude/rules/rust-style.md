# Rule: Rust Code Style

## When to follow this rule

You are writing or editing any `.rs` file in this project.

## Naming

- Functions and variables: `snake_case`
- Types, enums, structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

## Error handling

Return `Result<T, RuntimeError>` from fallible interpreter functions. Never use `unwrap()` or `panic!()` outside tests. Error messages must include line numbers.

## Clippy

Fix all new warnings. Do not add `#[allow(clippy::...)]` without a comment explaining why. The five existing `mutable_key_type` warnings are acceptable.

## File size

Split any `.rs` file that exceeds 1000 lines into a sub-module directory. See how `src/interpreter/evaluator/` was split for the pattern.

## Before every commit

```bash
cargo fmt
cargo clippy
cargo test -- --test-threads=1
```

See `docs/dev/DEVELOPMENT.md` for performance-critical patterns that must not be changed (`Rc<Stmt>`, `std::mem::swap`, `Rc::make_mut`).
