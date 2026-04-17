# Aether Testing Guide

How to run, write, and debug tests for the Aether Java interpreter.

---

## Test Layout

```
src/test/java/com/aether/
├── lexer/ScannerTest.java         # 16 tests — tokenisation
├── parser/ParserTest.java         # 36 tests — AST structure
└── interpreter/EvaluatorTest.java # 47 tests — end-to-end execution
```

**Total: 99 tests, 0 failures.**

All three suites are integration-style: they parse and execute Aether source strings directly, testing the full pipeline rather than mocked units.

---

## Running Tests

```bash
# All tests (Maven)
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn test

# Single class
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn test -Dtest=EvaluatorTest

# Single method
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn test -Dtest=EvaluatorTest#closures

# Show stdout during tests
JAVA_HOME=/opt/homebrew/opt/openjdk@25 mvn test -Dtest=EvaluatorTest -Dsurefire.useFile=false
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

`EvaluatorTest` uses `Evaluator.withoutStdlib()` so each test starts with a clean, fast environment:

```java
@BeforeEach
void setUp() {
  evaluator = Evaluator.withoutStdlib();
}
```

Use `Evaluator.withStdlib()` only when a test needs stdlib functions like `map()` or `range()`.

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

## Test Coverage

| Category | Tests |
|----------|-------|
| Arithmetic & operators | intArithmetic, floatArithmetic, mixedIntFloat, stringConcatenation |
| Booleans & truthiness | booleanLogic, comparison, truthinessRules |
| Variables | letAndLookup, assignment, compoundAssignment, undefinedVariableThrows |
| Control flow | ifTrue, ifElse, whileLoop, forLoop, breakInWhile |
| Functions | functionDeclarationAndCall, optionalParams, closures, recursion, stackOverflowThrows |
| Strings | stringIndexing, stringMethods, stringSplit, stringInterpolation, stringLength |
| Arrays | arrayLiteral, arrayIndex, arrayLength, arrayPushPop, arraySlice, spreadOperator |
| Dicts | dictLiteralAndAccess, dictKeys, dictContains |
| Error handling | tryCatch, tryCatchRuntimeError |
| Structs | structDeclarationAndInstantiation, structMethod, structFieldMutation |
| Builtins | typeBuiltin, lenBuiltin, conversionBuiltins, clockReturnsPositiveFloat |
| Runtime errors | typeError, indexOutOfBounds, divisionByZeroThrows |

---

**Last Updated**: April 17, 2026
**Implementation**: Java 25, JUnit 5
