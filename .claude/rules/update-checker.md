# Rule: Update the Static Checker

## When to follow this rule

You added a new built-in, a new stdlib function, a new `Stmt` variant, or a new `Expr` variant.

## Steps

**New built-in or stdlib function:**
Add the name to the `BUILTINS` slice in `src/checker.rs`.

**New `Stmt` AST variant:**
Add a match arm in `check_stmt` in `src/checker.rs`.

**New `Expr` AST variant:**
Add a match arm in `check_expr` in `src/checker.rs`.

Missing match arms are compile errors. Missing `BUILTINS` entries cause false "undefined variable" warnings on valid code.

## Check your work

```bash
# Must report no false positives on valid code
cargo run -- check examples/<feature>_demo.ae

# New checker behaviour must be tested
cargo test --test checker_test -- --test-threads=1
```
