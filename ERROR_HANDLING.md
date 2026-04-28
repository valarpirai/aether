---
layout: default
title: Error Handling in Aether
---

# Error Handling in Aether

**Status**: ✅ Complete  
**Added**: Phase 4 Sprint 4  
**Tests**: 10 tests passing

## Overview

Aether provides structured error handling through `try/catch/throw` statements, allowing you to gracefully handle exceptional conditions and recover from errors.

## Table of Contents
- [Throwing Errors](#throwing-errors)
- [Catching Errors](#catching-errors)
- [Error Propagation](#error-propagation)
- [Error Values](#error-values)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Comparison with Other Languages](#comparison-with-other-languages)

## Throwing Errors

Use `throw` to raise an error:

```aether
throw "Something went wrong"
```

### Throw Any Value

You can throw any Aether value as an error:

```aether
throw "error message"        // String
throw 404                    // Integer
throw {"code": 500}          // Dict
throw ["error", "details"]   // Array
```

### In Functions

```aether
fn divide(a, b) {
    if (b == 0) {
        throw "Division by zero"
    }
    return a / b
}
```

## Catching Errors

Use `try/catch` to handle errors:

```aether
try {
    // Code that might throw
    risky_operation()
} catch(e) {
    // Handle the error
    println("Error:", e)
}
```

### Basic Syntax

```aether
try {
    <statements>
} catch(<error_variable>) {
    <error_handling_statements>
}
```

The error variable (`e` in examples) captures the thrown value.

### Example

```aether
let result = 0
try {
    result = divide(10, 0)
} catch(e) {
    println("Caught error:", e)
    result = -1
}
println("Result:", result)  // Result: -1
```

## Error Propagation

Errors propagate up the call stack until caught:

```aether
fn level3() {
    throw "error at level 3"
}

fn level2() {
    level3()  // Error propagates through here
}

fn level1() {
    try {
        level2()  // Error propagates through here too
    } catch(e) {
        println("Caught:", e)  // Finally caught here
    }
}

fn main() {
    level1()
}
```

### Uncaught Errors

If an error is not caught, it terminates the program:

```aether
fn main() {
    throw "Fatal error"  // Program terminates
    println("Never reached")
}
```

## Error Values

### String Errors (Recommended)

Most errors should be descriptive strings:

```aether
throw "File not found: config.txt"
throw "Invalid input: expected number, got string"
throw "Connection timeout after 30 seconds"
```

### Structured Errors

Use dicts for rich error information:

```aether
fn validate_age(age) {
    if (age < 0) {
        throw {
            "type": "ValidationError",
            "field": "age",
            "message": "Age cannot be negative",
            "value": age
        }
    }
}

try {
    validate_age(-5)
} catch(e) {
    println("Error type:", e["type"])
    println("Message:", e["message"])
}
```

### Error Codes

Use integers for error codes:

```aether
fn http_request(url) {
    // ...
    if (status != 200) {
        throw status  // 404, 500, etc.
    }
}

try {
    http_request("http://example.com")
} catch(code) {
    if (code == 404) {
        println("Not found")
    } else {
        println("Error code:", code)
    }
}
```

## Best Practices

### 1. Be Specific with Error Messages

```aether
// Good
throw "Invalid email format: missing @ symbol"

// Less helpful
throw "Bad input"
```

### 2. Catch at the Right Level

Catch errors where you can meaningfully handle them:

```aether
fn main() {
    try {
        process_file("data.txt")
    } catch(e) {
        // Can show user-friendly message here
        println("Failed to process file:", e)
        // Maybe log to file, show UI dialog, etc.
    }
}
```

### 3. Re-throw When Needed

If you can't handle an error, let it propagate:

```aether
fn process() {
    try {
        risky_operation()
    } catch(e) {
        log_error(e)
        throw e  // Re-throw for caller to handle
    }
}
```

### 4. Clean Up Resources

Use try/catch to ensure cleanup:

```aether
fn process_file(filename) {
    let file = open_file(filename)
    try {
        // Process the file
        let data = read_file(file)
        transform(data)
    } catch(e) {
        println("Processing failed:", e)
        close_file(file)  // Clean up
        throw e
    }
    close_file(file)  // Normal cleanup
}
```

### 5. Validate Early

Throw errors as soon as problems are detected:

```aether
fn withdraw(account, amount) {
    if (amount <= 0) {
        throw "Amount must be positive"
    }
    if (amount > account.balance) {
        throw "Insufficient funds"
    }
    // Proceed with withdrawal
    account.balance = account.balance - amount
}
```

## Examples

### Example 1: Safe Division

```aether
fn safe_divide(a, b) {
    try {
        if (b == 0) {
            throw "Cannot divide by zero"
        }
        return a / b
    } catch(e) {
        println("Error:", e)
        return null
    }
}

fn main() {
    println(safe_divide(10, 2))  // 5
    println(safe_divide(10, 0))  // Error: Cannot divide by zero, null
}
```

### Example 2: Input Validation

```aether
fn create_user(name, age) {
    if (len(name) == 0) {
        throw "Name cannot be empty"
    }
    if (age < 18) {
        throw "User must be 18 or older"
    }
    return {"name": name, "age": age}
}

fn main() {
    try {
        let user1 = create_user("Alice", 25)
        println("Created:", user1)
        
        let user2 = create_user("", 30)  // Throws error
    } catch(e) {
        println("Validation failed:", e)
    }
}
```

### Example 3: Multiple Operations

```aether
fn process_data(data) {
    try {
        let parsed = parse_json(data)
        let validated = validate(parsed)
        let transformed = transform(validated)
        return transformed
    } catch(e) {
        println("Processing failed at some step:", e)
        return null
    }
}
```

### Example 4: Nested Try/Catch

```aether
fn complex_operation() {
    try {
        // Outer operation
        let data = load_data()
        
        try {
            // Inner operation that might fail
            let result = risky_transform(data)
            return result
        } catch(e) {
            // Handle inner error specifically
            println("Transform failed, using default")
            return default_value()
        }
    } catch(e) {
        // Handle outer error
        println("Fatal error:", e)
        return null
    }
}
```

### Example 5: Error Recovery

```aether
fn fetch_data() {
    let sources = ["primary.db", "backup.db", "cache.db"]
    
    for source in sources {
        try {
            let data = read_file(source)
            println("Loaded from:", source)
            return data
        } catch(e) {
            println("Failed to load from", source, "-", e)
            // Try next source
        }
    }
    
    throw "All data sources failed"
}

fn main() {
    try {
        let data = fetch_data()
        println("Got data:", data)
    } catch(e) {
        println("Could not fetch data:", e)
    }
}
```

## Comparison with Other Languages

### Python
```python
# Python
try:
    result = divide(10, 0)
except Exception as e:
    print("Error:", e)
```

```aether
// Aether
try {
    result = divide(10, 0)
} catch(e) {
    println("Error:", e)
}
```

### JavaScript
```javascript
// JavaScript
try {
    result = divide(10, 0);
} catch(e) {
    console.log("Error:", e);
}
```

```aether
// Aether
try {
    result = divide(10, 0)
} catch(e) {
    println("Error:", e)
}
```

### Java
```java
// Java
try {
    result = divide(10, 0);
} catch (Exception e) {
    System.out.println("Error: " + e);
}
```

```aether
// Aether
try {
    result = divide(10, 0)
} catch(e) {
    println("Error:", e)
}
```

## Implementation Details

### Error Type
- Errors in Aether are runtime values (any type)
- No special error class or type hierarchy
- Simple and flexible

### Stack Unwinding
- When an error is thrown, the stack unwinds
- Execution jumps to the nearest enclosing catch block
- If no catch block exists, program terminates

### Performance
- Try/catch has minimal overhead when no error is thrown
- Throwing an error involves stack unwinding (relatively expensive)
- Design code to avoid throwing in hot loops

## Limitations

**No Finally Block**: Aether doesn't have `finally` for guaranteed cleanup. Use explicit cleanup in both try and catch blocks.

**No Multiple Catch Blocks**: You can't catch different error types separately. Use conditional logic in the catch block instead:

```aether
try {
    operation()
} catch(e) {
    if (type(e) == "dict" and e.contains("type")) {
        // Handle structured error
    } else {
        // Handle other errors
    }
}
```

**No Stack Traces**: Error messages don't include stack traces. Include context in error messages.

## See Also

- [DESIGN.md](DESIGN.html) - Language specification
- [INTERPRETER.md](INTERPRETER.html) - Runtime implementation  
- [examples/error_handling.ae](../examples/error_handling.ae) - Error handling examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete and stable
