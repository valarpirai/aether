---
layout: default
title: "Aether — HTTP"
---

[Home](../index.html) › Language Reference › HTTP

# HTTP

Aether provides two built-in functions for making HTTP requests: `http_get()` and `http_post()`. Both return the response body as a string, or throw an error on failure.

## http_get(url [, opts])

Makes an HTTP GET request and returns the response body.

```aether
let response = http_get("https://api.example.com/users")
println(response)
```

## http_post(url, body [, opts])

Makes an HTTP POST request with the given body string and returns the response body.

```aether
let data = json_stringify({"name": "Alice", "email": "alice@example.com"})
let response = http_post("https://api.example.com/users", data)
println(response)
```

## Per-request options

Both functions accept an optional dict as the last argument to override defaults for that request:

| Key | Type | Description |
|-----|------|-------------|
| `timeout` | int | Timeout in seconds (default: 30) |
| `user_agent` | string | User-Agent header (default: `aether/0.1`) |

```aether
let data = http_get("https://slow.api.example.com/", {timeout: 60})
let resp = http_post("https://api.example.com/", payload, {user_agent: "mybot/1.0"})
```

Global defaults can be set via `AETHER_HTTP_TIMEOUT` and `AETHER_HTTP_USER_AGENT` env vars.

## Async mode

When `set_workers(n)` is active (or `AETHER_IO_WORKERS` is set), both functions return a Promise and execute on the I/O thread pool. Use `await` or `Promise.all` to run requests concurrently:

```aether
set_workers(4)

let p1 = http_get("https://api.example.com/users")
let p2 = http_get("https://api.example.com/posts")

let results = await Promise.all([p1, p2])
println(results[0])
println(results[1])
```

Without `set_workers`, requests run synchronously and block the main thread.

## Examples

### Fetch and parse JSON

```aether
fn fetch_user(id) {
    try {
        let response = http_get("https://api.example.com/users/${id}")
        return json_parse(response)
    } catch(e) {
        println("Failed: ${e}")
        return null
    }
}

fn main() {
    let user = fetch_user(123)
    if (user != null) {
        println(user["name"])
    }
}
```

### Create a resource

```aether
fn main() {
    let payload = json_stringify({"title": "Hello", "body": "World", "userId": 1})
    let response = http_post("https://jsonplaceholder.typicode.com/posts", payload)
    let post = json_parse(response)
    println("Created ID: ${post["id"]}")
}
```

### Retry on failure

```aether
fn get_with_retry(url, attempts) {
    for i in range(0, attempts) {
        try {
            return http_get(url)
        } catch(e) {
            if (i < attempts - 1) { sleep(1) }
        }
    }
    throw "Failed after ${attempts} attempts"
}
```

## Error handling

HTTP functions throw on network failure, invalid URL, DNS errors, timeout, and HTTP error status codes. Always wrap in `try/catch` for production code:

```aether
try {
    let response = http_get(url)
    process(response)
} catch(e) {
    println("Request failed: ${e}")
}
```

## Limitations

- GET and POST only — no PUT, PATCH, or DELETE
- No custom request headers beyond `User-Agent`
- No access to response status code or headers
- No built-in authentication — pass API keys in the URL or body
- Redirects are followed automatically

## Related

- [Async/Await](ASYNC.html) — `await`, `Promise.all`, `.then()` for concurrent requests
- [JSON](JSON.html) — parse response bodies as structured data
- [Configuration](CONFIGURATION.html) — `set_workers`, `AETHER_HTTP_TIMEOUT`, `AETHER_HTTP_USER_AGENT`

---
[← JSON](JSON.html) &nbsp;&nbsp; [Time →](TIME.html)
