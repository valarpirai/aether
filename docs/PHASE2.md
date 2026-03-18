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

### 1. Fix Critical Bugs ✅
**Status**: COMPLETE - Loops were already working correctly

- [x] **Fix while loop** - Un-ignored test, passes successfully
- [x] **Fix for-in loop** - Un-ignored test, passes successfully
- Note: The "bugs" were actually non-existent; tests were ignored during development

### 2. Built-in Functions ✅
**Status**: COMPLETE - Core functions implemented

#### Core I/O
- [x] `print(...values)` - Print values without newline
- [x] `println(...values)` - Print values with newline
- [ ] `input(prompt)` - Read user input from stdin (deferred)

#### Type Introspection
- [x] `type(value)` - Return type name as string
- [x] `len(collection)` - Return length of array/string

#### Type Conversions
- [x] `int(value)` - Convert to integer
- [x] `float(value)` - Convert to float
- [x] `str(value)` - Convert to string
- [x] `bool(value)` - Convert to boolean

#### Utility Functions
- [ ] `range(start, end)` or `range(end)` - Create integer range (deferred)
- [ ] `exit(code)` - Exit program with status code (deferred)

**Testing**: ✅ 17 new tests (10 unit + 7 integration)

### 3. Member Access Implementation ✅
**Status**: COMPLETE - Following TDD approach

- [x] TDD: Wrote 8 failing tests first (RED phase)
- [x] Implemented eval_member() method (GREEN phase)
- [x] Support for read-only properties (array.length, string.length)
- [x] Error handling for undefined properties
- Parser already had member access AST nodes

**Testing**: ✅ 8 new integration tests using TDD

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

### Sprint 1: Critical Fixes + Core I/O ✅
1. ✅ Un-ignored while loop test - passes
2. ✅ Un-ignored for loop test - passes
3. ✅ Implemented `print()` and `println()`
4. ⏭️ Deferred `input()` to later

**Actual Time**: 1 hour
**Tests**: 2 un-ignored + 6 new = 8 tests

### Sprint 2: Type System Built-ins ✅
1. ✅ Implemented `type()`
2. ✅ Implemented `len()`
3. ✅ Implemented type conversions (`int`, `float`, `str`, `bool`)
4. ⏭️ Deferred `range()` to later

**Actual Time**: 1 hour
**Tests**: 10 unit + 7 integration = 17 tests

### Sprint 3: Member Access ✅ (TDD)
1. ✅ Wrote 8 failing tests (RED phase)
2. ✅ Implemented eval_member() (GREEN phase)
3. ✅ Added error handling for undefined properties
4. ✅ Tested with literals, variables, expressions

**Actual Time**: 30 minutes
**Tests**: 8 integration tests

**Current Status**: 131 tests passing (94 unit + 29 integration + 8 member)

### Sprint 4: Collection Methods 🚧 (Current - TDD)
1. Write failing tests for array methods (RED)
2. Implement array.push(), pop() (GREEN)
3. Write failing tests for string methods (RED)
4. Implement string.upper(), lower(), split(), trim() (GREEN)
5. Refactor if needed

**Estimated Time**: 2-3 hours
**Tests**: ~15-20 new tests
**Approach**: Following TDD red-green-refactor cycle

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
