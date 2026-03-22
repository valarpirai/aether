# Aether Standard Library

**Status**: 🚧 In Development (Phase 3)

## Overview

The Aether standard library provides commonly-used functions and utilities implemented in Aether itself (not Rust). This demonstrates the language's capability and provides extensible, user-readable implementations.

## Design Philosophy

### What Goes Where?

**Built-in Functions (Rust)** - Primitives that require interpreter internals:
- I/O operations: `print()`, `println()`
- Type introspection: `type()`, `len()`
- Type conversions: `int()`, `float()`, `str()`, `bool()`
- Collection methods: `array.push()`, `string.upper()`, etc.

**Standard Library (Aether)** - Higher-level functions built on primitives:
- Iteration helpers: `range()`, `enumerate()`
- Collection utilities: `map()`, `filter()`, `reduce()`
- Math functions: `abs()`, `min()`, `max()`, `sum()`
- String utilities: `join()`, `repeat()`
- Functional utilities: `compose()`, `partial()`

## Module System

### Directory Structure

```
stdlib/
├── core.ae          # Core utilities (range, enumerate)
├── collections.ae   # Collection operations (map, filter, reduce)
├── math.ae          # Mathematical functions
├── string.ae        # String utilities
└── functional.ae    # Functional programming utilities
```

### Loading Mechanism

1. **Embedded in Binary**: Stdlib files are compiled into the binary using `include_str!()`
2. **Automatic Loading**: Core modules loaded automatically at startup
3. **Lazy Loading**: Other modules loaded on first use
4. **No File I/O**: Works everywhere, no deployment complexity

### Import Syntax (Future)

```aether
// Explicit import (Phase 4+)
import collections

let doubled = collections.map([1, 2, 3], fn(x) { return x * 2 })
```

For Phase 3, all stdlib functions are globally available (auto-imported).

## Standard Library Functions

### Core Utilities (`stdlib/core.ae`)

#### `range(n)` or `range(start, end)`
Creates an array of integers from start (inclusive) to end (exclusive).

```aether
range(5)        // [0, 1, 2, 3, 4]
range(2, 7)     // [2, 3, 4, 5, 6]
```

**Implementation**:
```aether
fn range(start, end) {
    // Handle single argument: range(n) -> range(0, n)
    let actual_start = 0
    let actual_end = start

    if (end != null) {
        actual_start = start
        actual_end = end
    }

    let result = []
    let i = actual_start
    while (i < actual_end) {
        result.push(i)
        i = i + 1
    }
    return result
}
```

#### `enumerate(array)`
Returns array of [index, value] pairs.

```aether
enumerate(["a", "b", "c"])  // [[0, "a"], [1, "b"], [2, "c"]]
```

### Collections (`stdlib/collections.ae`)

#### `map(array, fn)`
Apply function to each element, return new array.

```aether
map([1, 2, 3], fn(x) { return x * 2 })  // [2, 4, 6]
```

#### `filter(array, predicate)`
Keep only elements where predicate returns true.

```aether
filter([1, 2, 3, 4], fn(x) { return x % 2 == 0 })  // [2, 4]
```

#### `reduce(array, fn, initial)`
Reduce array to single value using accumulator function.

```aether
reduce([1, 2, 3, 4], fn(acc, x) { return acc + x }, 0)  // 10
```

#### `find(array, predicate)`
Find first element matching predicate.

```aether
find([1, 2, 3, 4], fn(x) { return x > 2 })  // 3
```

#### `every(array, predicate)`
Check if all elements satisfy predicate.

```aether
every([2, 4, 6], fn(x) { return x % 2 == 0 })  // true
```

#### `some(array, predicate)`
Check if any element satisfies predicate.

```aether
some([1, 3, 4], fn(x) { return x % 2 == 0 })  // true
```

### Math Utilities (`stdlib/math.ae`)

#### `abs(n)`
Absolute value.

```aether
abs(-5)   // 5
abs(3.14) // 3.14
```

#### `min(a, b)` or `min(array)`
Minimum value.

```aether
min(3, 7)        // 3
min([3, 1, 4])   // 1
```

#### `max(a, b)` or `max(array)`
Maximum value.

```aether
max(3, 7)        // 7
max([3, 1, 4])   // 4
```

#### `sum(array)`
Sum of array elements.

```aether
sum([1, 2, 3, 4])  // 10
```

#### `clamp(value, min_val, max_val)`
Constrain value to range.

```aether
clamp(15, 0, 10)  // 10
clamp(-5, 0, 10)  // 0
clamp(5, 0, 10)   // 5
```

### String Utilities (`stdlib/string.ae`)

#### `join(array, separator)`
Join array elements into string.

```aether
join(["hello", "world"], " ")  // "hello world"
```

#### `repeat(string, n)`
Repeat string n times.

```aether
repeat("ha", 3)  // "hahaha"
```

#### `reverse(string)`
Reverse a string.

```aether
reverse("hello")  // "olleh"
```

## Implementation Strategy

### Phase 3.1: Foundation
1. Create stdlib directory structure
2. Implement module loader (embedded resources)
3. Add stdlib prelude to evaluator
4. Test core utilities (range, enumerate)

### Phase 3.2: Collections
1. Implement map, filter, reduce
2. Add find, every, some
3. Comprehensive test coverage

### Phase 3.3: Math & String
1. Implement math utilities
2. Implement string utilities
3. Integration tests

### Phase 3.4: Documentation & Examples
1. Document all functions
2. Create example programs
3. Update user guide

## Technical Implementation

### Embedding Files in Binary

Use Rust's `include_str!()` macro:

```rust
// src/interpreter/stdlib.rs
pub const STDLIB_CORE: &str = include_str!("../../stdlib/core.ae");
pub const STDLIB_COLLECTIONS: &str = include_str!("../../stdlib/collections.ae");
pub const STDLIB_MATH: &str = include_str!("../../stdlib/math.ae");
pub const STDLIB_STRING: &str = include_str!("../../stdlib/string.ae");
```

### Loading at Runtime

```rust
impl Evaluator {
    pub fn new() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
        };
        evaluator.register_builtins();
        evaluator.load_stdlib();  // Load stdlib modules
        evaluator
    }

    fn load_stdlib(&mut self) {
        // Parse and execute stdlib modules
        self.exec_module(STDLIB_CORE).expect("Failed to load core stdlib");
        self.exec_module(STDLIB_COLLECTIONS).expect("Failed to load collections");
        // ... other modules
    }
}
```

## Usage Examples

### Example 1: FizzBuzz with stdlib

```aether
// Using range() from stdlib
for i in range(1, 101) {
    if (i % 15 == 0) {
        println("FizzBuzz")
    } else if (i % 3 == 0) {
        println("Fizz")
    } else if (i % 5 == 0) {
        println("Buzz")
    } else {
        println(i)
    }
}
```

### Example 2: Functional Programming

```aether
// Using map, filter, reduce from stdlib
let numbers = range(1, 11)

let evens = filter(numbers, fn(x) { return x % 2 == 0 })
println("Evens:", evens)  // [2, 4, 6, 8, 10]

let doubled = map(evens, fn(x) { return x * 2 })
println("Doubled:", doubled)  // [4, 8, 12, 16, 20]

let sum_val = reduce(doubled, fn(acc, x) { return acc + x }, 0)
println("Sum:", sum_val)  // 60
```

### Example 3: String Processing

```aether
let words = ["hello", "beautiful", "world"]
let sentence = join(words, " ")
println(sentence.upper())  // "HELLO BEAUTIFUL WORLD"

let repeated = repeat("*", 10)
println(repeated)  // "**********"
```

## Testing Strategy

### Unit Tests (Rust)
- Test module loading mechanism
- Test embedded resource reading
- Test stdlib execution without errors

### Integration Tests (Aether)
- Test each stdlib function individually
- Test function composition
- Test edge cases (empty arrays, null values, etc.)

### Example Programs
- Real-world programs using stdlib
- Performance benchmarks
- Error handling scenarios

## Performance Considerations

### Optimization Opportunities
- **Lazy Loading**: Only load modules when needed
- **Caching**: Parse stdlib once, reuse AST
- **Inlining**: Compiler could inline simple stdlib functions
- **Native Implementations**: Hot paths could be moved to Rust

### Benchmarking
Track performance of stdlib functions vs. Rust built-ins:
- `range()` vs. hypothetical Rust `builtin_range()`
- `map()` vs. manual loop
- Function call overhead

## Future Enhancements (Phase 4+)

### Module System
- Explicit imports: `import collections`
- Selective imports: `from math import abs, max`
- User modules: Load `.ae` files from filesystem
- Module namespaces: Prevent naming conflicts

### Advanced Features
- **Lazy sequences**: Infinite ranges, generators
- **Parallel operations**: `pmap()` for concurrent execution
- **Memoization**: Cache function results
- **Decorators**: Function wrapping and composition

### Standard Library Expansion
- **I/O**: File reading/writing (when file I/O built-in added)
- **JSON**: Parsing and serialization
- **HTTP**: Web requests (with async support)
- **Time/Date**: Date manipulation
- **RegEx**: Regular expressions
- **Testing**: Unit testing framework

## Contribution Guidelines

### Adding New Functions

1. **Determine placement**: Is this a built-in or stdlib function?
2. **Implement in Aether**: Write function in appropriate stdlib file
3. **Write tests**: Add integration tests
4. **Document**: Add to this file with examples
5. **Test coverage**: Ensure edge cases covered

### Code Style
- Clear, readable Aether code
- Descriptive function names
- Handle edge cases (empty arrays, null values)
- Return consistent types
- Use early returns for error cases

## Migration Path

### From Rust Built-ins to Stdlib

Some functions might start as Rust built-ins and later move to stdlib:

```rust
// Before: Rust built-in
pub fn builtin_range(args: &[Value]) -> Result<Value, RuntimeError> {
    // Complex Rust implementation
}
```

```aether
// After: Aether stdlib
fn range(start, end) {
    let result = []
    let i = start
    while (i < end) {
        result.push(i)
        i += 1
    }
    return result
}
```

This improves:
- Maintainability (simpler code)
- Transparency (users can read implementation)
- Flexibility (users can override if needed)

## References

- **Python**: Large stdlib, mix of C and Python
- **Ruby**: Core vs. stdlib distinction
- **JavaScript**: Small core, large npm ecosystem
- **Lua**: Minimal core, extension through modules
- **Scheme**: Small core, R6RS/R7RS libraries

Aether follows the **small core, rich stdlib** philosophy.

---

**Last Updated**: March 22, 2026
**Phase**: 3 Complete
**Status**: 28+ functions implemented, 83 tests passing
