# Rule: Add a Language Feature (New Syntax)

## When to follow this rule

You are adding new syntax — a new token, a new AST node, a new statement, or a new expression.

## Steps

1. **Lexer** — add the token to `src/lexer/token.rs`. Add the lexing rule in `src/lexer/lexer.rs`.
2. **AST** — add the node to `src/parser/ast.rs`. Add it to the relevant `Stmt` or `Expr` enum.
3. **Parser** — add the parse rule in `src/parser/parser.rs`. Write a parser test in `src/parser/parser_tests.rs`.
4. **Evaluator** — add the match arm in the right sub-module:
   - New statement → `src/interpreter/evaluator/statements.rs`
   - New expression → `src/interpreter/evaluator/expressions.rs`
   - New operator → `src/interpreter/evaluator/operators.rs`
5. **Checker** — add a match arm in `check_stmt` or `check_expr` in `src/checker.rs`. Missing arms are compile errors.
6. **Formatter** — add a match arm in `src/formatter.rs`. Missing arms are compile errors.
7. **Tests** — `tests/<feature>_test.rs`. See `write-tests.md`.
8. **Example** — `examples/<feature>_demo.ae` covering all new syntax.
9. **Docs** — update the relevant `docs/lang/*.md`. See `update-docs.md`.

## Check your work

```bash
cargo build
cargo run -- check examples/<feature>_demo.ae
cargo run -- fmt examples/<feature>_demo.ae
cargo test -- --test-threads=1
```
