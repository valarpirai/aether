# CLAUDE.md

Guidance for Claude Code when working with the Aether repository.

## Project Overview

Aether is a general-purpose programming language — dynamic, interpreted (tree-walking), with C-like syntax and no semicolons. This repository contains the **Java 25 implementation** on the `java-port` branch.

### Language characteristics
- Dynamic typing with runtime type checking
- First-class functions, closures, optional parameters
- `int`, `float`, `string`, `bool`, `null`, `array`, `dict`
- User-defined structs with methods and `self`
- String interpolation: `"Hello ${name}"`
- Error handling: `try/catch/throw`
- Module system: `import`, `from ... import`, `import ... as`
- File extension: `.ae`

---

## Build & Test (Java)

```bash
# Requires Java 25
export JAVA_HOME=/opt/homebrew/opt/openjdk@25

# Run all tests
mvn test

# Run one test class
mvn test -Dtest=EvaluatorTest

# Build fat JAR (target/aether.jar)
mvn package

# Run the interpreter
java --enable-preview -jar target/aether.jar [file.ae]

# Format code
mvn spotless:apply

# Full verify (tests + format + checkstyle)
mvn verify
```

---

## Project Structure

```
src/main/java/com/aether/
├── Main.java                      # CLI: file mode or REPL
├── Repl.java                      # Interactive REPL (JLine 3)
├── exception/
│   ├── AetherRuntimeException.java  # Sealed runtime error hierarchy
│   ├── LexerException.java
│   └── ParseException.java
├── lexer/
│   ├── Scanner.java               # Tokeniser
│   ├── Token.java                 # Token record
│   ├── TokenKind.java             # Token type enum
│   └── StringPart.java            # Interpolation segment (Literal / Placeholder)
├── parser/
│   ├── Parser.java                # Recursive-descent parser
│   └── ast/
│       ├── Expr.java              # Sealed expression AST nodes (records)
│       ├── Stmt.java              # Sealed statement AST nodes (records)
│       ├── BinaryOp.java
│       └── UnaryOp.java
└── interpreter/
    ├── Evaluator.java             # Tree-walking evaluator
    ├── Environment.java           # Lexical scoping (parent-chain)
    ├── Value.java                 # Sealed runtime value types (records)
    ├── Builtins.java              # Native built-in functions + display()
    └── StdlibLoader.java          # Loads .ae stdlib files from classpath

src/main/resources/stdlib/         # Standard library (written in Aether)
    core.ae, collections.ae, math.ae, string.ae, testing.ae

src/test/java/com/aether/
    lexer/ScannerTest.java                 # 16 tests
    parser/ParserTest.java                 # 36 tests
    interpreter/EvaluatorTest.java         # 47 tests (core evaluator)
    interpreter/MoreEvaluatorTest.java     # 53 tests (JSON, modules, closures)
    interpreter/StdlibTest.java            # 41 tests (stdlib functions)
    interpreter/IntegrationTest.java       # 24 tests (complete programs)
    interpreter/FunctionExprTest.java      # 13 tests (fn expressions)
    interpreter/StringInterpTest.java      # 14 tests (interpolation)
    interpreter/StringIndexingTest.java    # 15 tests (s[i])
    interpreter/StringMethodsTest.java     # 16 tests (upper/lower/trim/split)
    interpreter/SliceAndSpreadTest.java    # 32 tests (slices, spread)
    interpreter/ArrayMethodsTest.java      # 15 tests (push/pop/sort/concat)
    interpreter/DictTest.java              # 17 tests (dict operations)
    interpreter/StructTest.java            # 14 tests (struct and methods)
    interpreter/MemberAccessTest.java      #  8 tests (.length, errors)
    interpreter/JsonTest.java              # 25 tests (json_parse/stringify)
    interpreter/ModuleTest.java            # 13 tests (module imports)
    interpreter/ErrorHandlingTest.java     # 10 tests (try/catch/throw)
    interpreter/IoAndTimeTest.java         # 13 tests (file I/O, clock)
    interpreter/TestingFrameworkTest.java  # 19 tests (stdlib testing)
```

---

## Where to Add Things

| Task | Primary file | Test file |
|------|-------------|-----------|
| New token | `lexer/TokenKind.java` + `Scanner.java` | `ScannerTest.java` |
| New AST node | `parser/ast/Expr.java` or `Stmt.java` + `Parser.java` | `ParserTest.java` |
| New built-in | `interpreter/Builtins.java` + `Evaluator.registerBuiltins()` | `IntegrationTest.java` |
| New stdlib fn | `resources/stdlib/*.ae` | `StdlibTest.java` |
| New array method | `Evaluator.evalArrayMethod()` | `ArrayMethodsTest.java` |
| New string method | `Evaluator.evalStringMethod()` | `StringMethodsTest.java` |
| New member method | `Evaluator.evalMember()` / `evalCall()` | `MemberAccessTest.java` |
| New Value type | `interpreter/Value.java` (add record to sealed interface) | `EvaluatorTest.java` |

---

## Key Implementation Notes

### Explicit `return` required
Aether functions do **not** implicitly return the last expression. Always use `return`:
```aether
fn add(a, b) { return a + b }          # correct
fn add(a, b) { a + b }                 # returns null!
```

### Dict literals are expression-context only
`{}` at statement level parses as a block. Use dict literals only on the right side of `let`:
```aether
let d = {"a": 1}    # correct
{"a": 1}            # parsed as a block, not a dict
```

### Pattern-matching switch
All AST and Value dispatch uses Java sealed-interface `switch`. When adding a new `Expr` or `Value` variant, the compiler will point out every switch that needs updating.

### Lombok
`@Getter` on `Environment`, `LexerException`, `ParseException` — do not add manual accessor methods to these classes.

### Fat JAR
`target/aether.jar` bundles all dependencies. Always launch with `--enable-preview`:
```bash
java --enable-preview -jar target/aether.jar
```

---

## Test Conventions

- `Evaluator.withoutStdlib()` in `@BeforeEach` — fast, clean state per test
- `eval("source")` helper returns the display string of the last expression
- Statements (if/while/let) don't produce a value — end with a variable reference: `eval("let x = 1\nx")`
- `assertThrows(AetherRuntimeException.Foo.class, () -> eval("..."))` for error paths

---

## Documentation Index

| File | Content |
|------|---------|
| [docs/DESIGN.md](docs/DESIGN.md) | Language specification |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, project structure |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Build commands, code style, pitfalls |
| [docs/TESTING.md](docs/TESTING.md) | Test guide, writing tests, debugging |
| [docs/REPL.md](docs/REPL.md) | REPL commands and file execution |
| [docs/STDLIB.md](docs/STDLIB.md) | Standard library reference |

---

## Current Status

**Branch**: `java-port`
**Tests**: 451 passing (16 scanner + 36 parser + 399 interpreter)
**Distribution**: fat JAR (`mvn package` → `target/aether.jar`)

### Completed
- Lexer, Parser, Evaluator — full feature parity with Rust implementation
- REPL with JLine 3 (history, line editing, `help`/`env` commands)
- All builtins: print, println, input, read_file, write_file, type, len, int, float, str, bool, clock, sleep, json_parse, json_stringify
- Standard library: core, collections, math, string, testing modules
- Structs with fields, methods, self, mutation
- Error handling: try/catch/throw
- Module system: import, from...import, import...as
- Array built-in methods: push, pop, sort(comparator), concat, contains, length
- Fat JAR packaging (Maven Shade + Gradle Shadow)
- 451 tests across 20 test classes (parity with Rust ~420 tests); 3 http tests disabled (require network)
- `http_get(url)`, `http_post(url, body)` via Java HttpClient (Java 11+)

### Next
- GraalVM native-image for zero-JVM startup binary
- Bytecode compiler (performance)
