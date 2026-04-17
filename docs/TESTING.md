# Aether Testing Guide

How to run, write, and debug tests for the Aether Java interpreter.

---

## Test Layout

```
src/test/java/com/aether/
├── lexer/
│   └── ScannerTest.java           # 16 tests — tokenisation
├── parser/
│   └── ParserTest.java            # 36 tests — AST structure
└── interpreter/
    ├── EvaluatorTest.java         # 47 tests — core evaluator
    ├── MoreEvaluatorTest.java     # 53 tests — JSON, modules, closures, dicts
    ├── StdlibTest.java            # 41 tests — stdlib functions (needs withStdlib)
    ├── IntegrationTest.java       # 24 tests — end-to-end programs
    ├── FunctionExprTest.java      # 13 tests — function expressions, closures
    ├── StringInterpTest.java      # 14 tests — string interpolation
    ├── StringIndexingTest.java    # 15 tests — string character indexing
    ├── StringMethodsTest.java     # 16 tests — upper/lower/trim/split
    ├── SliceAndSpreadTest.java    # 32 tests — array/string slices, spread
    ├── ArrayMethodsTest.java      # 15 tests — push/pop/sort/concat/contains
    ├── DictTest.java              # 17 tests — dict literals and methods
    ├── StructTest.java            # 14 tests — struct declaration and methods
    ├── MemberAccessTest.java      #  8 tests — .length and property access
    ├── JsonTest.java              # 25 tests — json_parse / json_stringify
    ├── ModuleTest.java            # 13 tests — import / from...import / aliases
    ├── ErrorHandlingTest.java     # 10 tests — try/catch/throw
    ├── IoAndTimeTest.java         # 13 tests — read_file/write_file/clock/sleep
    └── TestingFrameworkTest.java  # 19 tests — stdlib testing module
```

**Total: 451 tests, 0 failures.**

All suites are integration-style: they parse and execute Aether source strings directly, testing the full pipeline rather than mocked units.

---

## Running Tests

```bash
# Requires Java 25
export JAVA_HOME=/opt/homebrew/opt/openjdk@25

# All tests
mvn test

# Single class
mvn test -Dtest=EvaluatorTest

# Single method
mvn test -Dtest=EvaluatorTest#closures

# Show stdout during tests
mvn test -Dtest=EvaluatorTest -Dsurefire.useFile=false
```

---

## Writing Tests

### Evaluator tests

Use the `eval()` helper — it parses a source string, executes all statements except the last, then returns the display value of the last expression:

```java
@Test
void myFeature() {
  assertEquals("42", eval("let x = 40\nx + 2"));
}
```

For statements that produce no value (control flow, declarations), end with a variable reference:

```java
assertEquals("10", eval("let x = 5\nx = 10\nx"));
```

For expected runtime errors:

```java
@Test
void divisionByZero() {
  assertThrows(AetherRuntimeException.DivisionByZero.class, () -> eval("1 / 0"));
}
```

### Parser tests

```java
private List<Stmt> parse(String source) {
  return new Parser(new Scanner(source).scanTokens()).parse();
}

private Expr parseExpr(String source) {
  return ((Stmt.ExprStmt) parse(source).get(0)).expr();
}

@Test
void integerLiteral() {
  assertEquals(42L, ((Expr.IntLiteral) parseExpr("42")).value());
}
```

### Scanner tests

```java
private List<Token> scan(String source) {
  return new Scanner(source).scanTokens();
}

@Test
void intToken() {
  Token t = scan("42").get(0);
  assertEquals(TokenKind.INT, t.kind());
  assertEquals(42L, t.intValue());
}
```

---

## Evaluator test setup

Most tests use `Evaluator.withoutStdlib()` so each test starts with a clean, fast environment:

```java
@BeforeEach
void setUp() {
  evaluator = Evaluator.withoutStdlib();
}
```

Use `Evaluator.withStdlib()` only when a test needs stdlib functions like `map()`, `range()`, or `assert_eq()`:

```java
@BeforeEach
void setUp() {
  evaluator = Evaluator.withStdlib();
}
```

---

## What to Test

For each new feature, cover:

1. **Happy path** — expected output for valid input
2. **Edge cases** — empty collections, zero, null, negative numbers
3. **Error cases** — `assertThrows` for type errors, bounds errors, etc.
4. **Interaction** — feature combined with existing features (closures + recursion, etc.)

---

## Debugging Failures

**Print the actual value:**
```java
System.out.println(eval("your expression here"));
```

**Run one test in isolation:**
```bash
mvn test -Dtest=EvaluatorTest#myTest -Dsurefire.useFile=false
```

**Common causes of unexpected `null`:**
- Function body missing `return` — Aether requires explicit `return`
- `if/else` used as an expression — it is a statement; assign to a variable instead

**Common parse failures:**
- Dict `{}` at statement level — wrap in `let d = {}` for expression context
- Missing commas or braces in struct/dict definitions

---

## Test Coverage Summary

| Test Class | Count | Coverage |
|---|---|---|
| `ScannerTest` | 16 | Tokenisation: keywords, literals, operators, strings |
| `ParserTest` | 36 | AST: expressions, statements, control flow, structs |
| `EvaluatorTest` | 47 | Core: arithmetic, variables, loops, functions, builtins |
| `MoreEvaluatorTest` | 53 | JSON, modules, closures, dicts, error handling, types |
| `StdlibTest` | 41 | range, enumerate, map, filter, reduce, sort, abs, join, reverse |
| `IntegrationTest` | 24 | Complete programs: fibonacci, builtins, operators |
| `FunctionExprTest` | 13 | fn expressions, closures, higher-order functions |
| `StringInterpTest` | 14 | `"${expr}"` interpolation in various contexts |
| `StringIndexingTest` | 15 | `s[i]` indexing, negative, out-of-bounds, expressions |
| `StringMethodsTest` | 16 | upper, lower, trim, split, length, chaining |
| `SliceAndSpreadTest` | 32 | `arr[a:b]`, `str[a:b]`, `[...arr]` |
| `ArrayMethodsTest` | 15 | push, pop, sort, sort(fn), concat, contains, length |
| `DictTest` | 17 | literals, keys/values/contains, assignment, iteration |
| `StructTest` | 14 | fields, methods, nesting, mutation, error cases |
| `MemberAccessTest` | 8 | `.length` on arrays/strings, error cases |
| `JsonTest` | 25 | json_parse/json_stringify: all types, round-trip |
| `ModuleTest` | 13 | import, from...import, aliases, error cases |
| `ErrorHandlingTest` | 10 | throw, catch, rethrow, nested, cross-call propagation |
| `IoAndTimeTest` | 13 | read_file, write_file, clock, sleep |
| `TestingFrameworkTest` | 19 | assert_eq, assert_true, expect_error, test(), test_summary |
| **Total** | **451** | |

---

**Last Updated**: April 17, 2026
**Implementation**: Java 25, JUnit 5
