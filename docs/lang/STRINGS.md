# Strings

Strings in Aether are UTF-8 sequences. They support indexing, slicing, interpolation, multi-line literals, and a set of built-in methods.

## Literals

```aether
let s = "Hello, World"
let s = 'single quotes work too'
```

Multi-line strings use triple quotes — content is taken verbatim, including newlines and embedded quotes:

```aether
let body = """
{
    "name": "Alice",
    "role": "admin"
}
"""
```

## Concatenation

```aether
let greeting = "Hello" + " " + "World"   // Hello World
```

## Interpolation

Embed any expression inside `${}`:

```aether
let name = "Alice"
let age = 30
println("${name} is ${age} years old")       // Alice is 30 years old
println("Sum: ${1 + 2 + 3}")                 // Sum: 6
println("Score: ${user["score"]} points")    // dict access in interpolation
```

## Indexing

Access a character by position. Negative indices count from the end. Out-of-bounds returns an empty string.

```aether
let s = "Hello"
s[0]    // "H"
s[4]    // "o"
s[-1]   // "o" (last character)
s[-2]   // "l"
s[99]   // "" (out of bounds)
```

## Slicing

Extract a substring with `[start:end]`. Either end is optional (defaults to start/end of string).

```aether
let s = "Hello World"
s[0:5]    // "Hello"
s[6:11]   // "World"
s[6:]     // "World"  (to end)
s[:5]     // "Hello"  (from start)
s[:]      // "Hello World"  (full copy)
s[-5:]    // "World"  (last 5 chars)
s[:-6]    // "Hello"  (all but last 6)
```

## Methods

| Method | Returns |
|--------|---------|
| `s.upper()` | uppercase copy |
| `s.lower()` | lowercase copy |
| `s.trim()` | whitespace stripped from both ends |
| `s.split(sep)` | array of substrings |
| `s.length` | character count (also `len(s)`) |

```aether
"Hello World".upper()      // "HELLO WORLD"
"  hello  ".trim()         // "hello"
"a,b,c".split(",")         // ["a", "b", "c"]
"Hello".length             // 5
```

## Examples

### Extract domain from email

```aether
fn domain(email) {
    let parts = email.split("@")
    return parts[1]
}
println(domain("user@example.com"))  // example.com
```

### Reverse a string

```aether
fn reverse(s) {
    let result = ""
    for i in range(len(s) - 1, -1) {
        result = result + s[i]
    }
    return result
}
```

### Format a name

```aether
fn last_first(full) {
    let parts = full.split(" ")
    return "${parts[1]}, ${parts[0]}"
}
println(last_first("Alice Johnson"))  // Johnson, Alice
```

## Limitations

- Strings are immutable — all operations return new strings
- Indexing returns single-character strings, not character codes
- No regex support built-in
