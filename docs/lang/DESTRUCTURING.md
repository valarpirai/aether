---
layout: default
title: "Aether — Destructuring"
---

[Home](../index.html) › Language Reference › Destructuring

# Destructuring

Destructuring lets you unpack arrays and dicts directly into named variables in a single `let` statement.

## Array Destructuring

```aether
let [a, b, c] = [1, 2, 3]
println(a)   // 1
println(b)   // 2
```

### Fewer values than bindings

Missing positions become `null`:

```aether
let [a, b, c] = [10, 20]
println(c)   // null
```

### Default values

Provide a default that is used when the position is missing:

```aether
let [x, y = 99] = [1]
println(y)   // 99
```

### Rest element

`...name` collects all remaining elements into an array. It must be the last element in the pattern.

```aether
let [head, ...tail] = [1, 2, 3, 4, 5]
println(head)        // 1
println(len(tail))   // 4
```

If there are no remaining elements, the rest binding is an empty array:

```aether
let [a, ...rest] = [1]
println(len(rest))   // 0
```

### Skipping elements

Use `_` to skip a position without creating a binding:

```aether
let [_, second, _] = [10, 20, 30]
println(second)   // 20
```

## Dict Destructuring

### Shorthand

Binds a variable with the same name as the key:

```aether
let point = {"x": 3, "y": 7}
let {x, y} = point
println(x)   // 3
println(y)   // 7
```

### Rename

Use `key: alias` to bind the value under a different name:

```aether
let config = {"host": "db.internal", "port": 5432}
let {host, port: p} = config
println(p)   // 5432
```

### Default values

Provide a default used when the key is absent or `null`:

```aether
let {timeout: t = 30} = {"host": "api.example.com"}
println(t)   // 30
```

### Missing keys

A key that is not present in the dict produces `null` (or the default if one is given):

```aether
let {x, z} = {"x": 1}
println(z)   // null
```

## Combining forms

```aether
let {host, port: p = 5432, timeout: t = 30} = cfg
```

## In functions

Destructuring works anywhere `let` does, including function bodies:

```aether
fn connect(cfg) {
    let {host, port: p = 5432} = cfg
    return host + ":" + str(p)
}

fn swap(pair) {
    let [a, b] = pair
    return [b, a]
}
```

---
[← Structs](STRUCT.html) &nbsp;&nbsp; [Modules →](MODULE_SYSTEM.html)
