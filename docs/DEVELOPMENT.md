# Aether Development Guide

Practical guide for building, testing, and extending the Aether Java interpreter.

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| JDK | 25 | `JAVA_HOME=/opt/homebrew/opt/openjdk@25` on macOS |
| Maven | 3.6+ | `mvn` |
| Gradle | 8.x | optional — identical outputs to Maven |

Check your setup:
```bash
$JAVA_HOME/bin/java -version   # must print openjdk 25
mvn -version
```

---

## Build Commands

### Maven (primary)

```bash
# Compile
mvn compile

# Run all tests
mvn test

# Package fat JAR (includes all dependencies)
mvn package
# → target/aether.jar

# Format code (Spotless / Google Java Format)
mvn spotless:apply

# Check formatting without modifying files
mvn spotless:check

# Checkstyle
mvn checkstyle:check

# Full verify (tests + spotless + checkstyle)
mvn verify
```

### Gradle (alternative)

```bash
# Run all tests
gradle test

# Build fat JAR
gradle shadowJar
# → build/libs/aether.jar

# Format
gradle spotlessApply

# Check
gradle check
```

### Running the interpreter

```bash
# File mode
java --enable-preview -jar target/aether.jar myprogram.ae

# REPL mode
java --enable-preview -jar target/aether.jar
```

---

## Project Layout

```
src/
├── main/java/com/aether/
│   ├── Main.java                      # CLI entry point
│   ├── Repl.java                      # Interactive REPL
│   ├── exception/                     # Typed runtime/parse/lex errors
│   ├── lexer/                         # Scanner, Token, TokenKind
│   ├── parser/                        # Parser + AST (Expr, Stmt records)
│   └── interpreter/                   # Evaluator, Environment, Value, Builtins
├── main/resources/stdlib/             # Standard library (.ae files)
└── test/java/com/aether/
    ├── lexer/ScannerTest.java
    ├── parser/ParserTest.java
    └── interpreter/EvaluatorTest.java
```

---

## Where to Add New Features

| Task | File |
|------|------|
| New token type | `lexer/TokenKind.java` + `lexer/Scanner.java` |
| New AST node | `parser/ast/Expr.java` or `Stmt.java` + `Parser.java` |
| New built-in function | `interpreter/Builtins.java` + `Evaluator.registerBuiltins()` |
| New stdlib function | `main/resources/stdlib/*.ae` |
| New member method | `Evaluator.evalMember()` / `evalCall()` |
| New runtime value type | `interpreter/Value.java` (add record to sealed interface) |

**Builtin vs Stdlib decision rule**: if it can be written in Aether without touching interpreter internals, put it in a `.ae` stdlib file.

---

## Code Style

Enforced automatically by Spotless (Google Java Format 1.23.0) and Checkstyle.

Run `mvn spotless:apply` before committing to auto-format.

Key conventions:
- 2-space indent, 100-column line limit
- `lowerCamelCase` for methods and fields
- `UpperCamelCase` for classes and records
- Use records for immutable data; `@Getter` (Lombok) on mutable classes
- Sealed interfaces + pattern-matching `switch` for AST and Value dispatch
- No raw `instanceof` checks where a sealed switch can be used

---

## TDD Workflow

1. Write a failing test in the appropriate `*Test.java`
2. Run `mvn test -Dtest=<TestClass>` — confirm it fails
3. Write minimal code to make it pass
4. Run the full suite: `mvn test`
5. Format: `mvn spotless:apply`
6. Commit

---

## Common Pitfalls

### Explicit `return` required

Aether functions do **not** return the last expression implicitly. The body must have an explicit `return` statement, or the function returns `null`.

```aether
# Wrong — returns null
fn add(a, b) { a + b }

# Correct
fn add(a, b) { return a + b }
```

This also affects struct methods and anonymous functions.

### Dict literals are expression-only

`{ }` at statement level always parses as a block. Dict literals only appear in expression context (right side of `let`, function argument, etc.).

```aether
# Wrong — parses as a block statement
{"a": 1}

# Correct
let d = {"a": 1}
```

### Struct init vs block disambiguation

The parser uses a one-token lookahead: if the token after `{` is `identifier:` or `}`, it treats the construct as a struct initialiser. Otherwise it's a block.

### `--enable-preview` is required everywhere

The fat JAR manifest includes `Multi-Release: true`, but the JVM still needs `--enable-preview` at runtime because preview features are opt-in per launch. Always run with:

```bash
java --enable-preview -jar aether.jar
```

---

## Adding a Built-in Function

1. Add a static factory method to `Builtins.java`:

```java
public static Value.Builtin myFunc() {
  return builtin("my_func", 1, args -> {
    // args.get(0) is the first argument
    return Value.Null.INSTANCE;
  });
}
```

2. Register it in `Evaluator.registerBuiltins()`:

```java
env.define("my_func", Builtins.myFunc());
```

3. Write a test in `EvaluatorTest.java`:

```java
@Test
void myFunc() {
  assertEquals("expected", eval("my_func(arg)"));
}
```

---

## Adding a Stdlib Function

Create or extend a file in `src/main/resources/stdlib/`:

```aether
fn my_util(x) {
    return x
}
```

It will be available in all Aether programs automatically (loaded at interpreter startup).

---

## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) — tree-walking interpreter patterns
- [Java Language Spec — Pattern Matching](https://openjdk.org/jeps/441)
- [Lombok docs](https://projectlombok.org/features/)
- [JLine 3 docs](https://github.com/jline/jline3)

---

**Last Updated**: April 17, 2026
**Implementation**: Java 25 — Maven + Gradle
