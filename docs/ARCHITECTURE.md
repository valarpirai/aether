# Aether Architecture

High-level overview of Aether's architecture, implementation choices, and roadmap.

> For day-to-day development see [DEVELOPMENT.md](DEVELOPMENT.md) and [CLAUDE.md](../CLAUDE.md).

---

## Pipeline

```
Source Code (.ae)
      ↓
   [Scanner]  →  Tokens          com.aether.lexer.Scanner
      ↓
   [Parser]   →  AST             com.aether.parser.Parser
      ↓
[Evaluator]   →  Output          com.aether.interpreter.Evaluator
```

## Project Structure

```
src/
├── main/
│   ├── java/com/aether/
│   │   ├── Main.java                  # CLI entry point
│   │   ├── Repl.java                  # Interactive REPL (JLine 3)
│   │   ├── exception/
│   │   │   ├── AetherRuntimeException.java  # Sealed runtime errors
│   │   │   ├── LexerException.java
│   │   │   └── ParseException.java
│   │   ├── lexer/
│   │   │   ├── Scanner.java           # Tokeniser
│   │   │   ├── Token.java             # Token record
│   │   │   ├── TokenKind.java         # Token type enum
│   │   │   └── StringPart.java        # Interpolation segments
│   │   ├── parser/
│   │   │   ├── Parser.java            # Recursive-descent parser
│   │   │   └── ast/
│   │   │       ├── Expr.java          # Sealed expression nodes (records)
│   │   │       ├── Stmt.java          # Sealed statement nodes (records)
│   │   │       ├── BinaryOp.java
│   │   │       └── UnaryOp.java
│   │   └── interpreter/
│   │       ├── Evaluator.java         # Tree-walking evaluator
│   │       ├── Environment.java       # Lexical scoping
│   │       ├── Value.java             # Sealed runtime value types
│   │       ├── Builtins.java          # Native built-in functions
│   │       └── StdlibLoader.java      # Classpath stdlib loader
│   └── resources/stdlib/              # Standard library (.ae files)
│       ├── core.ae
│       ├── collections.ae
│       ├── math.ae
│       ├── string.ae
│       └── testing.ae
└── test/java/com/aether/
    ├── lexer/ScannerTest.java             # 16 tests
    ├── parser/ParserTest.java             # 36 tests
    └── interpreter/
        ├── EvaluatorTest.java             # 47 tests — core evaluator
        ├── MoreEvaluatorTest.java         # 53 tests — JSON, modules, closures
        ├── StdlibTest.java                # 41 tests — stdlib functions
        ├── IntegrationTest.java           # 24 tests — complete programs
        ├── FunctionExprTest.java          # 13 tests — fn expressions
        ├── StringInterpTest.java          # 14 tests — string interpolation
        ├── StringIndexingTest.java        # 15 tests — s[i] indexing
        ├── StringMethodsTest.java         # 16 tests — upper/lower/trim/split
        ├── SliceAndSpreadTest.java        # 32 tests — slices and spread
        ├── ArrayMethodsTest.java          # 15 tests — push/pop/sort/concat
        ├── DictTest.java                  # 17 tests — dict operations
        ├── StructTest.java                # 14 tests — struct and methods
        ├── MemberAccessTest.java          #  8 tests — .length property
        ├── JsonTest.java                  # 25 tests — json_parse/stringify
        ├── ModuleTest.java                # 13 tests — module imports
        ├── ErrorHandlingTest.java         # 10 tests — try/catch/throw
        ├── IoAndTimeTest.java             # 13 tests — file I/O, clock
        └── TestingFrameworkTest.java      # 19 tests — stdlib testing
```

## Key Design Choices

### Java 25 + sealed interfaces + records

Every AST node (`Expr`, `Stmt`) and every runtime value (`Value`) is a **sealed interface** whose permitted types are **records**. This gives exhaustive `switch` expressions at compile time — the same safety as Rust `enum` matching.

```java
// Pattern-matched exhaustively — compiler rejects missing cases
return switch (expr) {
  case Expr.IntLiteral(long v)  -> new Value.IntVal(v);
  case Expr.Binary(Expr l, BinaryOp op, Expr r) -> evalBinary(l, op, r);
  // ...
};
```

### Lombok

`@Getter` on mutable classes (`Environment`, exception types) eliminates boilerplate accessor methods. Records generate their own compact accessors automatically.

### Closures

`Value.AetherFunction` stores a reference to the `Environment` at definition time. Java's reference semantics mean captured variables stay alive as long as the closure exists — no explicit `Rc` cloning needed.

### Control flow

`break`, `continue`, and `return` are modelled as a private sealed `ControlFlow` interface inside `Evaluator`. The execution loop checks the returned signal and unwinds the call stack accordingly.

### Standard library

`.ae` files in `src/main/resources/stdlib/` are read from the classpath at startup via `StdlibLoader`. The evaluator executes them in the global environment before user code runs.

## Component Status

| Component | Tests | Status |
|-----------|-------|--------|
| Lexer (Scanner) | 16 | Complete |
| Parser | 36 | Complete |
| Evaluator (core) | 47 | Complete |
| Evaluator (extended) | 53 | Complete |
| Stdlib | 41 | Complete |
| Integration | 171 | Complete |
| REPL | manual | Complete |

**Total: 451 tests, 0 failures.**

## Roadmap

### Near term
- GraalVM `native-image` build (zero-JVM startup)
- Homebrew tap / package for distribution
- HTTP stdlib module (`http_get`, `http_post`)

### Medium term
- Bytecode compiler + VM (replace tree-walking for performance)
- Iterator protocol
- Async/await

### Long term
- JIT for hot paths
- Generational garbage collection
- Official package registry

## Technical Decisions

**Why Java 25?**
Pattern matching in `switch` (preview) enables safe, exhaustive dispatch over sealed types — the closest Java equivalent to Rust `enum` + `match`.

**Why tree-walking?**
Fastest path to a correct, maintainable interpreter. Bytecode can be layered later without changing the language.

**Why dual Maven + Gradle?**
Developers can choose their preferred toolchain. Both produce equivalent outputs including the fat JAR.

**Why Jackson for JSON?**
Battle-tested, zero-configuration for basic serialisation, available on Maven Central.

---

**Last Updated**: April 17, 2026
**Implementation**: Java 25 (Maven + Gradle)
