---
layout: default
title: Iterator Protocol in Aether
---

# Iterator Protocol in Aether

**Status**: 🚧 In Design  
**Phase**: 5 Sprint 3  
**Target**: Lazy iteration and custom iterators

## Overview

The iterator protocol enables lazy, efficient iteration over collections and custom types. It provides a uniform interface for sequential access to elements without loading all data into memory.

## Design Goals

1. **Lazy Evaluation** - Process elements on-demand, not all at once
2. **Memory Efficient** - Iterate over large datasets without loading everything
3. **Uniform Interface** - Same pattern for built-in and custom types
4. **Composable** - Chain iterators for complex operations
5. **Backwards Compatible** - Existing for-in loops continue to work

## Iterator Interface

An iterator is an object that implements two methods:

### next()

Returns the next value in the sequence, or `null` when exhausted:

```aether
let value = iterator.next()
```

**Returns**:
- The next value in the sequence
- `null` when no more elements

### has_next()

Checks if more elements are available:

```aether
if (iterator.has_next()) {
    let value = iterator.next()
}
```

**Returns**:
- `true` if more elements available
- `false` if sequence exhausted

## Usage Patterns

### Basic Iteration

```aether
let arr = [1, 2, 3, 4, 5]
let iter = arr.iterator()

while (iter.has_next()) {
    let value = iter.next()
    println(value)
}
```

### For-In Loop Integration

For-in loops automatically use iterators:

```aether
let arr = [1, 2, 3]

// Under the hood, this uses arr.iterator()
for value in arr {
    println(value)
}
```

### Manual Iteration Control

```aether
let arr = [1, 2, 3, 4, 5]
let iter = arr.iterator()

// Take first 3 elements
let count = 0
while (iter.has_next() && count < 3) {
    println(iter.next())
    count = count + 1
}
// Remaining elements never accessed
```

## Built-in Iterators

### Array Iterator

Iterates over array elements in order:

```aether
let arr = [10, 20, 30]
let iter = arr.iterator()

println(iter.next())  // 10
println(iter.next())  // 20
println(iter.next())  // 30
println(iter.next())  // null
```

### Dict Iterator

Iterates over dict keys (like Python's dict iteration):

```aether
let dict = {"a": 1, "b": 2, "c": 3}
let iter = dict.iterator()

// Yields keys
println(iter.next())  // "a"
println(iter.next())  // "b"
println(iter.next())  // "c"
```

**Note**: Dict iteration order is not guaranteed.

### Dict Entries Iterator

Iterate over key-value pairs:

```aether
let dict = {"name": "Alice", "age": 30}
let iter = dict.entries()

while (iter.has_next()) {
    let entry = iter.next()  // Array [key, value]
    println("${entry[0]}: ${entry[1]}")
}
```

### Set Iterator

Iterates over set elements (order not guaranteed):

```aether
let s = set([1, 2, 3])
let iter = s.iterator()

while (iter.has_next()) {
    println(iter.next())
}
```

## Custom Iterators

### On Structs

Define `iterator()` method on structs to make them iterable:

```aether
struct Range {
    start
    end
    current
    
    fn iterator(self) {
        // Return a new iterator struct
        return RangeIterator { 
            current: self.start, 
            end: self.end 
        }
    }
}

struct RangeIterator {
    current
    end
    
    fn has_next(self) {
        return self.current < self.end
    }
    
    fn next(self) {
        if (self.current >= self.end) {
            return null
        }
        let value = self.current
        self.current = self.current + 1
        return value
    }
}

fn main() {
    let r = Range { start: 0, end: 5, current: 0 }
    
    for i in r {
        println(i)  // 0, 1, 2, 3, 4
    }
}
```

### Infinite Iterators

Iterators can be infinite (never return `null`):

```aether
struct Counter {
    value
    
    fn iterator(self) {
        return CounterIterator { value: self.value }
    }
}

struct CounterIterator {
    value
    
    fn has_next(self) {
        return true  // Always has next
    }
    
    fn next(self) {
        let current = self.value
        self.value = self.value + 1
        return current
    }
}

fn main() {
    let counter = Counter { value: 0 }
    let iter = counter.iterator()
    
    // Take first 5
    for i in range(0, 5) {
        println(iter.next())  // 0, 1, 2, 3, 4
    }
}
```

## Iterator Methods (Future)

These would be added to the stdlib:

### map(iterator, func)

Transform each element:

```aether
let arr = [1, 2, 3]
let iter = map(arr.iterator(), fn(x) { return x * 2 })

// Yields: 2, 4, 6
```

### filter(iterator, predicate)

Keep elements matching predicate:

```aether
let arr = [1, 2, 3, 4, 5]
let iter = filter(arr.iterator(), fn(x) { return x % 2 == 0 })

// Yields: 2, 4
```

### take(iterator, n)

Take first n elements:

```aether
let arr = [1, 2, 3, 4, 5]
let iter = take(arr.iterator(), 3)

// Yields: 1, 2, 3
```

### chain(iter1, iter2)

Chain two iterators:

```aether
let a = [1, 2].iterator()
let b = [3, 4].iterator()
let iter = chain(a, b)

// Yields: 1, 2, 3, 4
```

## Implementation Notes

### Value Type

Add `Iterator` variant to `Value` enum:

```rust
pub enum Value {
    // ... existing variants
    Iterator(Rc<RefCell<IteratorState>>),
}

pub struct IteratorState {
    source: Box<dyn IteratorSource>,
    exhausted: bool,
}

trait IteratorSource {
    fn next(&mut self) -> Option<Value>;
    fn has_next(&self) -> bool;
}
```

### For-In Loop Desugaring

Transform for-in loops to use iterators:

```aether
// Source:
for item in collection {
    body
}

// Desugars to:
{
    let __iter = collection.iterator()
    while (__iter.has_next()) {
        let item = __iter.next()
        body
    }
}
```

### Built-in Iterator Sources

Implement `IteratorSource` trait for:
- `ArrayIterator` - wraps `Vec<Value>` with index
- `DictIterator` - wraps `HashMap` keys iterator
- `SetIterator` - wraps `HashSet` iterator
- `StructIterator` - calls struct's `next()` and `has_next()` methods

## Examples

### Example 1: Fibonacci Iterator

```aether
struct Fibonacci {
    a
    b
    
    fn iterator(self) {
        return FibIterator { a: 0, b: 1 }
    }
}

struct FibIterator {
    a
    b
    
    fn has_next(self) {
        return true  // Infinite
    }
    
    fn next(self) {
        let current = self.a
        let next = self.a + self.b
        self.a = self.b
        self.b = next
        return current
    }
}

fn main() {
    let fib = Fibonacci { a: 0, b: 1 }
    let iter = fib.iterator()
    
    // First 10 Fibonacci numbers
    for i in range(0, 10) {
        println(iter.next())
    }
}
```

### Example 2: File Lines Iterator

```aether
struct FileLines {
    filename
    
    fn iterator(self) {
        let content = read_file(self.filename)
        let lines = content.split("\n")
        return lines.iterator()
    }
}

fn main() {
    let file = FileLines { filename: "data.txt" }
    
    // Process lines lazily
    for line in file {
        if (line.trim().length > 0) {
            println(line)
        }
    }
}
```

### Example 3: Chunked Iterator

```aether
struct ChunkedIterator {
    source
    chunk_size
    
    fn has_next(self) {
        return self.source.has_next()
    }
    
    fn next(self) {
        let chunk = []
        let count = 0
        
        while (self.source.has_next() && count < self.chunk_size) {
            chunk.push(self.source.next())
            count = count + 1
        }
        
        return chunk
    }
}

fn main() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9]
    let iter = ChunkedIterator { 
        source: arr.iterator(), 
        chunk_size: 3 
    }
    
    while (iter.has_next()) {
        println(iter.next())
    }
    // Output: [1,2,3], [4,5,6], [7,8,9]
}
```

## Best Practices

### 1. Always Check has_next()

```aether
// Good
if (iter.has_next()) {
    let value = iter.next()
}

// Risky - may get null
let value = iter.next()
```

### 2. Don't Mutate During Iteration

```aether
// Bad - modifying collection during iteration
let arr = [1, 2, 3]
for item in arr {
    arr.push(item * 2)  // Undefined behavior
}

// Good - iterate over copy
let arr = [1, 2, 3]
let copy = arr[:]
for item in copy {
    arr.push(item * 2)
}
```

### 3. Iterator Ownership

Once you call `iterator()`, the original collection is still accessible:

```aether
let arr = [1, 2, 3]
let iter = arr.iterator()

println(len(arr))  // Still works: 3
```

### 4. Multiple Iterators

Each `iterator()` call creates independent iterator:

```aether
let arr = [1, 2, 3]
let iter1 = arr.iterator()
let iter2 = arr.iterator()

println(iter1.next())  // 1
println(iter2.next())  // 1 (independent)
```

## Limitations

### No Iterator Invalidation

Unlike Rust, Aether doesn't prevent collection modification during iteration:

```aether
let arr = [1, 2, 3]
for item in arr {
    arr.push(4)  // Allowed but discouraged
}
```

### No Generic Iterator Combinators

Without generics, iterator combinators must be specific functions:

```aether
// Can't write generic combinator
fn map_iterator(iter, func) { ... }  // Must handle all types
```

### Performance Overhead

Iterator abstraction adds method call overhead compared to direct indexing.

## Migration Path

### Phase 1: Core Protocol
- Add `Iterator` value type
- Implement `iterator()` for arrays, dicts, sets
- Add `next()` and `has_next()` methods
- Tests for basic iteration

### Phase 2: For-In Integration
- Desugar for-in to use iterators
- Maintain backwards compatibility
- Test all existing for-in loops

### Phase 3: Stdlib Iterators
- Add iterator utility functions (map, filter, take, chain)
- Document patterns
- Examples

### Phase 4: Custom Iterators
- Enable struct-based iterators
- Document custom iterator pattern
- Advanced examples

## See Also

- [DESIGN.md](DESIGN.html) - Language specification
- [STRUCT.md](STRUCT.html) - User-defined types
- [STDLIB.md](STDLIB.html) - Standard library

---

**Last Updated**: 2026-04-28  
**Status**: Design document (not yet implemented)
