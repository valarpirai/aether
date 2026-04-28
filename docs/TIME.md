# Time Functions in Aether

**Status**: ✅ Complete  
**Added**: Phase 5  
**Tests**: 10 tests passing

## Overview

Aether provides simple time utilities through `clock()` and `sleep()` built-in functions.

## Functions

### clock()

Returns the current Unix timestamp as a float (seconds since epoch):

```aether
let now = clock()
println(now)  // 1714252800.123456
```

**Precision**: Microsecond precision (6 decimal places)

### sleep(seconds)

Pause execution for the specified duration:

```aether
sleep(2)  // Sleep for 2 seconds
```

**Arguments**:
- `seconds` - Can be int or float
- Fractional seconds supported: `sleep(0.5)` = 500ms

## Examples

### Example 1: Timing Operations

```aether
fn time_operation(func) {
    let start = clock()
    func()
    let end = clock()
    let elapsed = end - start
    println("Operation took ${elapsed} seconds")
}

fn main() {
    time_operation(fn() {
        let sum = 0
        for i in range(0, 1000000) {
            sum = sum + i
        }
    })
}
```

### Example 2: Rate Limiting

```aether
fn rate_limited_task() {
    let delay = 1.0  // 1 second between tasks
    
    for i in range(0, 5) {
        println("Task ${i} at ${clock()}")
        process_item(i)
        sleep(delay)
    }
}

fn process_item(i) {
    println("  Processing item ${i}")
}

fn main() {
    rate_limited_task()
}
```

### Example 3: Simple Benchmark

```aether
fn benchmark(name, iterations, func) {
    let start = clock()
    for i in range(0, iterations) {
        func()
    }
    let end = clock()
    let total = end - start
    let per_iter = total / iterations
    println("${name}: ${total}s total, ${per_iter}s per iteration")
}

fn main() {
    benchmark("String concat", 10000, fn() {
        let s = "hello" + "world"
    })
}
```

### Example 4: Retry with Backoff

```aether
fn retry_with_backoff(func, max_attempts) {
    let attempt = 0
    let delay = 0.5
    
    for attempt in range(0, max_attempts) {
        try {
            return func()
        } catch(e) {
            println("Attempt ${attempt + 1} failed: ${e}")
            if (attempt < max_attempts - 1) {
                println("Retrying in ${delay} seconds...")
                sleep(delay)
                delay = delay * 2  // Exponential backoff
            }
        }
    }
    
    throw "All ${max_attempts} attempts failed"
}

fn main() {
    try {
        retry_with_backoff(fn() {
            // Simulate flaky operation
            let rand = clock()
            if (rand % 3 != 0) {
                throw "Random failure"
            }
            return "Success!"
        }, 5)
    } catch(e) {
        println("Final error:", e)
    }
}
```

### Example 5: Timeout Simulation

```aether
fn with_timeout(func, timeout_sec) {
    let start = clock()
    
    let result = func()
    
    let elapsed = clock() - start
    if (elapsed > timeout_sec) {
        throw "Operation exceeded ${timeout_sec}s timeout"
    }
    
    return result
}

fn main() {
    try {
        with_timeout(fn() {
            println("Starting slow operation")
            sleep(2)
            return "Done"
        }, 3)
        println("Completed within timeout")
    } catch(e) {
        println("Timeout error:", e)
    }
}
```

## Best Practices

### 1. Use Float for Precise Timing

```aether
let start = clock()
// ... operation ...
let duration = clock() - start
println("Took ${duration} seconds")
```

### 2. Don't Sleep in Tight Loops

```aether
// Bad - inefficient
for i in range(0, 1000) {
    sleep(0.001)  // 1ms each iteration
}

// Better - sleep once
sleep(1.0)  // 1 second total
```

### 3. Handle Sleep Interruption

While Aether's `sleep()` isn't interruptible, design code to be cancellable:

```aether
fn cancellable_wait(seconds, should_cancel) {
    let start = clock()
    while (clock() - start < seconds) {
        if (should_cancel()) {
            return false
        }
        sleep(0.1)  // Small intervals
    }
    return true
}
```

### 4. Use for Profiling, Not Production Timing

`clock()` is good for benchmarks and profiling, but don't rely on it for precise real-time control.

## Implementation Notes

- `clock()` uses `SystemTime::now()` (Unix epoch)
- `sleep()` uses `std::thread::sleep()`
- Minimum sleep duration depends on OS scheduler
- Sleep of 0 or negative is no-op

## Limitations

- No date/time formatting or parsing
- No timezone support
- No high-precision timers (nanosecond)
- Sleep is not interruptible
- No async/non-blocking sleep

## See Also

- [DESIGN.md](DESIGN.md) - Language specification
- [examples/](../examples/) - Usage examples

---

**Last Updated**: 2026-04-28  
**Status**: Complete and stable
