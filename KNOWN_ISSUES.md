# Known Issues

## Critical: Memory Leak in Tests (135+ GB!)

**Severity**: High
**Status**: Identified, not fixed
**Affects**: String stdlib tests primarily

### Symptoms
- Test process consumes 135+ GB of memory
- String tests take 60+ seconds to run
- System becomes unresponsive during test runs

### Root Causes

1. **No Garbage Collection**
   - No GC implementation yet
   - All allocations persist for program lifetime
   - Particularly bad for long-running tests

2. **O(n²) String Operations**
   - String concatenation in loops creates new strings each iteration
   - `result = result + char` allocates new memory every time
   - Example: reversing "hello" creates 5 intermediate strings

3. **split("") Workaround**
   - Creates arrays with empty boundary strings
   - Every string operation using split allocates large arrays
   - Arrays never freed without GC

4. **Test Parallelization**
   - 24 tests run in parallel (15 threads)
   - Each test accumulates memory independently
   - Memory multiplied by thread count

### Workarounds

**Immediate:**
```bash
# Run tests sequentially (slower but less memory)
cargo test -- --test-threads=1

# Run specific test suites
cargo test --test stdlib_core_test    # Small, fast
cargo test --test stdlib_math_test    # Medium
# Skip: stdlib_string_test            # Memory intensive!
```

**In Code:**
```rust
// Mark heavy tests as ignored
#[test]
#[ignore]
fn test_heavy_operation() { ... }

// Run separately:
cargo test -- --ignored
```

### Solutions Planned

#### Phase 4 (Short-term)
- [ ] Add string indexing (eliminate split workaround)
- [ ] Implement StringBuilder for efficient concatenation
- [ ] Profile and optimize hot paths
- [ ] Add memory benchmarks

#### Phase 5 (Long-term)
- [ ] Implement garbage collection (mark-and-sweep or reference counting)
- [ ] Add arena allocators for temporary values
- [ ] Implement copy-on-write strings
- [ ] Add memory profiling tools

### Technical Details

**Memory Growth Pattern:**
```
String reverse("hello"):
- split("")     → ["", "h", "e", "l", "l", "o", ""] (7 allocations)
- Iteration 1:  → "" + "o" = "o" (1 byte)
- Iteration 2:  → "o" + "l" = "ol" (2 bytes)
- Iteration 3:  → "ol" + "l" = "oll" (3 bytes)
- Iteration 4:  → "oll" + "e" = "olle" (4 bytes)
Total: 7 + 1 + 2 + 3 + 4 = 17 allocations for a 5-char string
Complexity: O(n²) memory
```

**Problematic Functions:**
- `reverse()` - O(n²) memory
- `starts_with()` - O(n) arrays × 2
- `ends_with()` - O(n) arrays × 2
- `join()` - O(n) string concatenations
- `repeat()` - O(n) string concatenations

### Mitigation Strategy

**For Contributors:**
1. Avoid string operations in hot loops
2. Use array accumulation then single join
3. Mark heavy tests with `#[ignore]`
4. Profile before optimizing

**For Users:**
1. Run programs with small inputs initially
2. Monitor memory usage with Activity Monitor
3. Use `--test-threads=1` for test runs
4. Consider batch processing for large data

### References
- Issue discovered: March 18, 2026
- Memory usage: 135.28 GB in stdlib_collections_test
- Test runtime: 60+ seconds for 24 string tests
- Related: No GC implementation (Phase 5 planned)

## Other Known Issues

### Function Expressions Not Supported
**Status**: Deferred to Phase 4

Cannot write:
```aether
map([1,2,3], fn(x) { return x * 2 })  // ❌ Syntax error
```

Must use:
```aether
fn double(x) { return x * 2 }
map([1,2,3], double)  // ✅ Works
```

### No Module System
**Status**: Planned for Phase 4

- No `import` statements yet
- All stdlib functions global
- No user-defined modules from filesystem

### No Error Handling
**Status**: Planned for Phase 5

- No `try`/`catch` mechanism
- Errors propagate immediately
- No error recovery

### Limited String Operations
**Status**: Partially addressed

- No string indexing (must use split workaround)
- No `contains()`, `replace()`, `substring()`
- No regex support

### Performance
**Status**: Expected (interpreted)

- Tree-walking interpreter (not bytecode)
- No JIT compilation
- No optimization passes
- Expected for Phase 1-3, will address in Phase 5+

---

**Note**: Despite these issues, Aether successfully implements a complete standard library in itself (bootstrapping), proving the language design is sound. Performance and memory optimizations are typical for Phase 5+ of language development.
