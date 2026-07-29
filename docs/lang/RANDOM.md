---
layout: default
title: "Aether — Random"
---

[Home](../index.md) › Language Reference › Random

# Random

Aether provides two built-in random number functions: `random()` for a float and `rand_int(n)` for an integer.

## random()

Returns a random float in the range `[0, 1)`.

```aether
let r = random()
println(r)  // 0.7423619...
```

## rand_int(n)

Returns a random integer in the range `[0, n)`. `n` must be a positive int.

```aether
rand_int(6)    // int in [0, 6) — e.g. dice roll
rand_int(2)    // 0 or 1 — coin flip
```

## Examples

### Dice roll

```aether
fn main() {
    let roll = rand_int(6) + 1
    println("You rolled a ${roll}")
}
```

### Random element from an array

```aether
fn choice(arr) {
    return arr[rand_int(len(arr))]
}

fn main() {
    let colors = ["red", "green", "blue"]
    println(choice(colors))
}
```

## Limitations

- Not cryptographically secure — do not use for tokens, keys, or passwords
- No seeding support; each call draws from the OS-seeded thread-local RNG

## Related

- [Time](TIME.md) — `clock()`, `sleep()`

---
[← Time](TIME.md) &nbsp;&nbsp; [Async →](ASYNC.md)
