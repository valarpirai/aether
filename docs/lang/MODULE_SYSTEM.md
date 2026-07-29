---
layout: default
title: "Aether — Modules"
---

[Home](../index.md) › Language Reference › Modules

# Modules

Modules let you split code across files and reuse it across programs. Aether also ships a standard library of built-in modules.

## Import syntax

```aether
import math                         // import entire module
import math as m                    // import with alias
from math import abs, min, max      // import specific names
from math import abs as absolute    // import with rename
```

## Using a module

After `import math`, access its functions via the module namespace:

```aether
import math

println(math.abs(-5))   // 5
println(math.max(3, 7)) // 7
```

After `from math import abs`, the name is in the current scope:

```aether
from math import abs

println(abs(-5))   // 5
```

## User modules

Create a file `utils.ae` with your functions, then import it from another file:

```aether
// utils.ae
fn greet(name) {
    return "Hello, ${name}!"
}
```

```aether
// main.ae
import utils

fn main() {
    println(utils.greet("Alice"))
}
```

## Module resolution order

1. `<name>.ae` in the same directory as the importing file
2. `modules/<name>.ae` relative to the importing file
3. Embedded standard library modules (`math`, `collections`, `string`, `core`, `testing`)

## Standard library modules

| Module | What it provides |
|--------|-----------------|
| `core` | `range()`, `enumerate()` |
| `collections` | `map()`, `filter()`, `reduce()`, `find()`, `every()`, `some()` |
| `math` | `abs()`, `min()`, `max()`, `sum()`, `clamp()`, `sign()` |
| `string` | `join()`, `repeat()`, `reverse()`, `starts_with()`, `ends_with()` |
| `testing` | `assert_eq()`, `test()`, `test_summary()`, etc. |

## Error handling

```aether
// Module not found
import unknown_module   // Error: Module not found: 'unknown_module'

// Member not found
from math import no_such_fn  // Error: Cannot import 'no_such_fn' from module 'math'
```

## Limitations

- Module paths are flat — `import utils.helper` is not supported
- No directory modules (`__init__.ae`)
- No relative imports (`.` or `..`)
- No export control — all top-level definitions are visible to importers
- No dynamic imports (module name must be a literal string)

---
[← Destructuring](DESTRUCTURING.md) &nbsp;&nbsp; [Standard Library →](STDLIB.md)
