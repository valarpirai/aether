---
layout: default
title: "Aether — Structs"
---

[Home](../index.md) › Language Reference › Structs

# Structs

A struct is a user-defined type that groups related fields and methods together.

## Declaration

```aether
struct Point {
    x
    y
}
```

With methods:

```aether
struct Rectangle {
    width
    height

    fn area(self) {
        return self.width * self.height
    }
}
```

## Creating instances

Use struct literal syntax. Omitted fields default to `null`.

```aether
let p = Point { x: 10, y: 20 }
let r = Rectangle { width: 5, height: 10 }
let partial = Point { x: 5 }  // p.y is null
```

## Fields

Access and mutate fields with dot notation:

```aether
println(p.x)   // 10
p.x = 100      // mutate in place
println(p.x)   // 100
```

## Methods

Methods take `self` as their first parameter. `self` refers to the current instance.

```aether
struct Counter {
    count

    fn increment(self) {
        self.count = self.count + 1
    }

    fn get(self) {
        return self.count
    }
}

let c = Counter { count: 0 }
c.increment()
c.increment()
println(c.get())  // 2
```

Methods can accept additional parameters beyond `self`:

```aether
struct BankAccount {
    balance

    fn deposit(self, amount) {
        self.balance = self.balance + amount
    }
}
```

## Type introspection

`type()` returns the struct name; `str()` returns a readable representation:

```aether
struct Point { x, y }
let p = Point { x: 1, y: 2 }

println(type(p))  // Point
println(str(p))   // Point { x: 1, y: 2 }
```

## Examples

### 2D point with distance

```aether
struct Point {
    x
    y

    fn distance_squared(self) {
        return self.x * self.x + self.y * self.y
    }
}

fn main() {
    let p = Point { x: 3, y: 4 }
    println(p.distance_squared())  // 25
}
```

### Shopping cart

```aether
struct Cart {
    items
    total

    fn add(self, name, price) {
        self.items.push(name)
        self.total = self.total + price
    }
}

fn main() {
    let cart = Cart { items: [], total: 0 }
    cart.add("Apple", 1)
    cart.add("Banana", 2)
    println("Total: ${cart.total}")  // Total: 3
}
```

### Composition instead of inheritance

```aether
struct Engine { power }
struct Car { engine, model }

let car = Car {
    engine: Engine { power: 200 },
    model: "Sedan"
}
println(car.engine.power)  // 200
```

## Limitations

- No inheritance — use composition
- No static fields or class-level methods
- No built-in constructors — use a plain function if you need initialization logic:

```aether
fn new_point(x, y) {
    return Point { x: x, y: y }
}
```

---
[← Error Handling](ERROR_HANDLING.md) &nbsp;&nbsp; [Destructuring →](DESTRUCTURING.md)
