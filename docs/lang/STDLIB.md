# Standard Library

Most standard library functions are available automatically — no import needed. The `testing` module must be imported explicitly.

## Quick Reference

| Function | Module | Signature |
|----------|--------|-----------|
| `range` | core | `range(end)` / `range(start, end)` → array |
| `enumerate` | core | `enumerate(array)` → `[[index, value], ...]` |
| `map` | collections | `map(array, fn)` → array |
| `filter` | collections | `filter(array, fn)` → array |
| `reduce` | collections | `reduce(array, fn, initial)` → value |
| `find` | collections | `find(array, fn)` → value or null |
| `every` | collections | `every(array, fn)` → bool |
| `some` | collections | `some(array, fn)` → bool |
| `abs` | math | `abs(n)` → number |
| `min` | math | `min(a, b)` / `min(array)` → number |
| `max` | math | `max(a, b)` / `max(array)` → number |
| `sum` | math | `sum(array)` → number |
| `clamp` | math | `clamp(value, min_val, max_val)` → number |
| `sign` | math | `sign(n)` → -1, 0, or 1 |
| `join` | string | `join(array, separator)` → string |
| `repeat` | string | `repeat(string, n)` → string |
| `reverse` | string | `reverse(string)` → string |
| `starts_with` | string | `starts_with(string, prefix)` → bool |
| `ends_with` | string | `ends_with(string, suffix)` → bool |
| `assert_eq` | testing* | `assert_eq(actual, expected)` |
| `test` | testing* | `test(name, fn)` |
| `test_summary` | testing* | `test_summary()` |

*Requires explicit import: `from testing import assert_eq, test, test_summary`

## Core (`core.ae`)

### range(n) / range(start, end)

Returns an array of integers from `start` (inclusive) to `end` (exclusive). With one argument, starts from 0.

```aether
range(5)        // [0, 1, 2, 3, 4]
range(2, 7)     // [2, 3, 4, 5, 6]
```

### enumerate(array)

Returns an array of `[index, value]` pairs.

```aether
enumerate(["a", "b", "c"])  // [[0, "a"], [1, "b"], [2, "c"]]
```

## Collections (`collections.ae`)

### map(array, fn)

Applies `fn` to every element and returns a new array.

```aether
map([1, 2, 3], fn(x) { return x * 2 })   // [2, 4, 6]
```

### filter(array, fn)

Returns a new array with only the elements for which `fn` returns true.

```aether
filter([1, 2, 3, 4], fn(x) { return x % 2 == 0 })  // [2, 4]
```

### reduce(array, fn, initial)

Reduces the array to a single value by calling `fn(accumulator, element)` for each element.

```aether
reduce([1, 2, 3, 4], fn(acc, x) { return acc + x }, 0)  // 10
```

### find(array, fn)

Returns the first element for which `fn` returns true, or `null` if none match.

```aether
find([1, 2, 3, 4], fn(x) { return x > 2 })  // 3
```

### every(array, fn)

Returns `true` if `fn` returns true for every element.

```aether
every([2, 4, 6], fn(x) { return x % 2 == 0 })  // true
```

### some(array, fn)

Returns `true` if `fn` returns true for at least one element.

```aether
some([1, 3, 4], fn(x) { return x % 2 == 0 })  // true
```

## Math (`math.ae`)

### abs(n)

```aether
abs(-5)    // 5
abs(3.14)  // 3.14
```

### min / max

```aether
min(3, 7)       // 3
max(3, 7)       // 7
min([3, 1, 4])  // 1
max([3, 1, 4])  // 4
```

### sum(array)

```aether
sum([1, 2, 3, 4])  // 10
```

### clamp(value, min_val, max_val)

Returns `value` constrained to `[min_val, max_val]`.

```aether
clamp(15, 0, 10)  // 10
clamp(-5, 0, 10)  // 0
clamp(5, 0, 10)   // 5
```

### sign(n)

Returns `-1`, `0`, or `1`.

```aether
sign(-5)  // -1
sign(0)   // 0
sign(7)   // 1
```

## String (`string.ae`)

### join(array, separator)

```aether
join(["hello", "world"], " ")  // "hello world"
join(["a", "b", "c"], ", ")    // "a, b, c"
```

### repeat(string, n)

```aether
repeat("ha", 3)   // "hahaha"
repeat("*", 10)   // "**********"
```

### reverse(string)

```aether
reverse("hello")  // "olleh"
```

### starts_with(string, prefix) / ends_with(string, suffix)

```aether
starts_with("hello", "he")  // true
ends_with("hello", "lo")    // true
```

## Testing (`testing.ae`)

Import explicitly to use the testing framework:

```aether
from testing import assert_eq, test, test_summary
```

### assert_eq(actual, expected)

Throws if `actual != expected`.

```aether
assert_eq(1 + 1, 2)
assert_eq("hello".upper(), "HELLO")
```

### assert_true(value) / assert_false(value) / assert_null(value) / assert_not_null(value)

```aether
assert_true(1 > 0)
assert_null(null)
```

### expect_error(fn)

Asserts that calling `fn()` throws an error.

```aether
expect_error(fn() { throw "boom" })
```

### test(name, fn)

Registers and runs a named test case.

```aether
test("addition works", fn() {
    assert_eq(1 + 2, 3)
})
```

### test_summary()

Prints a pass/fail summary of all tests run so far.

## Example: functional pipeline

```aether
let numbers = range(1, 11)

let result = reduce(
    map(
        filter(numbers, fn(x) { return x % 2 == 0 }),
        fn(x) { return x * 2 }
    ),
    fn(acc, x) { return acc + x },
    0
)
println(result)  // 60
```
