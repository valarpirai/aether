---
layout: default
title: "Aether — Error Handling"
---

[Home](../index.md) › Language Reference › Error Handling

# Error Handling

Aether uses `try/catch/throw` for structured error handling. Errors propagate up the call stack until caught; an uncaught error terminates the program.

## throw

Raises an error. You can throw any value — a string is the most common choice.

```aether
throw "Something went wrong"
throw {"code": 404, "message": "Not found"}
throw 500
```

## try / catch

Executes the `try` block. If any statement throws, execution jumps to the `catch` block with the thrown value bound to the catch variable.

```aether
try {
    let result = divide(10, 0)
} catch(e) {
    println("Error:", e)
}
```

### Error object properties

When a runtime error is caught (including errors thrown by the interpreter itself), the catch variable has two properties:

```aether
try {
    throw "oops"
} catch(e) {
    println(e.message)      // "oops"
    println(e.stack_trace)  // call stack at the point of the throw
}
```

`e.stack_trace` includes function names, filenames, and line numbers:

```
Error: division by zero
  at divide (script.ae:2)
  at calculate (script.ae:5)
```

## finally

The `finally` block runs after `try` and `catch` regardless of whether an error was thrown, caught, or re-thrown — including when `return` is used inside `try`.

```aether
fn process(filename) {
    let data = read_file(filename)
    try {
        transform(data)
    } catch(e) {
        println("Failed:", e)
        throw e
    } finally {
        println("Always runs")
    }
}
```

`finally` is optional.

## Error propagation

Errors propagate up through callers until caught. If never caught, the program terminates with the error message.

```aether
fn level3() { throw "deep error" }
fn level2() { level3() }
fn level1() {
    try {
        level2()
    } catch(e) {
        println("Caught at level 1:", e)
    }
}
```

## Structured errors

Throw a dict when you need rich error information:

```aether
fn validate_age(age) {
    if (age < 0) {
        throw {"type": "ValidationError", "field": "age", "value": age}
    }
}

try {
    validate_age(-5)
} catch(e) {
    if (type(e) == "dict") {
        println("Validation failed on field:", e["field"])
    } else {
        println(e)
    }
}
```

## Examples

### Safe division

```aether
fn safe_divide(a, b) {
    if (b == 0) { throw "Cannot divide by zero" }
    return a / b
}

fn main() {
    try {
        println(safe_divide(10, 2))  // 5
        println(safe_divide(10, 0))  // throws
    } catch(e) {
        println("Error:", e)
    }
}
```

### Fallback across multiple sources

```aether
fn load_data(sources) {
    for source in sources {
        try {
            return read_file(source)
        } catch(e) {
            println("Skipping ${source}: ${e}")
        }
    }
    throw "All sources failed"
}
```

## Limitations

- One `catch` block per `try` — use conditionals inside `catch` to distinguish error types
- No custom exception classes or type hierarchy

---
[← Format](FORMAT.md) &nbsp;&nbsp; [Structs →](STRUCT.md)
