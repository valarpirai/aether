# Phase 2: Essential Features

**Status**: 🚧 In Progress
**Started**: March 18, 2026

## Overview

Phase 2 builds on the complete Phase 1 interpreter by adding essential built-in functions, fixing known issues, and implementing member access for a more usable language.

### Phase 1 Completion Summary
- ✅ Lexer (14 tests)
- ✅ Parser (53 tests)
- ✅ Interpreter (82 tests, 2 ignored for loop bugs)
- ✅ Integration tests (20 tests)
- ✅ REPL with line editing
- **Total**: 102 tests passing

## Phase 2 Goals

### 1. Fix Critical Bugs ⚠️
**Priority**: HIGH - These prevent basic loops from working

- [ ] **Fix while loop infinite loop bug** (interpreter_tests.rs:711 ignored)
  - Test: `test_while_loop` currently ignored
  - Issue: While loops don't terminate properly
  - Impact: Breaks all while-based iteration

- [ ] **Fix for-in loop infinite loop bug** (interpreter_tests.rs:733 ignored)
  - Test: `test_for_loop` currently ignored
  - Issue: For loops don't terminate properly
  - Impact: Breaks all for-in iteration

### 2. Built-in Functions 🔧
**Priority**: HIGH - Essential for any real program

#### Core I/O
- [ ] `print(...values)` - Print values without newline
- [ ] `println(...values)` - Print values with newline
- [ ] `input(prompt)` - Read user input from stdin

#### Type Introspection
- [ ] `type(value)` - Return type name as string
- [ ] `len(collection)` - Return length of array/string/dict

#### Type Conversions
- [ ] `int(value)` - Convert to integer
- [ ] `float(value)` - Convert to float
- [ ] `str(value)` - Convert to string
- [ ] `bool(value)` - Convert to boolean

#### Utility Functions
- [ ] `range(start, end)` or `range(end)` - Create integer range
- [ ] `exit(code)` - Exit program with status code

**Testing**: Add ~15-20 tests for built-in functions

### 3. Member Access Implementation 🎯
**Priority**: MEDIUM - Needed for collection methods

- [ ] Parser: Add member access expressions (`obj.property`)
- [ ] Interpreter: Implement member access evaluation
- [ ] Support for built-in properties/methods
- [ ] Error handling for missing properties

**Testing**: Add ~8-10 tests for member access

### 4. Collection Methods 📚
**Priority**: MEDIUM - Makes collections actually usable

#### Array Methods
- [ ] `array.push(item)` - Add item to end
- [ ] `array.pop()` - Remove and return last item
- [ ] `array.length` - Property for array length
- [ ] `array.clear()` - Remove all items
- [ ] `array.contains(item)` - Check if item exists

#### String Methods
- [ ] `string.upper()` - Convert to uppercase
- [ ] `string.lower()` - Convert to lowercase
- [ ] `string.length` - Property for string length
- [ ] `string.split(delimiter)` - Split into array
- [ ] `string.trim()` - Remove whitespace

#### Dictionary Methods (Future)
- [ ] `dict.keys()` - Return array of keys
- [ ] `dict.values()` - Return array of values
- [ ] `dict.contains(key)` - Check if key exists
- [ ] `dict.get(key, default)` - Get with default value

**Testing**: Add ~15-20 tests for collection methods

### 5. String Interpolation Evaluation 🔤
**Priority**: LOW - Nice to have but not critical

- [ ] Parse string interpolation (`"Hello ${name}"`)
- [ ] Evaluate expressions inside `${...}`
- [ ] Handle nested interpolations
- [ ] Type conversion in interpolation

**Testing**: Add ~5-8 tests for string interpolation

## Implementation Order

### Sprint 1: Critical Fixes + Core I/O (Current)
1. Fix while loop bug
2. Fix for-in loop bug
3. Implement `print()` and `println()`
4. Implement `input()`
5. Un-ignore loop tests and verify fixes

**Estimated Time**: 1-2 hours
**Tests**: 2 un-ignored + 5-8 new = ~10 tests

### Sprint 2: Type System Built-ins
1. Implement `type()`
2. Implement `len()`
3. Implement type conversion functions (`int`, `float`, `str`, `bool`)
4. Implement `range()`

**Estimated Time**: 1-2 hours
**Tests**: ~10-12 new tests

### Sprint 3: Member Access
1. Add member access to parser AST
2. Implement member access evaluation
3. Add error handling for missing members
4. Test with simple property access

**Estimated Time**: 1 hour
**Tests**: ~8-10 new tests

### Sprint 4: Collection Methods
1. Implement array methods (push, pop, length, etc.)
2. Implement string methods (upper, lower, split, etc.)
3. Add comprehensive collection tests

**Estimated Time**: 2-3 hours
**Tests**: ~15-20 new tests

### Sprint 5: Polish (Optional)
1. String interpolation evaluation
2. Additional utility functions as needed
3. Performance improvements
4. Documentation updates

**Estimated Time**: 1-2 hours
**Tests**: ~5-8 new tests

## Success Criteria

Phase 2 is complete when:

- ✅ All loop bugs fixed (2 ignored tests now passing)
- ✅ All built-in functions working with tests
- ✅ Member access fully implemented
- ✅ Array and string methods working
- ✅ At least 140+ total tests passing (current 102 + ~40 new)
- ✅ All existing tests still passing
- ✅ REPL works with new features
- ✅ Can write real programs with loops, I/O, and collections

## Example Programs After Phase 2

### Fibonacci with I/O
```aether
fn fib(n) {
    if (n <= 1) {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

println("Enter a number:")
let input = input()
let n = int(input)
println("Fibonacci of ${n} is ${fib(n)}")
```

### Array Manipulation
```aether
let numbers = [1, 2, 3, 4, 5]
println("Original: ${numbers}")

numbers.push(6)
println("After push: ${numbers}")

let last = numbers.pop()
println("Popped: ${last}")
println("Final: ${numbers}")
println("Length: ${numbers.length}")
```

### String Processing
```aether
let text = "  Hello World  "
println("Original: '${text}'")
println("Trimmed: '${text.trim()}'")
println("Upper: '${text.upper()}'")
println("Words: ${text.trim().split(" ")}")
```

## Documentation Updates Needed

After Phase 2:
- [ ] Update CLAUDE.md with Phase 2 status
- [ ] Update ARCHITECTURE.md with new features
- [ ] Update REPL.md with new built-in functions
- [ ] Create BUILTINS.md documenting all built-in functions
- [ ] Update README.md with examples using new features

## Future Work (Phase 3+)

Deferred to later phases:
- Module system (import/from)
- Error handling (try/catch)
- Set operations
- Dictionary methods
- Advanced string formatting
- File I/O
- Standard library organization
