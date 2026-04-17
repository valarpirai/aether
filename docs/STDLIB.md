# Aether Standard Library

**Status**: ✅ Complete (Phase 5)

## Overview

The Aether standard library provides commonly-used functions and utilities implemented in Aether itself (not Rust). This demonstrates the language's capability and provides extensible, user-readable implementations.

## Design Philosophy

### What Goes Where?

**Built-in Functions (Java)** - Primitives that require interpreter internals:
- I/O operations: `print()`, `println()`
- Type introspection: `type()`, `len()`
- Type conversions: `int()`, `float()`, `str()`, `bool()`
- Collection methods: `array.push()`, `array.sort()`, `string.upper()`, etc.

**Standard Library (Aether)** - Higher-level functions built on primitives:
- Iteration helpers: `range()`, `enumerate()`
- Collection utilities: `map()`, `filter()`, `reduce()`
- Math functions: `abs()`, `min()`, `max()`, `sum()`
- String utilities: `join()`, `repeat()`
- Functional utilities: `compose()`, `partial()`

## Module System

### Directory Structure

```
src/main/resources/stdlib/
├── core.ae          # Core utilities (range, enumerate)
├── collections.ae   # Collection operations (map, filter, reduce)
├── math.ae          # Mathematical functions
├── string.ae        # String utilities
└── testing.ae       # Testing framework (assert_eq, test, test_summary)
```

### Loading Mechanism

1. **Classpath resources**: Stdlib `.ae` files are bundled inside the JAR under `stdlib/`
2. **On-demand loading**: Modules are loaded when first imported via `import` or `from ... import`
3. **Caching**: Each module is parsed and executed once; subsequent imports reuse the cached environment
4. **No File I/O**: Works everywhere the JAR runs, no deployment complexity

### Import Syntax

All stdlib functions are globally available (auto-loaded at startup). The module system also supports explicit imports for user modules:

```aether
import collections

let doubled = collections.map([1, 2, 3], fn(x) { return x * 2 })
```

## Built-in Array Methods

These are implemented in the Java interpreter directly (not in Aether).

#### `array.push(value)`
Append a value to the end. Returns `null`. Mutates the array.

```aether
let a = [1, 2]
a.push(3)    // a is now [1, 2, 3]
```

#### `array.pop()`
Remove and return the last element. Returns `null` on an empty array.

```aether
let a = [1, 2, 3]
a.pop()      // 3  (a is now [1, 2])
```

#### `array.sort()` / `array.sort(comparator)`
Sort in place. Without a comparator, sorts in natural order. With a comparator `fn(a, b)`, the function should return `true` if `a` should come before `b`.

```aether
let a = [3, 1, 4, 2]
a.sort()                              // a is now [1, 2, 3, 4]
a.sort(fn(a, b) { return a > b })    // a is now [4, 3, 2, 1]
```

#### `array.contains(value)`
Return `true` if the value exists in the array.

```aether
[1, 2, 3].contains(2)   // true
[1, 2, 3].contains(9)   // false
```

#### `array.concat(other)`
Return a new array with all elements of both arrays (does not mutate).

```aether
[1, 2].concat([3, 4])   // [1, 2, 3, 4]
```

#### `array.length`
Property (not method) returning the number of elements.

```aether
[1, 2, 3].length   // 3
```

---

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

#### `starts_with(string, prefix)`
Check if string starts with prefix.

```aether
starts_with("hello", "he")  // true
```

#### `ends_with(string, suffix)`
Check if string ends with suffix.

```aether
ends_with("hello", "lo")  // true
```

### Testing Framework (`stdlib/testing.ae`)

#### `assert_eq(actual, expected)`
Assert two values are equal. Throws on failure.

```aether
assert_eq(1 + 1, 2)
assert_eq("hello".upper(), "HELLO")
```

#### `assert_true(value)`
Assert value is truthy.

#### `assert_false(value)`
Assert value is falsy.

#### `assert_null(value)`
Assert value is null.

#### `assert_not_null(value)`
Assert value is not null.

#### `expect_error(fn)`
Assert that calling fn() throws an error.

```aether
expect_error(fn() { throw "boom" })
```

#### `test(name, fn)`
Register and run a named test case.

```aether
test("addition works", fn() {
    assert_eq(1 + 2, 3)
})
```

#### `test_summary()`
Print summary of all test results (pass/fail counts).

## Technical Implementation

### Bundling Files in the JAR

Stdlib `.ae` files live in `src/main/resources/stdlib/` and are packaged into the fat JAR automatically by Maven. `StdlibLoader` reads them at runtime via the classpath:

```java
// StdlibLoader.java
private static String loadResource(String name) {
  try (InputStream is = StdlibLoader.class.getResourceAsStream("/stdlib/" + name)) {
    return new String(is.readAllBytes(), StandardCharsets.UTF_8);
  }
}
```

### Loading at Runtime

```java
// Evaluator.java
public static Evaluator withStdlib() {
  Evaluator ev = new Evaluator();
  ev.registerBuiltins();
  StdlibLoader.load(ev);   // parses and executes each module
  return ev;
}

// Evaluator.withoutStdlib() skips this for fast test startup
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

### Unit Tests (Java — `StdlibTest.java`, 41 tests)
Use `Evaluator.withStdlib()` so stdlib functions are available:

```java
@BeforeEach
void setUp() {
  evaluator = Evaluator.withStdlib();
}

@Test
void mapDoubles() {
  assertEquals("[2, 4, 6]", eval("map([1, 2, 3], fn(x) { return x * 2 })"));
}
```

### Integration Tests
- `TestingFrameworkTest.java` — assert_eq, assert_true, test(), test_summary
- `ModuleTest.java` — import/from-import/alias for all stdlib modules
- `StdlibTest.java` — each function exercised individually with edge cases

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

## Future Enhancements

### Standard Library Expansion (Planned)
- **HTTP**: `http_get()`, `http_post()` (requires an HTTP client dependency)
- **RegEx**: Regular expression matching
- **Path**: File path manipulation utilities

### Advanced Features
- **Lazy sequences**: Infinite ranges, generators
- **Memoization**: Cache function results
- **Parallel operations**: `pmap()` for concurrent execution

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

## References

- **Python**: Large stdlib, mix of native C and Python
- **Ruby**: Core vs. stdlib distinction
- **JavaScript**: Small core, large npm ecosystem
- **Lua**: Minimal core, extension through modules
- **Scheme**: Small core, R6RS/R7RS libraries

Aether follows the **small core, rich stdlib** philosophy.

---

**Last Updated**: April 17, 2026
**Phase**: Complete
**Status**: 35+ functions implemented, 451 tests passing
