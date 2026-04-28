---
layout: default
title: Structs in Aether
---

# Structs in Aether

**Status**: ✅ Complete  
**Added**: Phase 5 Sprint 2  
**Tests**: 14 tests passing

## Overview

Structs (structures) are user-defined types that group related data and behavior together. They provide object-oriented programming capabilities in Aether with fields, methods, and the `self` keyword.

## Table of Contents
- [Declaring Structs](#declaring-structs)
- [Creating Instances](#creating-instances)
- [Accessing Fields](#accessing-fields)
- [Mutating Fields](#mutating-fields)
- [Methods](#methods)
- [The `self` Keyword](#the-self-keyword)
- [Default Values](#default-values)
- [Type Introspection](#type-introspection)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Declaring Structs

Define a struct using the `struct` keyword, followed by the struct name and field list:

```aether
struct Point {
    x
    y
}
```

### With Methods

Structs can include methods defined with `fn`:

```aether
struct Rectangle {
    width
    height
    
    fn area(self) {
        return self.width * self.height
    }
}
```

## Creating Instances

Create instances using struct literal syntax:

```aether
let p = Point { x: 10, y: 20 }
```

### Partial Initialization

You can omit fields - they default to `null`:

```aether
let p = Point { x: 5 }  // p.y is null
```

## Accessing Fields

Use dot notation to access fields:

```aether
struct Point { x, y }
let p = Point { x: 10, y: 20 }

println(p.x)  // 10
println(p.y)  // 20
```

## Mutating Fields

Fields are mutable - you can reassign them:

```aether
struct Point { x, y }
let p = Point { x: 1, y: 2 }

p.x = 100  // Mutate the field
println(p.x)  // 100
```

**Note**: Field mutation modifies the instance in place thanks to `RefCell` interior mutability.

## Methods

Methods are functions associated with a struct that take `self` as the first parameter.

### Defining Methods

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
```

### Calling Methods

```aether
let c = Counter { count: 0 }
c.increment()
c.increment()
println(c.get())  // 2
```

## The `self` Keyword

`self` refers to the current instance within methods:

```aether
struct Circle {
    radius
    
    fn area(self) {
        return 3.14159 * self.radius * self.radius
    }
    
    fn circumference(self) {
        return 2 * 3.14159 * self.radius
    }
}

let c = Circle { radius: 5 }
println(c.area())           // 78.53975
println(c.circumference())  // 31.4159
```

### Mutating via `self`

Methods can mutate fields through `self`:

```aether
struct BankAccount {
    balance
    
    fn deposit(self, amount) {
        self.balance = self.balance + amount
    }
    
    fn withdraw(self, amount) {
        self.balance = self.balance - amount
    }
}

let account = BankAccount { balance: 100 }
account.deposit(50)
account.withdraw(30)
println(account.balance)  // 120
```

## Default Values

Unspecified fields default to `null`:

```aether
struct Person {
    name
    age
    email
}

let p = Person { name: "Alice", age: 30 }
println(p.name)   // Alice
println(p.email)  // null
```

## Type Introspection

Use `type()` to get the struct name:

```aether
struct Point { x, y }
let p = Point { x: 1, y: 2 }

println(type(p))  // Point
```

Use `str()` to get string representation:

```aether
println(str(p))  // Point { x: 1, y: 2 }
```

## Examples

### Example 1: 2D Point

```aether
struct Point {
    x
    y
    
    fn distance_from_origin(self) {
        return (self.x * self.x + self.y * self.y)
    }
}

fn main() {
    let p = Point { x: 3, y: 4 }
    println("Point:", p)
    println("Distance²:", p.distance_from_origin())  // 25
}
```

### Example 2: Shopping Cart

```aether
struct ShoppingCart {
    items
    total
    
    fn add_item(self, name, price) {
        self.items.push(name)
        self.total = self.total + price
    }
    
    fn checkout(self) {
        println("Total:", self.total)
        return self.total
    }
}

fn main() {
    let cart = ShoppingCart { items: [], total: 0 }
    cart.add_item("Apple", 1)
    cart.add_item("Banana", 2)
    cart.checkout()  // Total: 3
}
```

### Example 3: Counter with Reset

```aether
struct Counter {
    count
    initial
    
    fn increment(self) {
        self.count = self.count + 1
    }
    
    fn decrement(self) {
        self.count = self.count - 1
    }
    
    fn reset(self) {
        self.count = self.initial
    }
}

fn main() {
    let c = Counter { count: 0, initial: 0 }
    c.increment()
    c.increment()
    c.increment()
    println(c.count)  // 3
    c.reset()
    println(c.count)  // 0
}
```

### Example 4: Rectangle Methods

```aether
struct Rectangle {
    width
    height
    
    fn area(self) {
        return self.width * self.height
    }
    
    fn perimeter(self) {
        return 2 * (self.width + self.height)
    }
    
    fn is_square(self) {
        return self.width == self.height
    }
}

fn main() {
    let r = Rectangle { width: 5, height: 10 }
    println("Area:", r.area())          // 50
    println("Perimeter:", r.perimeter())  // 30
    println("Is square?", r.is_square())  // false
    
    let s = Rectangle { width: 5, height: 5 }
    println("Square?", s.is_square())     // true
}
```

## Best Practices

### 1. Naming Conventions
- Use PascalCase for struct names: `Point`, `BankAccount`, `ShoppingCart`
- Use snake_case for field names: `first_name`, `total_price`

### 2. Initialize All Fields
While partial initialization is allowed, it's better to be explicit:

```aether
// Good
let p = Point { x: 0, y: 0 }

// Works but less clear
let p = Point { x: 0 }  // y defaults to null
```

### 3. Use Methods for Behavior
Encapsulate behavior in methods rather than external functions:

```aether
// Good
r.area()

// Less idiomatic
fn calculate_area(rect) { ... }
calculate_area(r)
```

### 4. Keep Structs Focused
Each struct should have a single, clear responsibility:

```aether
// Good
struct User { name, email }
struct Order { user, items, total }

// Avoid mixing concerns
struct UserOrder { name, email, items, total }  // Less clear
```

### 5. Return Values from Methods
When methods compute values, return them:

```aether
struct Circle {
    radius
    
    fn area(self) {
        return 3.14159 * self.radius * self.radius  // Return the result
    }
}
```

## Implementation Details

### Memory Management
- Struct instances use `Rc<RefCell<HashMap>>` for fields
- Fields are mutable via RefCell interior mutability
- Automatic garbage collection via reference counting

### Method Binding
- Methods are stored in the struct definition
- `self` is automatically bound when methods are called
- Methods have access to all fields via `self`

### Type System
- Each struct declaration creates a new type
- Instances know their struct type name
- Type checking is dynamic at runtime

## Limitations

**No Inheritance**: Aether structs don't support inheritance. Use composition instead:

```aether
struct Engine { power }
struct Car { engine, model }  // Composition

let car = Car { 
    engine: Engine { power: 200 },
    model: "Sedan"
}
```

**No Static Fields/Methods**: All fields and methods are instance-specific.

**No Constructors**: Use regular functions to create instances with validation:

```aether
fn new_point(x, y) {
    return Point { x: x, y: y }
}
```

## See Also

- [DESIGN.md](DESIGN.html) - Language specification
- [INTERPRETER.md](INTERPRETER.html) - Runtime implementation
- [examples/shapes.ae](../examples/shapes.ae) - Struct examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete and stable
