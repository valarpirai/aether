# Rule: Add a Member Method or Property

## When to follow this rule

You are adding a new method or property to a built-in type: `array`, `dict`, `set`, `string`, or `struct`.

## Steps

1. Open `src/interpreter/evaluator/members.rs`.
2. Find `eval_method_call` or `eval_member` for the target type.
3. Add a match arm for the new name. Return `RuntimeError` for wrong arg count or wrong type.
4. Add a test in the relevant test file — `tests/array_methods_test.rs`, `tests/dict_test.rs`, `tests/set_test.rs`, `tests/string_methods_test.rs`, or `tests/struct_test.rs`.
5. Update `docs/lang/STDLIB.md` with the method signature and what it does.

## What not to do

Do not add member names to the `BUILTINS` slice in `src/checker.rs`. The static checker does not validate member names.

## Check your work

```bash
cargo test --test <type>_test -- --test-threads=1
```
