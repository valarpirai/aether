# Async/Await in Aether

Aether supports JavaScript-style `async`/`await` with a single-threaded event loop and a configurable I/O thread pool.

## Phase 1: Promise-based Async/Await

### Syntax

```aether
// Async function declaration
async fn fetch_user(id) {
    let data = await http_get("/users/" + id)
    return data
}

// Calling returns a Promise (body does NOT execute yet)
let p = fetch_user(42)        // p = <promise:pending>

// await executes the deferred call and returns the result
let user = await fetch_user(42)

// await on a non-Promise is identity — safe to use anywhere
let x = await 42              // x = 42

// Anonymous async function expression
let f = async fn(x) { return x * 2 }
let result = await f(10)      // result = 20
```

### Semantics

| Concept | Behaviour |
|---|---|
| `async fn name()` | Defines an async function. Stored as `Value::AsyncFunction`. |
| Calling async fn | Returns `Value::Promise(Pending)` immediately. Body not executed yet. |
| `await promise` | Executes the deferred body synchronously and returns the result. Caches result. |
| `await non_promise` | Returns the value unchanged (JavaScript-compatible). |
| Nested `await` | `async fn` bodies can `await` other async calls. |
| Argument evaluation | Arguments are evaluated eagerly at the call site — errors surface immediately. |
| Second `await` on same Promise | Returns cached result without re-executing. |

### Type System

```aether
async fn f() { return 1 }
let p = f()

type(f)   // "async_function"
type(p)   // "promise"

println(f)   // <async fn(0)>
println(p)   // <promise:pending>  (before await)
             // <promise:1>        (after await)
```

### Example: Sequential Async

```aether
async fn double(x) { return x * 2 }

async fn quadruple(x) {
    let a = await double(x)
    return await double(a)
}

let result = await quadruple(5)   // 20
println(result)
```

### Example: Async with Error Handling

```aether
async fn safe_fetch(url) {
    try {
        return await http_get(url)
    } catch(e) {
        return null
    }
}

let data = await safe_fetch("https://api.example.com/data")
if (data != null) {
    println(data)
}
```

---

## Phase 2: I/O Thread Pool (planned)

The I/O thread pool enables truly concurrent I/O without changing the single-threaded interpreter.

### Architecture

```
┌────────────────────────────────────────┐
│       Main Thread (Interpreter)         │
│  Rc<T> values, single-threaded          │
│  async fn → Value::Promise (lazy)       │
│  await → blocks on channel receiver     │
└──────────┬─────────────────────────────┘
           │  IoTask (only String/f64/i64 + channel)
           ▼
┌────────────────────────────────────────┐
│         I/O Thread Pool                 │
│  N worker threads (configurable)        │
│  http_get, http_post, sleep,            │
│  read_file, write_file                  │
└────────────────────────────────────────┘
```

**Key invariant:** Worker threads never see `Value` or `Rc<T>`. Only primitive types (`String`, `f64`) cross thread boundaries via channels, preserving `Rc<T>` safety.

### Concurrent I/O

```aether
// Submit both I/O tasks before blocking on either
let p1 = http_get("https://api.example.com/users")
let p2 = http_get("https://api.example.com/posts")

// Both run in parallel on the thread pool
let results = await Promise.all([p1, p2])
println(results[0])   // users
println(results[1])   // posts
```

### Configuring Worker Count

**Environment variable** (at startup):
```bash
AETHER_IO_WORKERS=8 aether script.ae
```

**Runtime** (from Aether code):
```aether
set_workers(8)
```

**Default:** `max(num_cpu_cores - 1, 4)` workers.

### Why No Tokio?

The thread pool uses only `std::sync::mpsc` channels (already in the Rust standard library) — no new dependencies. Worker threads run blocking I/O (`reqwest::blocking`, `std::thread::sleep`) on background threads and post results back via channels. The main interpreter thread retains sole ownership of all `Value`/`Rc<T>` objects.

---

## Implementation Notes

### Promise Resolution Without Borrow Conflicts

`Expr::Await` uses `std::mem::replace` to extract the `Pending` state before calling the function body, ensuring no active `RefCell` borrow exists during execution:

```rust
// Extract Pending state, drop borrow, then execute
let pending = {
    let mut state = state_rc.borrow_mut();
    std::mem::replace(&mut *state, PromiseState::Resolved(Value::Null))
}; // borrow dropped here — safe to call exec_async_body
let result = self.exec_async_body(func, args)?;
*state_rc.borrow_mut() = PromiseState::Resolved(result.clone());
```

### exec_async_body vs call_value

`call_value(AsyncFunction, args)` wraps the call in a new Promise (lazy semantics). `exec_async_body` executes the body directly — it is only called from `Expr::Await` to resolve a Promise, never to create one.

### No Changes to Existing Eval Call Sites

The entire implementation adds ~160 lines and modifies zero existing `eval_expr` / `exec_stmt_internal` call sites. The async feature slots in as new match arms in existing dispatch functions.
