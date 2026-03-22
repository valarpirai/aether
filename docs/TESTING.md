# Aether Testing Guide

This document provides comprehensive guidance on testing the Aether interpreter.

## Table of Contents
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Test-Driven Development](#test-driven-development)
- [Writing Tests](#writing-tests)
- [Debugging Test Failures](#debugging-test-failures)
- [Test Coverage Goals](#test-coverage-goals)

## Test Organization

### Directory Structure

```
tests/
├── integration_tests.rs     # End-to-end program tests
├── member_access_tests.rs   # Member access feature tests
├── array_tests.rs           # Array method tests
├── string_tests.rs          # String method tests
└── stdlib_tests.rs          # Standard library tests

src/
├── lexer/
│   └── lexer_tests.rs       # Lexer unit tests
├── parser/
│   └── parser_tests.rs      # Parser unit tests
└── interpreter/
    └── interpreter_tests.rs # Interpreter unit tests
```

### Test Categories

**Unit Tests** (94 tests):
- Test individual components in isolation
- Located in module test files (`*_tests.rs`)
- Fast execution (< 1 second)
- No dependencies between tests

**Integration Tests** (136 tests):
- Test complete programs end-to-end
- Located in `tests/` directory
- Test feature interaction
- Slower execution (few seconds)

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test lexer
cargo test parser
cargo test interpreter

# Show println! output during tests
cargo test -- --nocapture
```

### Recommended Options

**IMPORTANT**: Always use these flags to prevent memory issues:

```bash
# Sequential execution (prevents memory pressure)
cargo test -- --test-threads=1

# With output
cargo test -- --nocapture --test-threads=1

# Single test with output
cargo test test_name -- --nocapture --test-threads=1
```

**Why `--test-threads=1`?**
- Reduces memory pressure (parallel tests can use 135 GB+)
- More predictable execution
- Easier debugging

### Test Filtering

```bash
# Run only integration tests
cargo test --test integration_tests

# Run only unit tests
cargo test --lib

# Run tests matching pattern
cargo test string  # Runs all tests with "string" in name

# Run ignored tests
cargo test -- --ignored
```

### Continuous Testing

```bash
# Watch for changes and re-run tests
cargo watch -x test

# Watch with clear screen
cargo watch -c -x "test -- --test-threads=1"
```

## Test-Driven Development

### The Red-Green-Refactor Cycle

**1. Red**: Write a failing test
```rust
#[test]
fn test_exponentiation() {
    let result = eval("2 ** 3");
    assert_eq!(result, Value::Int(8));  // FAILS - feature not implemented
}
```

**2. Green**: Write minimal code to make it pass
```rust
// Add just enough code to pass the test
// Don't over-engineer!
```

**3. Refactor**: Improve code while keeping tests green
```rust
// Clean up implementation
// Tests should still pass
```

### TDD Workflow Example

**Goal**: Add string `repeat()` method

**Step 1**: Write the test first
```rust
#[test]
fn test_string_repeat() {
    let result = eval("\"ha\".repeat(3)");
    assert_eq!(result, Value::string("hahaha".to_string()));
}
```

**Step 2**: Run test (it should fail)
```bash
cargo test test_string_repeat
# Expected failure: repeat not implemented
```

**Step 3**: Implement minimum code to pass
```aether
// In stdlib/string.ae
fn repeat(text, n) {
    let result = ""
    let i = 0
    while (i < n) {
        result = result + text
        i = i + 1
    }
    return result
}
```

**Step 4**: Run test again (should pass)
```bash
cargo test test_string_repeat -- --test-threads=1
```

**Step 5**: Add edge case tests
```rust
#[test]
fn test_string_repeat_zero() {
    assert_eq!(eval("\"x\".repeat(0)"), Value::string("".to_string()));
}

#[test]
fn test_string_repeat_negative() {
    assert!(eval("\"x\".repeat(-1)").is_err());
}
```

**Step 6**: Refactor if needed

### Benefits of TDD

✅ **Clear requirements** - Test defines what "done" means
✅ **Confidence** - Refactoring doesn't break functionality
✅ **Documentation** - Tests show how to use features
✅ **Better design** - Testable code is usually better code
✅ **Regression prevention** - Old tests catch new bugs

## Writing Tests

### Unit Test Structure

```rust
#[test]
fn test_<component>_<feature>() {
    // Arrange: Set up test data
    let input = "...";

    // Act: Execute the operation
    let result = operation(input);

    // Assert: Verify the result
    assert_eq!(result, expected);
}
```

### Integration Test Structure

```rust
#[test]
fn test_<feature_name>() {
    let source = r#"
        fn main() {
            // Aether code here
        }
    "#;

    let result = run_program(source);
    assert_eq!(result, expected_output);
}
```

### Assertion Helpers

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// Boolean
assert!(condition);
assert!(!condition);

// Error handling
assert!(result.is_ok());
assert!(result.is_err());

// Pattern matching
match result {
    Ok(Value::Int(n)) => assert_eq!(n, 42),
    _ => panic!("Expected Int"),
}
```

### Test Naming Conventions

**Good test names**:
- `test_addition_integers`
- `test_division_by_zero_error`
- `test_array_push_increases_length`
- `test_string_upper_ascii`

**Poor test names**:
- `test1`, `test2`
- `test_function`
- `test_it_works`

**Pattern**: `test_<what>_<scenario>_<expected_result>`

## Debugging Test Failures

### Step 1: Read the Error Message

```
---- test_division_by_zero panicked at 'assertion failed: `(left == right)`
  left: `Ok(Int(5))`,
 right: `Err(DivisionByZero)`', tests/integration_tests.rs:42:5
```

**Key information**:
- Test name: `test_division_by_zero`
- Expected: `Err(DivisionByZero)`
- Actual: `Ok(Int(5))`
- Location: `tests/integration_tests.rs:42:5`

### Step 2: Isolate the Test

```bash
# Run only the failing test
cargo test test_division_by_zero -- --nocapture --test-threads=1
```

### Step 3: Add Debug Output

```rust
#[test]
fn test_division_by_zero() {
    let source = "10 / 0";
    println!("Input: {}", source);

    let result = eval(source);
    println!("Result: {:?}", result);

    assert!(result.is_err());
}
```

### Step 4: Use Rust Debugger

```bash
# Install rust-lldb or rust-gdb
rust-lldb target/debug/deps/aether-<hash>

# Set breakpoint
(lldb) breakpoint set --name test_division_by_zero
(lldb) run

# Step through
(lldb) step
(lldb) print variable_name
```

### Common Test Failures

**Memory Issues**
```
error: test failed, to rerun pass '--lib'
signal: 9, SIGKILL: kill
```
**Solution**: Use `--test-threads=1`

**Timeout**
```
test hangs indefinitely
```
**Solution**:
- Check for infinite loops
- Add timeout to test command
- Use `timeout 60 cargo test`

**Floating Point Precision**
```rust
// Wrong: Exact equality
assert_eq!(result, 3.14159);

// Right: Approximate equality
assert!((result - 3.14159).abs() < 0.00001);
```

**String/Array Comparison**
```rust
// For Rc-wrapped values, use pattern matching or helper methods
match &value {
    Value::String(s) => assert_eq!(s.as_ref(), "expected"),
    _ => panic!("Expected string"),
}
```

## Test Coverage Goals

### Current Coverage (Phase 3)

- **Total**: 230 tests ✅
- **Success Rate**: 100%
- **Lines Covered**: ~85% (estimated)

### Coverage by Component

| Component | Unit Tests | Integration Tests | Status |
|-----------|------------|-------------------|--------|
| Lexer | 14 | - | ✅ Complete |
| Parser | 53 | - | ✅ Complete |
| Interpreter | 17 | 29 | ✅ Complete |
| Built-ins | 10 | - | ✅ Complete |
| Member Access | - | 8 | ✅ Complete |
| Array Methods | - | 8 | ✅ Complete |
| String Methods | - | 8 | ✅ Complete |
| Stdlib Core | - | 9 | ✅ Complete |
| Stdlib Collections | - | 24 | ✅ Complete |
| Stdlib Math | - | 26 | ✅ Complete |
| Stdlib String | - | 24 | ✅ Complete |

### What to Test

**Always test**:
- ✅ Happy path (valid inputs)
- ✅ Edge cases (empty, zero, null)
- ✅ Error conditions (invalid inputs)
- ✅ Boundary values (min, max)
- ✅ Type mismatches

**Example - Testing `array.push()`**:
```rust
// Happy path
test_array_push_adds_element()

// Edge cases
test_array_push_to_empty_array()
test_array_push_multiple_types()

// Integration
test_array_push_in_loop()
test_array_push_with_function_result()
```

### Measuring Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage report
cargo tarpaulin --out Html

# Open report
open tarpaulin-report.html
```

**Coverage Goals**:
- Core components: 90%+ line coverage
- Error handling paths: 80%+ coverage
- Overall: 85%+ coverage

## Best Practices

### Do's ✅

- **Write tests first** (TDD)
- **Test one thing** per test
- **Use descriptive names**
- **Test edge cases**
- **Keep tests independent**
- **Use `--test-threads=1`** for Aether tests
- **Commit tests with code**

### Don'ts ❌

- **Don't test implementation details**
- **Don't share state between tests**
- **Don't skip error cases**
- **Don't use random values** (unless testing random behavior)
- **Don't make tests depend on each other**
- **Don't ignore failing tests** (fix or remove)

### Test Smells

**Problem**: Tests are slow (> 10 seconds)
- **Solution**: Mock external dependencies, reduce test scope

**Problem**: Tests are flaky (pass/fail randomly)
- **Solution**: Check for race conditions, shared state, random values

**Problem**: Tests break on every change
- **Solution**: Test behavior, not implementation details

**Problem**: Can't understand what test does
- **Solution**: Better names, comments, simpler test structure

## Continuous Integration

### Future CI Setup

When setting up CI, include:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test -- --test-threads=1
      - name: Check formatting
        run: cargo fmt --check
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

## Resources

### Internal Documentation
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Development workflow
- **[CLAUDE.md](../CLAUDE.md)** - Project overview
- Component docs: LEXER.md, PARSER.md, INTERPRETER.md

### External Resources
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [TDD in Rust](https://www.youtube.com/watch?v=2vBQFIWl36k)

---

**Last Updated**: March 22, 2026
**Phase**: 3 Complete
**Status**: 230 tests passing, comprehensive test coverage
