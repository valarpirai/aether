# Iterators

An iterator is any object with `has_next()` and `next()` methods. `for-in` loops work with any iterator automatically.

## Iterator interface

| Method | Returns |
|--------|---------|
| `has_next()` | `true` if more elements remain, `false` when exhausted |
| `next()` | Next value, or `null` when exhausted |

## Built-in iterators

Every collection has an `.iterator()` method that returns an iterator:

```aether
let arr = [10, 20, 30]
let iter = arr.iterator()

while (iter.has_next()) {
    println(iter.next())   // 10, 20, 30
}
```

**Array** — yields elements in order  
**Dict** — yields keys (order not guaranteed)  
**Set** — yields elements (order not guaranteed)  
**String** — for-in yields one character at a time

### Dict entries

`dict.entries()` yields `[key, value]` pairs:

```aether
let d = {"name": "Alice", "age": 30}
let iter = d.entries()

while (iter.has_next()) {
    let pair = iter.next()
    println("${pair[0]}: ${pair[1]}")
}
```

## for-in with iterators

`for-in` works directly with iterators — no need to call `.iterator()` explicitly:

```aether
for value in [1, 2, 3] {
    println(value)
}

for char in "hello" {
    println(char)
}
```

## Manual iteration control

Use `has_next()` / `next()` when you need to stop early or skip elements:

```aether
let iter = [1, 2, 3, 4, 5].iterator()

// Take only first 3
let count = 0
while (iter.has_next() && count < 3) {
    println(iter.next())
    count = count + 1
}
```

## Custom iterators

Make any struct iterable by defining `has_next()` and `next()` methods. `for-in` will call them automatically.

```aether
struct CountUp {
    current
    limit

    fn has_next(self) {
        return self.current < self.limit
    }

    fn next(self) {
        let v = self.current
        self.current = self.current + 1
        return v
    }
}

fn main() {
    let counter = CountUp { current: 0, limit: 5 }
    for n in counter {
        println(n)   // 0, 1, 2, 3, 4
    }
}
```

### Fibonacci iterator

```aether
struct Fibonacci {
    a
    b

    fn has_next(self) {
        return true
    }

    fn next(self) {
        let value = self.a
        let next = self.a + self.b
        self.a = self.b
        self.b = next
        return value
    }
}

fn main() {
    let fib = Fibonacci { a: 0, b: 1 }
    for i in range(0, 10) {
        println(fib.next())   // 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
    }
}
```

## Notes

- Built-in `map()` and `filter()` from `collections` work on arrays, not raw iterators
- Iterator state is mutable — advancing one reference advances all references to the same iterator
