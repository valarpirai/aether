# Aether Programming Language

A general-purpose, dynamically typed programming language — C-like syntax, no semicolons, tree-walking interpreter written in Java.

## Quick Start

### Requirements

- JDK 25 ([Homebrew](https://brew.sh): `brew install openjdk@25`)
- Maven 3.6+ (`brew install maven`)

### Build & run

```bash
# Build fat JAR (includes all dependencies)
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn package -q

# Run a program
java --enable-preview -jar target/aether.jar myprogram.ae

# Start the interactive REPL
java --enable-preview -jar target/aether.jar
```

### Example program

```aether
fn main() {
    let words = ["hello", "beautiful", "world"]
    let sentence = join(words, " ")
    println("Original: " + sentence)
    println("Uppercase: " + sentence.upper())
    println("Reversed: " + reverse(sentence))

    let nums = range(1, 6)
    let squares = map(nums, fn(x) { return x * x })
    println("Squares: ${squares}")
    println("Sum: ${sum(squares)}")
}
```

## Language Features

| Feature | Syntax |
|---------|--------|
| Variables | `let x = 42` |
| Functions | `fn add(a, b) { return a + b }` |
| Closures | `fn(x) { return x * 2 }` |
| String interpolation | `"Hello ${name}!"` |
| Arrays | `[1, 2, 3]`, `arr.push(4)`, `arr[1:3]` |
| Dicts | `{"key": value}`, `d["key"]` |
| Structs | `struct Point { x, y fn dist(self) { ... } }` |
| Error handling | `try { ... } catch(e) { ... }` / `throw value` |
| Modules | `import math`, `from collections import map` |
| For-each | `for item in array { ... }` |
| While | `while (cond) { ... }` |
| Types | `int`, `float`, `string`, `bool`, `null`, `array`, `dict` |

## Standard Library

The stdlib is written in Aether and loaded automatically.

| Module | Functions |
|--------|-----------|
| core | `range(n)`, `range(start, end)`, `enumerate(arr)` |
| collections | `map`, `filter`, `reduce`, `find`, `every`, `some`, `sort`, `concat` |
| math | `abs`, `min`, `max`, `sum`, `clamp`, `sign` |
| string | `join`, `repeat`, `reverse`, `starts_with`, `ends_with` |
| testing | `assert_eq`, `assert_true`, `test`, `test_summary` |

## Built-in Functions

`print`, `println`, `input`, `read_file`, `write_file`, `type`, `len`, `int`, `float`, `str`, `bool`, `clock`, `sleep`, `json_parse`, `json_stringify`

## REPL

```
>> let name = "Aether"
>> "Hello ${name}!"
Hello Aether!
>> fn square(n) { return n * n }
>> square(9)
81
>> help
Commands:
  help  — show this message
  env   — list all defined variables
  Ctrl+D — exit
```

## Development

```bash
# Run all tests (203 tests)
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn test

# Run a specific test class
mvn test -Dtest=StdlibTest

# Format code (Google Java Format)
mvn spotless:apply

# Full verify (tests + format + checkstyle)
mvn verify
```

### Test breakdown

| Suite | Tests | Coverage |
|-------|-------|---------|
| `ScannerTest` | 16 | Tokenisation |
| `ParserTest` | 36 | AST structure |
| `EvaluatorTest` | 47 | Core language features |
| `StdlibTest` | 51 | All stdlib modules |
| `MoreEvaluatorTest` | 53 | JSON, modules, closures, error propagation |
| **Total** | **203** | |

## Distribution

```bash
# Fat JAR (no install needed, just JDK 25)
mvn package
java --enable-preview -jar target/aether.jar

# Gradle (alternative)
gradle shadowJar
java --enable-preview -jar build/libs/aether.jar
```

## Documentation

| Doc | Content |
|-----|---------|
| [docs/DESIGN.md](docs/DESIGN.md) | Language specification |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design and structure |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Build commands, conventions, pitfalls |
| [docs/TESTING.md](docs/TESTING.md) | Test guide |
| [docs/REPL.md](docs/REPL.md) | REPL reference |
| [docs/STDLIB.md](docs/STDLIB.md) | Standard library reference |

## License

MIT — see [LICENSE](LICENSE).
