---
layout: default
title: HTTP Functions in Aether
---

# HTTP Functions in Aether

**Status**: ✅ Complete  
**Added**: Phase 5  
**Tests**: 5 tests (ignored - require network)  
**Backend**: reqwest (blocking)

## Overview

Aether provides basic HTTP client functionality through `http_get()` and `http_post()` built-in functions for making web requests.

## Functions

### http_get(url)

Make a GET request to the specified URL:

```aether
let response = http_get("https://api.example.com/users")
println(response)
```

**Returns**: Response body as string  
**Throws**: Error on network failure, invalid URL, or HTTP errors

### http_post(url, body)

Make a POST request with a JSON body:

```aether
let data = {"name": "Alice", "email": "alice@example.com"}
let json_body = json_stringify(data)
let response = http_post("https://api.example.com/users", json_body)
println(response)
```

**Arguments**:
- `url` - Target URL (must be valid HTTP/HTTPS)
- `body` - Request body as string (typically JSON)

**Returns**: Response body as string  
**Content-Type**: Automatically set to `application/json`

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
- Invalid URL format
- Network connectivity issues
- DNS resolution failures
- HTTP 4xx/5xx status codes
- Timeout (no explicit timeout setting yet)

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

- **Blocking Only**: Requests block until complete (no async)
- **No Custom Headers**: Can't set request headers
- **No Authentication**: No built-in auth support (Basic, Bearer, etc.)
- **No Timeout Control**: Can't set request timeout
- **POST Only**: No PUT, PATCH, DELETE methods
- **JSON Content-Type Only**: POST always sends `application/json`
- **No Query Parameters**: Must manually construct URLs
- **No Response Headers**: Can't access response headers or status code
- **No Redirect Control**: Follows redirects automatically

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

- [JSON.md](JSON.html) - JSON parsing for HTTP responses
- [ERROR_HANDLING.md](ERROR_HANDLING.html) - Error handling patterns
- [TIME.md](TIME.html) - Sleep for rate limiting
- [examples/http_demo.ae](../examples/http_demo.ae) - HTTP examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete but basic (blocking only, limited features)
