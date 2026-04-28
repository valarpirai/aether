# Advanced String Features in Aether

**Status**: ✅ Complete  
**Added**: Phase 4 Sprint 2  
**Tests**: 50 tests passing (indexing: 16, interpolation: 9, slicing: 15, spread: 9)

## Overview

Aether provides powerful string manipulation features including direct character access, template interpolation, slicing, and spread operations.

## String Indexing

Access individual characters using bracket notation:

```aether
let text = "Hello"
println(text[0])  // H
println(text[1])  // e
println(text[4])  // o
```

### Negative Indices

Use negative indices to count from the end:

```aether
let text = "Hello"
println(text[-1])  // o (last character)
println(text[-2])  // l
```

### Out of Bounds

Returns empty string for invalid indices:

```aether
let text = "Hi"
println(text[10])  // "" (empty string)
```

## String Interpolation

Embed expressions in strings using `${}` syntax:

```aether
let name = "Alice"
let age = 30
println("Hello ${name}, you are ${age} years old")
// Output: Hello Alice, you are 30 years old
```

### With Expressions

```aether
let x = 5
let y = 10
println("Sum: ${x + y}")     // Sum: 15
println("Product: ${x * y}") // Product: 50
```

### Nested Interpolation

```aether
let user = {"name": "Bob", "score": 95}
println("Player ${user["name"]} scored ${user["score"]} points")
// Output: Player Bob scored 95 points
```

## String Slicing

Extract substrings using slice syntax `[start:end]`:

```aether
let text = "Hello World"
println(text[0:5])   // "Hello"
println(text[6:11])  // "World"
```

### Open-Ended Slices

```aether
let text = "Hello World"
println(text[0:])    // "Hello World" (to end)
println(text[6:])    // "World" (from index to end)
println(text[:5])    // "Hello" (from start to index)
```

### Negative Indices in Slices

```aether
let text = "Hello World"
println(text[-5:])   // "World" (last 5 characters)
println(text[:-6])   // "Hello" (all except last 6)
```

### Full Copy

```aether
let text = "Hello"
let copy = text[:]   // Full copy
```

## String Methods

### Case Conversion

```aether
let text = "Hello World"
println(text.upper())  // HELLO WORLD
println(text.lower())  // hello world
```

### Trimming

```aether
let text = "  hello  "
println(text.trim())   // "hello"
```

### Splitting

```aether
let csv = "apple,banana,orange"
let fruits = csv.split(",")
println(fruits)  // ["apple", "banana", "orange"]
```

### Length

```aether
let text = "Hello"
println(text.length)  // 5
println(len(text))    // 5 (alternative)
```

## String Concatenation

### Using `+` Operator

```aether
let greeting = "Hello" + " " + "World"
println(greeting)  // Hello World
```

### With Interpolation

```aether
let first = "Hello"
let second = "World"
let combined = "${first} ${second}"
println(combined)  // Hello World
```

## Examples

### Example 1: Parse and Format

```aether
fn format_name(full_name) {
    let parts = full_name.split(" ")
    let first = parts[0]
    let last = parts[1]
    return "${last}, ${first}"
}

fn main() {
    println(format_name("Alice Johnson"))  // Johnson, Alice
}
```

### Example 2: Character Iteration

```aether
fn print_chars(text) {
    for i in range(0, len(text)) {
        println("Character ${i}: ${text[i]}")
    }
}

fn main() {
    print_chars("Hi!")
    // Character 0: H
    // Character 1: i
    // Character 2: !
}
```

### Example 3: String Manipulation

```aether
fn extract_domain(email) {
    let parts = email.split("@")
    if (len(parts) == 2) {
        return parts[1]
    }
    return null
}

fn main() {
    println(extract_domain("user@example.com"))  // example.com
}
```

### Example 4: Template Rendering

```aether
fn render_greeting(user) {
    let name = user["name"]
    let time = user["last_login"]
    return "Welcome back, ${name}! Last seen: ${time}"
}

fn main() {
    let user = {"name": "Alice", "last_login": "2pm"}
    println(render_greeting(user))
    // Welcome back, Alice! Last seen: 2pm
}
```

## Best Practices

### 1. Use Interpolation for Readability

```aether
// Good
let msg = "User ${name} has ${count} items"

// Less readable
let msg = "User " + name + " has " + str(count) + " items"
```

### 2. Check Bounds When Indexing

```aether
fn safe_char_at(text, index) {
    if (index < 0 or index >= len(text)) {
        return ""
    }
    return text[index]
}
```

### 3. Use Slicing for Substrings

```aether
// Get first 10 characters
let preview = long_text[:10]

// Get last 5 characters
let suffix = filename[-5:]
```

### 4. Prefer split() for Parsing

```aether
// Good
let fields = csv_line.split(",")

// Avoid manual parsing when split() works
```

## Implementation Notes

- Strings are UTF-8 encoded
- Indexing returns single-character strings
- Slicing creates new strings (not views)
- Interpolation is evaluated at runtime
- All string operations are immutable

## See Also

- [DESIGN.md](DESIGN.md) - Language specification
- [STDLIB.md](STDLIB.md) - String utility functions
- [examples/string_demo.ae](../examples/string_demo.ae) - String examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete and stable
