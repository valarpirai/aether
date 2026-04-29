---
layout: default
title: Aether — Example Programs
---

# Example Programs

Real Aether programs covering the full language. Each section runs with `cargo run -- examples/<file>.ae`.

---

## Hello World {#hello}

```aether
fn greet(name) {
    return "Hello, " + name + "!"
}

fn main() {
    let message = greet("Aether")
    println(message)

    let name = "World"
    println("Hello, ${name}!")

    let numbers = [1, 2, 3, 4, 5]
    println("Numbers:", numbers)
}
```

---

## Null Safety — `??` and `?.` {#null-safety}

```aether
fn main() {
    // ?? returns right side when left is null
    let name = null
    println(name ?? "anonymous")          // anonymous

    let score = 0 ?? 100
    println(score)                        // 0 (not null, stays 0)

    // ?. returns null when object is null (no error)
    let s = null
    println(s?.upper())                   // null
    println(s?.length)                    // null

    // Combine ?. with ?? for safe access with fallback
    println(s?.upper() ?? "UNKNOWN")      // UNKNOWN

    let real = "hello"
    println(real?.upper() ?? "UNKNOWN")   // HELLO

    // Works on arrays and their methods
    let arr = null
    println(arr?.length ?? 0)             // 0
    println(arr?.contains(1))             // null
}
```

---

## Multi-line Strings {#multiline-strings}

```aether
fn main() {
    // Triple-quoted strings: raw content, leading newline stripped
    let query = """
SELECT *
FROM users
WHERE active = true
ORDER BY name"""

    println(query)

    // Great for embedded templates or multi-line messages
    let message = """
Dear user,

Your account has been updated.
Please log in to see the changes.

Thanks,
The Aether Team"""

    println(message)

    // Works with string interpolation via concatenation
    let table = "users"
    let condition = "active = true"
    let sql = "SELECT * FROM " + table + " WHERE " + condition
    println(sql)
}
```

---

## Error Handling with `finally` {#error-handling}

```aether
fn risky_op(x) {
    if x < 0 {
        throw "negative value: " + x
    }
    return x * 2
}

fn main() {
    // finally runs whether or not an exception is thrown
    try {
        let result = risky_op(5)
        println("success:", result)
    } catch(e) {
        println("error:", e.message)
    } finally {
        println("cleanup complete")
    }

    println("---")

    try {
        let result = risky_op(-1)
        println("success:", result)
    } catch(e) {
        println("error:", e.message)
    } finally {
        println("cleanup complete")
    }
}
```

---

## FizzBuzz {#fizzbuzz}

```aether
fn fizzbuzz(n) {
    let i = 1
    while (i <= n) {
        if (i % 15 == 0) {
            println("FizzBuzz")
        } else if (i % 3 == 0) {
            println("Fizz")
        } else if (i % 5 == 0) {
            println("Buzz")
        } else {
            println(i)
        }
        i += 1
    }
}

fn main() {
    println("FizzBuzz 1..20:")
    fizzbuzz(20)
}
```

---

## Fibonacci {#fibonacci}

Recursive and iterative implementations:

```aether
fn fib_recursive(n) {
    if (n <= 1) { return n }
    return fib_recursive(n - 1) + fib_recursive(n - 2)
}

fn fib_iterative(n) {
    if (n <= 1) { return n }
    let a = 0
    let b = 1
    let i = 2
    while (i <= n) {
        let temp = a + b
        a = b
        b = temp
        i += 1
    }
    return b
}

fn main() {
    println("Fibonacci sequence (first 10):")
    let i = 0
    while (i < 10) {
        print(fib_iterative(i), " ")
        i += 1
    }
    println()
}
```

---

## Shapes — Structs {#shapes}

```aether
struct Circle {
    radius

    fn area(self) {
        return 3.14159 * self.radius * self.radius
    }

    fn perimeter(self) {
        return 2.0 * 3.14159 * self.radius
    }

    fn describe(self) {
        return "Circle(r=" + str(self.radius) + ")"
    }
}

struct Rectangle {
    width
    height

    fn area(self) { return self.width * self.height }
    fn perimeter(self) { return 2 * (self.width + self.height) }

    fn describe(self) {
        return "Rectangle(" + str(self.width) + "x" + str(self.height) + ")"
    }
}

fn main() {
    let c = Circle { radius: 5.0 }
    let r = Rectangle { width: 4, height: 6 }

    println(c.describe())
    println("  area:      ", c.area())
    println("  perimeter: ", c.perimeter())

    println(r.describe())
    println("  area:      ", r.area())
    println("  perimeter: ", r.perimeter())
}
```

---

## Task Manager — Real-world Structs {#task-manager}

```aether
struct Task {
    id
    title
    done

    fn complete(self) { self.done = true }

    fn display(self) {
        let status = "[ ]"
        if (self.done) { status = "[x]" }
        return status + " #" + str(self.id) + " " + self.title
    }
}

struct TaskList {
    tasks
    next_id

    fn add(self, title) {
        let t = Task { id: self.next_id, title: title, done: false }
        self.tasks.push(t)
        self.next_id = self.next_id + 1
    }

    fn complete(self, id) {
        for task in self.tasks {
            if (task.id == id) {
                task.complete()
                return true
            }
        }
        return false
    }

    fn pending_count(self) {
        let count = 0
        for task in self.tasks {
            if (!task.done) { count += 1 }
        }
        return count
    }

    fn print_all(self) {
        for task in self.tasks { println(task.display()) }
    }
}

fn main() {
    let list = TaskList { tasks: [], next_id: 1 }

    list.add("Buy groceries")
    list.add("Write Aether examples")
    list.add("Fix remaining bugs")
    list.add("Update documentation")

    println("All tasks:")
    list.print_all()
    println()

    list.complete(2)
    list.complete(4)

    println("After completing tasks 2 and 4:")
    list.print_all()
    println()
    println("Pending:", list.pending_count())
}
```

---

## Error Handling with Stack Traces {#error-context}

```aether
fn divide(a, b) {
    if (b == 0) {
        throw "division by zero: cannot divide " + str(a) + " by 0"
    }
    return a / b
}

fn calculate(values) {
    let total = 0
    for v in values {
        total = total + divide(v, v - v)   // v-v == 0 intentionally
    }
    return total
}

fn nested_a() { throw "deep error" }
fn nested_b() { nested_a() }
fn nested_c() { nested_b() }

fn main() {
    // Thrown error with stack trace
    try {
        calculate([10, 20, 30])
    } catch(e) {
        println("message:", e.message)
        println("stack trace:")
        println(e.stack_trace)
    }

    // Deep call stack
    try {
        nested_c()
    } catch(e) {
        println("deep error:", e.message)
        println(e.stack_trace)
    }

    // Runtime error (undefined variable)
    try {
        let x = no_such_variable
    } catch(e) {
        println("runtime error:", e.message)
    }

    // Re-throw
    try {
        try {
            throw "original problem"
        } catch(inner) {
            throw "wrapped: " + inner.message
        }
    } catch(outer) {
        println("outer caught:", outer.message)
    }
}
```

---

## Data Processing — Functional Pipeline {#data-processing}

```aether
fn main() {
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    let squares   = map(numbers, fn(x) { return x * x })
    let evens     = filter(numbers, fn(x) { return x % 2 == 0 })
    let total     = reduce(numbers, fn(acc, x) { return acc + x }, 0)

    println("Squares:", squares)
    println("Evens:", evens)
    println("Sum:", total)

    // Chain: sum of squares of odd numbers
    let odd_sq_sum = reduce(
        map(filter(numbers, fn(x) { return x % 2 != 0 }), fn(x) { return x * x }),
        fn(acc, x) { return acc + x },
        0
    )
    println("Sum of odd squares:", odd_sq_sum)

    // Word frequency
    let words = ["apple", "banana", "apple", "cherry", "banana", "apple"]
    let freq = {}
    for word in words {
        try {
            freq[word] = freq[word] + 1
        } catch(e) {
            freq[word] = 1
        }
    }
    println("apple:", freq["apple"])
    println("banana:", freq["banana"])
}
```

---

## Collections — Arrays, Dicts, Sets {#collections}

```aether
fn main() {
    // Arrays
    let arr = [3, 1, 4, 1, 5, 9, 2, 6]
    println("original:", arr)
    println("sorted:", arr.sort())
    println("reversed:", arr.sort().reverse())
    println("slice [2:5]:", arr[2:5])
    println("len:", len(arr))

    // Array methods
    let a = [1, 2, 3]
    a.push(4)
    let last = a.pop()
    println("pop:", last, "remaining:", a)

    // Spread
    let b = [0, ...a, 5]
    println("spread:", b)

    // Dicts
    let person = {"name": "Alice", "age": 30, "city": "London"}
    println("name:", person["name"])
    println("keys:", person.keys())
    println("values:", person.values())
    println("contains 'age':", person.contains("age"))

    // Sets
    let s1 = set([1, 2, 3, 4, 5])
    let s2 = set([3, 4, 5, 6, 7])
    println("union:", s1.union(s2))
    println("intersection:", s1.intersection(s2))
    println("difference:", s1.difference(s2))
}
```

---

## File Utilities {#file-utilities}

```aether
fn basename(path) {
    let parts = path.split("/")
    return parts[len(parts) - 1]
}

fn dirname(path) {
    let parts = path.split("/")
    let dir = ""
    let i = 0
    while (i < len(parts) - 1) {
        if (i > 0) { dir = dir + "/" }
        dir = dir + parts[i]
        i = i + 1
    }
    return dir
}

fn grep(path, pattern) {
    let matches = []
    let n = 0
    for line in lines_iter(path) {
        n = n + 1
        if (line.contains(pattern)) {
            matches.push(str(n) + ": " + line)
        }
    }
    return matches
}

fn head(path, n) {
    let result = []
    let count = 0
    for line in lines_iter(path) {
        if (count >= n) { break }
        result.push(line)
        count = count + 1
    }
    return result
}

fn read_config(path) {
    let config = {}
    for line in lines_iter(path) {
        let trimmed = line.trim()
        if (len(trimmed) == 0 || trimmed[0] == "#") { continue }
        let eq = trimmed.index_of("=")
        if (eq < 0) { continue }
        config[trimmed[0:eq].trim()] = trimmed[eq + 1:len(trimmed)].trim()
    }
    return config
}

fn main() {
    // Path helpers
    let path = "/home/user/projects/aether/src/main.ae"
    println("basename:", basename(path))
    println("dirname:", dirname(path))

    // Write and read a config file
    write_file("/tmp/app.cfg", "host=localhost\nport=8080\n# comment\ndebug=true\n")
    let cfg = read_config("/tmp/app.cfg")
    println("host:", cfg["host"])
    println("port:", cfg["port"])

    // Write sample log and grep it
    write_file("/tmp/app.log", "INFO server started\nERROR disk full\nINFO request ok\nERROR timeout\n")
    let errors = grep("/tmp/app.log", "ERROR")
    println("errors found:", len(errors))
    for e in errors { println(" ", e) }

    // Head of file
    println("head(2):", head("/tmp/app.log", 2))

    // File predicates
    println("exists:", file_exists("/tmp/app.log"))
    println("is_file:", is_file("/tmp/app.log"))
    println("is_dir:", is_dir("/tmp"))

    // Directory listing
    let entries = list_dir("/tmp")
    println("files in /tmp:", len(entries))

    // Path joining
    let base = "/home/user"
    let full = path_join(base, "projects", "aether", "src")
    println("joined:", full)

    // Rename and remove
    write_file("/tmp/old_name.txt", "content")
    rename("/tmp/old_name.txt", "/tmp/new_name.txt")
    println("renamed:", file_exists("/tmp/new_name.txt"))
    rm("/tmp/new_name.txt")
    println("removed:", file_exists("/tmp/new_name.txt"))
}
```

---

## String Utilities {#string-utilities}

```aether
fn main() {
    let s = "Hello, World!"
    println("upper:", s.upper())
    println("lower:", s.lower())
    println("trim:", "   spaces   ".trim())

    // Search
    let text = "the quick brown fox"
    println("contains 'fox':", text.contains("fox"))
    println("index_of 'fox':", text.index_of("fox"))
    println("starts_with 'the':", starts_with(text, "the"))
    println("ends_with 'fox':", ends_with(text, "fox"))

    // Replace
    println("replace:", "I like cats".replace("cats", "dogs"))

    // Split / join
    let csv = "alice,bob,carol"
    let names = csv.split(",")
    println("split:", names)
    println("join:", join(names, " | "))

    // Slice
    let lang = "Aether"
    println("lang[0:3]:", lang[0:3])
    println("last char:", lang[len(lang) - 1])

    // Repeat / reverse
    println("repeat:", repeat("ab", 3))
    println("reverse:", reverse("Aether"))

    // Interpolation
    let x = 42
    println("result: ${x * 2 + 1}")
}
```

---

## Async / Concurrent I/O {#async}

```aether
// Phase 1: Promise-based async (no thread pool needed)
async fn fetch_number() { return 42 }

async fn double_async(x) { return x * 2 }

async fn quadruple(x) {
    let a = await double_async(x)
    return await double_async(a)
}

fn main() {
    // Calling async fn returns a Promise
    let p = fetch_number()
    println("type:", type(p))           // promise

    // await resolves it
    let n = await fetch_number()
    println("awaited:", n)              // 42

    // Chain async calls
    println("quadruple(5):", await quadruple(5))    // 20

    // await non-Promise is identity
    println("await 42:", await 42)      // 42

    // Phase 2: concurrent I/O with thread pool
    // set_workers(4)
    // let p1 = http_get("https://httpbin.org/get")
    // let p2 = http_get("https://httpbin.org/ip")
    // let results = await Promise.all([p1, p2])
    // println(results[0])
}
```

---

## Standard Library Showcase {#stdlib}

```aether
fn main() {
    // range / enumerate
    println("range(5):", range(5))
    println("range(2,8):", range(2, 8))
    for pair in enumerate(["a", "b", "c"]) {
        println(pair[0], "->", pair[1])
    }

    // Math
    println("abs(-7):", abs(-7))
    println("min(3,7):", min(3, 7))
    println("max(3,7):", max(3, 7))
    println("clamp(15,0,10):", clamp(15, 0, 10))
    println("sum([1..5]):", sum(range(1, 6)))

    // Higher-order
    let nums = range(1, 11)
    println("map squares:", map(nums, fn(x) { return x * x }))
    println("filter evens:", filter(nums, fn(x) { return x % 2 == 0 }))
    println("reduce sum:", reduce(nums, fn(a, b) { return a + b }, 0))
    println("find >7:", find(nums, fn(x) { return x > 7 }))
    println("every >0:", every(nums, fn(x) { return x > 0 }))
    println("some >9:", some(nums, fn(x) { return x > 9 }))

    // String stdlib
    println("join:", join(["a","b","c"], "-"))
    println("repeat:", repeat("ha", 3))
    println("reverse:", reverse("Aether"))
    println("starts_with:", starts_with("hello", "he"))
    println("ends_with:", ends_with("hello", "lo"))
}
```
