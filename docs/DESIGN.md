# Aether Programming Language Design Document

## Overview

Aether is a general-purpose, dynamically typed programming language with automatic memory management. It combines familiar C-like syntax with modern features: async/await, null safety, structs, iterators, and a self-hosted standard library.

## Core Philosophy

- **General-purpose**: suitable for scripting, data processing, and concurrent I/O
- **Dynamic typing**: types are checked at runtime for flexibility
- **Automatic memory management**: Rc-based garbage collection
- **Interpreted**: direct execution without a compilation step
- **Clean syntax**: C-like blocks without semicolons

---

## Language Specifications

### 1. Execution Model

- **Type**: interpreted (tree-walking interpreter)
- **File extension**: `.ae`
- **Entry point**: required `main()` function
- **REPL**: interactive mode with history and tab-completion

### 2. Syntax

#### 2.1 General Structure

- **Block delimiters**: `{ }`
- **Statement termination**: no semicolons
- **Comments**: `// single-line` and `/* multi-line */`

#### 2.2 Program Structure

```aether
fn main() {
    println("Hello, Aether!")
}
```

---

### 3. Type System

#### 3.1 Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `int` | 64-bit integer | `42`, `-17`, `0` |
| `float` | 64-bit float | `3.14`, `-0.5`, `2.0` |
| `string` | UTF-8 immutable text | `"hello"`, `"世界"` |
| `bool` | Boolean | `true`, `false` |
| `null` | Absence of value | `null` |

#### 3.2 Collection Types

| Type | Description | Example |
|------|-------------|---------|
| `array` | Ordered, mutable | `[1, 2, 3]` |
| `dict` | Key-value, insertion-ordered | `{"name": "Alice"}` |
| `set` | Unique, unordered (hashable elements only) | `set([1, 2, 3])` |

Sets are created with `set(array)`. Only `int`, `float`, `string`, `bool`, and `null` can be set elements.

#### 3.3 Function Types

- **Named function**: `fn name(params) { ... }`
- **Async function**: `async fn name(params) { ... }` — returns a Promise
- **Function expression**: `fn(params) { ... }` — anonymous, assignable

#### 3.4 Special Types

- **Promise** — result of an `async fn` or I/O builtin
- **Iterator** — returned by `lines_iter()` or custom `has_next`/`next` structs
- **Struct instance** — user-defined type with fields and methods

---

### 4. Variables and Scoping

```aether
let x = 10
let name = "Aether"
let items = [1, 2, 3]
```

- **Block scope**: variables are scoped to their enclosing `{}`
- **Mutable**: variables can be reassigned without `let`
- **No hoisting**: variables must be declared before use

```aether
let global = 100

fn example() {
    let x = 10
    if (x > 5) {
        let y = 20   // only visible inside this block
    }
    // y is not accessible here
}
```

---

### 5. Functions

#### 5.1 Named Functions

```aether
fn add(a, b) {
    return a + b
}
```

#### 5.2 Optional Parameters

```aether
fn greet(name, greeting = "Hello") {
    return greeting + ", " + name + "!"
}
```

#### 5.3 Function Expressions

```aether
let double = fn(x) { return x * 2 }
let result = double(5)   // 10
```

#### 5.4 Closures

```aether
fn make_counter() {
    let count = 0
    return fn() {
        count = count + 1
        return count
    }
}
let counter = make_counter()
println(counter())  // 1
println(counter())  // 2
```

#### 5.5 Async Functions

```aether
async fn fetch(url) {
    return http_get(url)
}

fn main() {
    set_workers(4)
    let p = fetch("https://example.com/api")
    let body = await p
    println(body)
}
```

---

### 6. Operators

#### 6.1 Arithmetic

| Operator | Description |
|----------|-------------|
| `+` | Addition (also string concat) |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |

#### 6.2 Comparison

| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` `>` `<=` `>=` | Ordered comparison |

#### 6.3 Logical

| Operator | Description |
|----------|-------------|
| `&&` | Logical AND |
| `\|\|` | Logical OR |
| `!` | Logical NOT |

#### 6.4 Assignment

| Operator | Description |
|----------|-------------|
| `=` | Assignment |
| `+=` `-=` `*=` `/=` | Compound assignment |

#### 6.5 Null Safety

| Operator | Description |
|----------|-------------|
| `??` | Null coalescing — returns left if non-null, else right |
| `?.` | Optional chaining — returns null if left is null |

```aether
let name = null
println(name ?? "anonymous")       // anonymous
println(name?.upper() ?? "NONE")   // NONE
```

#### 6.6 Spread

```aether
let a = [1, 2, 3]
let b = [...a, 4, 5]   // [1, 2, 3, 4, 5]
```

---

### 7. Strings

#### 7.1 String Literals

```aether
let s = "Hello, World!"
let interp = "Sum: ${1 + 2}"      // "Sum: 3"
let raw = """
    multi-line
    string
"""
```

#### 7.2 String Indexing and Slicing

```aether
let s = "hello"
println(s[0])      // "h"
println(s[1:3])    // "el"
println(s[-1])     // "o"
```

#### 7.3 String Methods

| Method | Description |
|--------|-------------|
| `s.upper()` | Uppercase |
| `s.lower()` | Lowercase |
| `s.trim()` | Strip whitespace |
| `s.split(sep)` | Split into array |
| `s.contains(sub)` | True if contains substring |
| `s.index_of(sub)` | First index of substring (-1 if absent) |
| `s.replace(old, new)` | Replace all occurrences |
| `s.length` | Character count |

#### 7.4 Escape Sequences

| Sequence | Character |
|----------|-----------|
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Backslash |
| `\"` | Double quote |

---

### 8. Control Flow

#### 8.1 Conditionals

```aether
if (condition) {
    // ...
} else if (other) {
    // ...
} else {
    // ...
}
```

#### 8.2 While Loop

```aether
while (condition) {
    if (done) { break }
    if (skip) { continue }
}
```

#### 8.3 For Loop

```aether
// Range-based
for i in range(0, 10) { ... }

// Array
for item in array { ... }

// Dict (key, value)
for key, value in dict { ... }

// String (character by character)
for ch in "hello" { ... }

// Set
for elem in my_set { ... }

// Iterator
for line in lines_iter("file.txt") { ... }
```

#### 8.4 Labeled Break / Continue

```aether
outer: for i in range(3) {
    for j in range(3) {
        if (i == 1 && j == 1) {
            break outer
        }
    }
}
```

---

### 9. Error Handling

```aether
try {
    let data = json_parse(raw)
    process(data)
} catch (e) {
    println("Error:", e.message)
    println("Stack:", e.stack_trace)
} finally {
    cleanup()   // always runs
}
```

- `throw value` — throws any value as an error
- `e.message` — the error message string
- `e.stack_trace` — array of stack frame strings

---

### 10. Null Safety

```aether
let user = get_user()
let name = user?.name ?? "anonymous"
let upper = user?.name?.upper() ?? "UNKNOWN"
```

`?.` short-circuits to `null` if the left side is `null` (no exception thrown).

---

### 11. Structs

```aether
struct Point {
    x
    y

    fn distance(self) {
        return (self.x * self.x + self.y * self.y)
    }
}

fn main() {
    let p = Point(3, 4)
    println(p.x)           // 3
    println(p.distance())  // 25
    p.x = 10              // mutable field
}
```

---

### 12. Async / Await

```aether
async fn load(url) {
    return http_get(url)
}

fn main() {
    set_workers(4)
    let p1 = load("https://api.example.com/a")
    let p2 = load("https://api.example.com/b")
    let results = await Promise.all([p1, p2])
    println(results[0])
    println(results[1])
}
```

- `async fn` returns a Promise
- `await expr` blocks until the Promise resolves
- `Promise.all([...])` waits for all Promises concurrently

---

### 13. Event Loop

```aether
fn main() {
    set_workers(2)
    let p = sleep(0.1)
    on_ready(p, fn(v) {
        println("sleep done")
    })
    event_loop()   // runs until all callbacks fire
}
```

- `on_ready(promise, callback)` — register a callback
- `event_loop([timeout_secs])` — run the event loop
- `set_queue_limit(n)` — cap the callback queue
- `set_task_timeout(secs|null)` — deadline per callback

---

### 14. Built-in Functions

#### I/O

| Function | Description |
|----------|-------------|
| `print(...)` | Print without newline |
| `println(...)` | Print with newline |
| `input([prompt])` | Read a line from stdin |

#### File System

| Function | Description |
|----------|-------------|
| `read_file(path)` | Read entire file as string |
| `write_file(path, content)` | Write string to file |
| `read_lines(path)` | Read file as array of lines |
| `append_file(path, content)` | Append string to file |
| `lines_iter(path)` | Lazy line iterator |
| `read_bytes(path)` | Read file as byte array |
| `write_bytes(path, bytes)` | Write byte array to file |
| `file_exists(path)` | True if path exists |
| `is_file(path)` | True if path is a file |
| `is_dir(path)` | True if path is a directory |
| `mkdir(path)` | Create directory |
| `list_dir(path)` | List directory entries |
| `path_join(a, b, ...)` | Join path segments |
| `rename(src, dst)` | Rename / move |
| `rm(path)` | Delete file or directory |

#### HTTP

| Function | Description |
|----------|-------------|
| `http_get(url [, opts])` | HTTP GET request |
| `http_post(url, body [, opts])` | HTTP POST request |

`opts` dict keys: `timeout` (seconds, int/float) and `user_agent` (string).

#### JSON

| Function | Description |
|----------|-------------|
| `json_parse(s)` | Parse JSON string to value |
| `json_stringify(v)` | Serialize value to JSON string |

#### Time

| Function | Description |
|----------|-------------|
| `clock()` | Unix epoch as float (seconds) |
| `sleep(secs)` | Sleep (sync or async via I/O pool) |

#### Type

| Function | Description |
|----------|-------------|
| `type(v)` | Returns type name string |
| `len(v)` | Length of string/array/dict/set |
| `int(v)` | Convert to int |
| `float(v)` | Convert to float |
| `str(v)` | Convert to string |
| `bool(v)` | Convert to bool |
| `set(arr)` | Create a set from array |

#### Async / Event Loop

| Function | Description |
|----------|-------------|
| `set_workers(n)` | Set I/O thread pool size |
| `on_ready(promise, callback)` | Register callback |
| `event_loop([timeout])` | Run event loop |
| `set_queue_limit(n)` | Limit queue depth |
| `set_task_timeout(secs\|null)` | Per-task deadline |

---

### 15. Module System

```aether
import math
from collections import map, filter
import string as str_utils

fn main() {
    let nums = range(1, 6)
    let doubled = map(nums, fn(x) { return x * 2 })
    println(doubled)
}
```

- Modules are `.ae` files resolved relative to the current file
- The embedded stdlib modules (`math`, `collections`, `string`, `core`, `testing`) are always available

---

### 16. Standard Library

| Module | Functions |
|--------|-----------|
| `core` | `range(n)`, `range(start, end)`, `range(start, end, step)`, `enumerate(arr)` |
| `collections` | `map`, `filter`, `reduce`, `find`, `every`, `some` |
| `math` | `abs`, `min`, `max`, `sum`, `clamp`, `sign` |
| `string` | `join`, `repeat`, `reverse`, `starts_with`, `ends_with` |
| `testing` | `assert_eq`, `assert_true`, `assert_false`, `assert_null`, `assert_not_null`, `expect_error`, `test`, `test_summary` |

---

### 17. Collection Methods

#### Array

| Method/Property | Description |
|-----------------|-------------|
| `arr.push(item)` | Append item |
| `arr.pop()` | Remove and return last item |
| `arr.sort()` | Sort in place |
| `arr.reverse()` | Reverse in place |
| `arr.concat(other)` | Return new concatenated array |
| `arr.slice(start, end)` | Return sub-array |
| `arr.contains(item)` | True if item is present |
| `arr.index_of(item)` | First index (-1 if absent) |
| `arr.join(sep)` | Join elements to string |
| `arr.length` | Element count |

#### Dict

| Method | Description |
|--------|-------------|
| `d.keys()` | Array of keys |
| `d.values()` | Array of values |
| `d.contains(key)` | True if key exists |

#### Set

| Method | Description |
|--------|-------------|
| `s.add(item)` | Add element |
| `s.remove(item)` | Remove element |
| `s.contains(item)` | True if element present |
| `s.union(other)` | Union of two sets |
| `s.intersection(other)` | Intersection |
| `s.difference(other)` | Difference |
| `s.is_subset(other)` | True if subset |
| `s.length` | Element count |

---

## Design Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Typing | Dynamic | Flexibility and ease of use |
| Memory | Rc-based GC | Predictable, no pauses |
| Syntax | C-like, no semicolons | Familiar yet clean |
| Execution | Tree-walking interpreter | Fast to implement, easy to extend |
| Scoping | Block-scoped | Prevents variable leakage |
| Strings | Immutable | Safe to share via Rc |
| Entry point | Required `main()` | Clear program structure |
| Numbers | Separate int / float | Precision control |
| Async | Thread pool + mpsc channels | No tokio dependency, thread-safe |
| Stdlib | Written in Aether | Dogfooding; user-readable |

---

**Last Updated**: April 29, 2026
**Phase**: 5 Complete
**Status**: Language stable; backlog items in progress
