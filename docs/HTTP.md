# HTTP Functions in Aether

**Status**: ✅ Complete  
**Added**: Phase 5  
**Tests**: 9 tests passing (0 ignored)  
**Backend**: reqwest (blocking or async via I/O pool)

## Overview

Aether provides basic HTTP client functionality through `http_get()` and `http_post()` built-in functions for making web requests.

## Functions

### http_get(url [, opts])

Make a GET request to the specified URL:

```aether
let response = http_get("https://api.example.com/users")
println(response)
```

**Arguments**:
- `url` — Target URL (must be valid HTTP/HTTPS)
- `opts` *(optional)* — Config dict with per-request overrides

**Returns**: Response body as string  
**Throws**: Error on network failure, invalid URL, or HTTP errors

### http_post(url, body [, opts])

Make a POST request:

```aether
let data = {"name": "Alice", "email": "alice@example.com"}
let json_body = json_stringify(data)
let response = http_post("https://api.example.com/users", json_body)
println(response)
```

**Arguments**:
- `url` — Target URL (must be valid HTTP/HTTPS)
- `body` — Request body as string (typically JSON)
- `opts` *(optional)* — Config dict with per-request overrides

**Returns**: Response body as string

## Per-request Configuration

Both functions accept an optional config dict as their last argument. Dict keys override the global env-var defaults for that single request:

| Key | Type | Description |
|-----|------|-------------|
| `timeout` | int (seconds) | Request timeout. Overrides `AETHER_HTTP_TIMEOUT`. |
| `user_agent` | string | `User-Agent` header. Overrides `AETHER_HTTP_USER_AGENT`. |

```aether
// Custom timeout for a slow endpoint
let data = http_get("https://slow.api.example.com/", {timeout: 60})

// Custom user-agent
let resp = http_get("https://api.example.com/", {user_agent: "mybot/1.0"})

// Both options together
let result = http_post("https://api.example.com/submit", payload, {
    timeout: 10,
    user_agent: "myapp/2.0"
})
```

**Global defaults** (apply when no per-request opt is given):
- `AETHER_HTTP_TIMEOUT` — timeout in seconds (default: 30)
- `AETHER_HTTP_USER_AGENT` — User-Agent string (default: `aether/0.1`)

See [CONFIGURATION.md](CONFIGURATION.md) for details.

## Async Mode

When `AETHER_IO_WORKERS` is set (or `set_workers(n)` is called), both functions return a `Promise` and execute on the I/O thread pool:

```aether
set_workers(4)

// Both requests start concurrently
let p1 = http_get("https://api.example.com/users")
let p2 = http_get("https://api.example.com/posts", {timeout: 5})

let results = await Promise.all([p1, p2])
println(results[0])
println(results[1])
```

Without `set_workers`, requests run synchronously and block the main thread.

See [ASYNC.md](ASYNC.md) for the I/O thread pool documentation.

## Examples

### Example 1: Fetch Data

```aether
fn fetch_user(user_id) {
    let url = "https://api.example.com/users/${user_id}"
    try {
        let response = http_get(url)
        let user = json_parse(response)
        return user
    } catch(e) {
        println("Failed to fetch user:", e)
        return null
    }
}

fn main() {
    let user = fetch_user(123)
    if (user != null) {
        println("User name:", user["name"])
    }
}
```

### Example 2: Create Resource

```aether
fn create_post(title, body) {
    let data = {
        "title": title,
        "body": body,
        "userId": 1
    }
    let json = json_stringify(data)
    
    try {
        let response = http_post("https://jsonplaceholder.typicode.com/posts", json)
        let created = json_parse(response)
        println("Created post with ID:", created["id"])
        return created
    } catch(e) {
        println("Failed to create post:", e)
        return null
    }
}

fn main() {
    create_post("My Title", "This is the content")
}
```

### Example 3: Weather API

```aether
fn get_weather(city) {
    let url = "https://api.weather.com/v1/current/${city}"
    try {
        let response = http_get(url)
        let data = json_parse(response)
        return {
            "temp": data["temperature"],
            "condition": data["condition"],
            "humidity": data["humidity"]
        }
    } catch(e) {
        println("Weather fetch failed:", e)
        return null
    }
}

fn main() {
    let weather = get_weather("london")
    if (weather != null) {
        println("Temperature: ${weather["temp"]}°C")
        println("Condition: ${weather["condition"]}")
    }
}
```

### Example 4: Multiple Requests

```aether
fn fetch_all_users(user_ids) {
    let users = []
    for id in user_ids {
        let url = "https://api.example.com/users/${id}"
        try {
            let response = http_get(url)
            let user = json_parse(response)
            users.push(user)
        } catch(e) {
            println("Failed to fetch user ${id}:", e)
        }
    }
    return users
}

fn main() {
    let ids = [1, 2, 3]
    let users = fetch_all_users(ids)
    println("Fetched ${len(users)} users")
}
```

### Example 5: Retry Logic

```aether
fn http_get_with_retry(url, max_attempts) {
    for attempt in range(1, max_attempts + 1) {
        try {
            println("Attempt ${attempt}...")
            return http_get(url)
        } catch(e) {
            println("Failed: ${e}")
            if (attempt < max_attempts) {
                sleep(1)  // Wait before retry
            }
        }
    }
    throw "Failed after ${max_attempts} attempts"
}

fn main() {
    try {
        let data = http_get_with_retry("https://api.example.com/data", 3)
        println("Success:", data)
    } catch(e) {
        println("All attempts failed:", e)
    }
}
```

## Error Handling

HTTP functions throw errors for various failures:

```aether
try {
    let response = http_get("https://invalid-url")
} catch(e) {
    // Handle network errors, DNS failures, timeouts, etc.
    println("Request failed:", e)
}
```

**Common Errors**:
- Invalid URL format or unsupported scheme
- Network connectivity issues
- DNS resolution failures
- HTTP 4xx/5xx status codes
- Request timeout (configurable via `timeout` key or `AETHER_HTTP_TIMEOUT`)

## Best Practices

### 1. Always Use Error Handling

```aether
// Good
try {
    let data = http_get(url)
    process(data)
} catch(e) {
    println("Request failed:", e)
    // Handle gracefully
}

// Bad - uncaught errors terminate program
let data = http_get(url)
```

### 2. Parse JSON Responses

```aether
let response = http_get(api_url)
let data = json_parse(response)  // Parse JSON string
```

### 3. Use String Interpolation for URLs

```aether
let user_id = 123
let url = "https://api.example.com/users/${user_id}"
let response = http_get(url)
```

### 4. Validate Before Sending

```aether
fn safe_post(url, data) {
    if (len(url) == 0) {
        throw "URL cannot be empty"
    }
    if (data == null) {
        throw "Data cannot be null"
    }
    return http_post(url, json_stringify(data))
}
```

### 5. Rate Limit Requests

```aether
fn rate_limited_requests(urls, delay_sec) {
    let results = []
    for url in urls {
        let response = http_get(url)
        results.push(response)
        sleep(delay_sec)  // Don't overwhelm server
    }
    return results
}
```

## Limitations

### Current Limitations

- **Sync by default**: Without `set_workers`, requests block the main thread. Use the I/O pool for concurrency.
- **No Custom Headers**: Can't set arbitrary request headers (only `User-Agent`)
- **No Authentication**: No built-in auth support (Basic, Bearer, etc.)
- **GET and POST only**: No PUT, PATCH, DELETE methods
- **No Response Headers**: Can't access response headers or status code
- **No Redirect Control**: Follows redirects automatically
- **No Query Parameter helpers**: Must manually construct URL query strings

### Workarounds

**Authentication**: Include in URL or use services with API keys in query params:
```aether
let url = "https://api.example.com/data?api_key=YOUR_KEY"
```

**Query Parameters**: Build URL manually:
```aether
fn build_url(base, params) {
    let query = ""
    let keys = params.keys()
    for key in keys {
        if (len(query) > 0) {
            query = query + "&"
        }
        query = query + key + "=" + str(params[key])
    }
    return base + "?" + query
}

let url = build_url("https://api.example.com/search", {
    "q": "aether",
    "limit": 10
})
```

## Complete Example

See [examples/http_demo.ae](../examples/http_demo.ae) for a full working example using JSONPlaceholder API.

## See Also

- [JSON.md](JSON.md) - JSON parsing for HTTP responses
- [ERROR_HANDLING.md](ERROR_HANDLING.md) - Error handling patterns
- [TIME.md](TIME.md) - Sleep for rate limiting
- [examples/http_demo.ae](../examples/http_demo.ae) - HTTP examples

---

**Last Updated**: 2026-04-29  
**Status**: Complete — sync and async modes, per-request config dict, configurable timeout and User-Agent
