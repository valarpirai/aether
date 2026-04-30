# Async / Await

Aether supports async programming with `async fn`, `await`, and a configurable I/O thread pool for concurrent I/O operations.

## async fn

Declaring a function `async` makes it lazy — calling it returns a Promise immediately without executing the body.

```aether
async fn fetch_user(id) {
    let data = await http_get("/users/" + id)
    return data
}

let p = fetch_user(42)       // returns a Promise, body not yet run
let user = await fetch_user(42)  // executes body and returns result
```

Anonymous async functions work the same way:

```aether
let double = async fn(x) { return x * 2 }
let result = await double(10)  // 20
```

## await

`await` resolves a Promise by executing its deferred body and returning the result. The result is cached — awaiting the same Promise twice only runs the body once.

`await` on a non-Promise value is a no-op and returns the value unchanged:

```aether
let x = await 42   // x = 42
```

## Promise combinators

### Promise.all(array)

Runs all I/O Promises in the array concurrently and returns an array of results in the same order. Fails fast if any Promise throws.

```aether
set_workers(4)

let p1 = http_get("https://api.example.com/users")
let p2 = http_get("https://api.example.com/posts")

let results = await Promise.all([p1, p2])
println(results[0])  // users response
println(results[1])  // posts response
```

### Promise.race(array)

Returns the result of the first Promise to complete. Remaining Promises are discarded.

```aether
set_workers(4)

let p1 = http_get("https://primary.example.com/data")
let p2 = http_get("https://backup.example.com/data")

let fastest = await Promise.race([p1, p2])
println(fastest)
```

### Promise.allSettled(array)

Runs all Promises and returns an array of result dicts — one per input — regardless of success or failure. Each dict has `"status"` (`"fulfilled"` or `"rejected"`) and either `"value"` or `"reason"`.

```aether
set_workers(4)

let promises = [
    http_get("https://api.example.com/a"),
    http_get("https://api.example.com/b"),
    http_get("https://will-fail.example.com/c")
]

let results = await Promise.allSettled(promises)
for r in results {
    if (r["status"] == "fulfilled") {
        println("OK:", r["value"])
    } else {
        println("Failed:", r["reason"])
    }
}
```

## I/O thread pool

By default, I/O functions like `http_get`, `http_post`, `sleep`, `read_file`, and `write_file` run synchronously and block the main thread. To run them concurrently, enable the I/O thread pool:

```aether
set_workers(4)   // or set AETHER_IO_WORKERS=4 env var
```

Once workers are active, calling any I/O function submits the task to the pool and immediately returns a Promise. Use `await` or `Promise.all` to collect results.

**Default worker count:** `max(cpu_count - 1, 4)` when `AETHER_IO_WORKERS` is set.

## Examples

### Sequential async

```aether
async fn double(x) { return x * 2 }

async fn quadruple(x) {
    let a = await double(x)
    return await double(a)
}

fn main() {
    println(await quadruple(5))  // 20
}
```

### Async with error handling

```aether
async fn safe_fetch(url) {
    try {
        return await http_get(url)
    } catch(e) {
        return null
    }
}

fn main() {
    let data = await safe_fetch("https://api.example.com/data")
    if (data != null) {
        println(data)
    }
}
```

### Concurrent file reads

```aether
fn main() {
    set_workers(4)

    let files = ["a.txt", "b.txt", "c.txt"]
    let promises = []
    for f in files {
        promises.push(read_file(f))
    }

    let contents = await Promise.all(promises)
    for c in contents {
        println(c)
    }
}
```

## Behavior reference

| Situation | What happens |
|-----------|-------------|
| Call `async fn` | Returns a pending Promise; body not executed |
| `await` a pending Promise | Executes the body, caches the result |
| `await` a resolved Promise | Returns the cached result immediately |
| `await` a non-Promise | Returns the value unchanged |
| `Promise.all` | All I/O runs concurrently; waits for all |
| `Promise.race` | Returns the first to complete |
| `Promise.allSettled` | Waits for all; never throws |
