---
layout: default
title: "Aether — Standard Library"
---

[Home](../index.html) › Language Reference › Standard Library

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
| `sort` | collections | `sort(array, fn?)` → sorted array |
| `zip` | collections | `zip(arr1, arr2)` → array of pairs |
| `flatten` | collections | `flatten(array)` → one level flattened |
| `first` | collections | `first(array)` → value or null |
| `last` | collections | `last(array)` → value or null |
| `chunk` | collections | `chunk(array, size)` → array of chunks |
| `partition` | collections | `partition(array, fn)` → `[matches, rest]` |
| `zip_longest` | collections | `zip_longest(arr1, arr2, fill)` → array |
| `uniq_by` | collections | `uniq_by(array, fn)` → deduped array |
| `sqrt` | math | `sqrt(n)` → float |
| `floor` / `ceil` / `round` | math | → int |
| `trunc` | math | `trunc(n)` → int (toward zero) |
| `log` | math | `log(n)` / `log(n, base)` → float |
| `gcd` / `lcm` | math | `gcd(a, b)` / `lcm(a, b)` → int |
| `factorial` | math | `factorial(n)` → int |
| `hypot` | math | `hypot(x, y)` → float |
| `exp` | math | `exp(x)` → float |
| `sin` / `cos` / `tan` | math | trig in radians → float |
| `degrees` / `radians` | math | angle conversion → float |
| `pi` / `e` / `tau` | math | constants |
| `contains` | string | `contains(string, sub)` → bool |
| `index_of` | string | `index_of(string, sub)` → int (-1 if absent) |
| `replace` | string | `replace(string, old, new)` → string |
| `count` | string | `count(string, sub)` → int |
| `pad_left` | string | `pad_left(string, width, char)` → string |
| `pad_right` | string | `pad_right(string, width, char)` → string |
| `strip_prefix` | string | `strip_prefix(string, prefix)` → string |
| `strip_suffix` | string | `strip_suffix(string, suffix)` → string |
| `is_alpha` | string | `is_alpha(string)` → bool |
| `is_digit` | string | `is_digit(string)` → bool |
| `is_space` | string | `is_space(string)` → bool |
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

### first(array) / last(array)

Return the first or last element, or `null` for empty arrays.

```aether
first([10, 20, 30])  // 10
last([10, 20, 30])   // 30
first([])            // null
```

### chunk(array, size)

Splits an array into chunks of the given size. The last chunk may be smaller.

```aether
chunk([1,2,3,4,5], 2)  // [[1,2],[3,4],[5]]
```

### partition(array, fn)

Returns `[matches, non_matches]`.

```aether
fn is_even(x) { return x % 2 == 0 }
partition([1,2,3,4,5], is_even)  // [[2,4],[1,3,5]]
```

### zip_longest(arr1, arr2, fill)

Like `zip` but pads the shorter array with `fill`.

```aether
zip_longest([1,2,3], ["a","b"], null)  // [[1,"a"],[2,"b"],[3,null]]
```

### uniq_by(array, fn)

Removes all duplicates (not just consecutive) keyed by `fn`. Stable — first occurrence wins.

```aether
fn identity(x) { return x }
uniq_by([1,2,1,3,2], identity)  // [1,2,3]
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

### Mathematical constants

```aether
pi   // 3.141592653589793
e    // 2.718281828459045
tau  // 6.283185307179586
```

### factorial(n)

```aether
factorial(5)   // 120
factorial(10)  // 3628800
```

### trunc(n)

Truncates toward zero (unlike `floor` which rounds toward negative infinity).

```aether
trunc(3.9)   // 3
trunc(-3.9)  // -3
```

### degrees(r) / radians(d)

```aether
degrees(pi)    // 180.0
radians(180.0) // 3.141592653589793
```

### hypot(x, y)

Euclidean distance: `sqrt(x² + y²)`.

```aether
hypot(3, 4)  // 5.0
```

### exp(x)

`e` raised to the power `x`.

```aether
exp(1.0)  // 2.718281828...
```

### sin(x) / cos(x) / tan(x)

Trigonometric functions, argument in radians.

```aether
sin(pi / 2.0)  // 1.0
cos(0.0)       // 1.0
tan(pi / 4.0)  // 1.0
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

### contains(string, sub) / index_of(string, sub)

```aether
contains("hello world", "world")  // true
index_of("hello", "ll")           // 2
index_of("hello", "xyz")          // -1
```

### replace(string, old, new_val)

Replaces all occurrences.

```aether
replace("hello world", "world", "aether")  // "hello aether"
```

### count(string, sub)

Counts non-overlapping occurrences.

```aether
count("banana", "a")  // 3
```

### pad_left(string, width, char) / pad_right(string, width, char)

```aether
pad_left("42", 5, "0")    // "00042"
pad_right("hi", 5, ".")   // "hi..."
```

### strip_prefix(string, prefix) / strip_suffix(string, suffix)

Removes the prefix/suffix if present, otherwise returns the string unchanged.

```aether
strip_prefix("https://example.com", "https://")  // "example.com"
strip_suffix("report.ae", ".ae")                  // "report"
```

### is_alpha(string) / is_digit(string) / is_space(string)

Returns `true` if the string is non-empty and all characters match.

```aether
is_alpha("hello")   // true
is_digit("12345")   // true
is_space("   ")     // true
is_alpha("hello1")  // false
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

---
[← Modules](MODULE_SYSTEM.html) &nbsp;&nbsp; [Iterators →](ITERATORS.html)
