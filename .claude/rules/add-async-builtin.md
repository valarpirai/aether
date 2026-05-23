# Rule: Add an Async I/O Built-in

## When to follow this rule

You are adding a built-in that performs async I/O — file, network, or timer — and must not block the main thread.

If the function can run synchronously, stop. Use `add-builtin.md` instead.

## Steps

1. Open `src/interpreter/evaluator/functions.rs`.
2. Add the function name to the match in `try_submit_io_task`.
3. Spawn the blocking work onto the I/O thread pool. Send the result back through the channel.
4. Return `Value::promise_io(rx)` from the match arm.
5. Add the name to the `BUILTINS` slice in `src/checker.rs`.
6. Add tests in `tests/io_pool_test.rs`. Test both resolved value and concurrent execution.
7. Update the relevant `docs/lang/` file (`HTTP.md`, `TIME.md`, or a new file).

## How the I/O thread pool works

`try_submit_io_task` submits a closure to the thread pool. The closure runs on a worker thread. It sends the result through a `oneshot` channel. The evaluator wraps the receiver in `Value::promise_io(rx)`. `await` resolves it by blocking on `rx.recv()`.

## Check your work

```bash
cargo test --test io_pool_test -- --test-threads=1
cargo run -- check examples/<feature>_demo.ae
```

See `docs/dev/INTERPRETER.md` and `docs/lang/ASYNC.md` for the full async model.
