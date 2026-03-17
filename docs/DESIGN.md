# Aether Programming Language Design Document

## Overview

Aether is a general-purpose, dynamically typed programming language with automatic memory management. It combines the familiarity of C-like syntax with the ease of use of modern interpreted languages.

## Core Philosophy

- **General-purpose**: Suitable for a wide range of programming tasks
- **Dynamic typing**: Types are checked at runtime for flexibility
- **Automatic memory management**: Garbage collection handles memory allocation/deallocation
- **Interpreted**: Direct execution without compilation step for rapid development
- **Clean syntax**: C-like blocks without semicolons

## Language Specifications

### 1. Execution Model

- **Type**: Interpreted language
- **File extension**: `.ae`
- **Entry point**: Required `main()` function
- **REPL**: Interactive mode supported for testing and exploration

### 2. Syntax

#### 2.1 General Structure
- **Block delimiters**: Curly braces `{ }`
- **Statement termination**: No semicolons required
- **Comments**:
  - Single-line: `// comment`
  - Multi-line: `/* comment */`

#### 2.2 Program Structure
```aether
// Every program requires a main function
fn main() {
    // Program logic here
}
```

### 3. Type System

#### 3.1 Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `int` | Integer numbers | `42`, `-17`, `0` |
| `float` | Floating-point numbers | `3.14`, `-0.5`, `2.0` |
| `string` | UTF-8 immutable text | `"hello"`, `"世界"` |
| `bool` | Boolean values | `true`, `false` |
| `null` | Absence of value | `null` |

#### 3.2 Collection Types

| Type | Description | Example |
|------|-------------|---------|
| `array` | Ordered, mutable collection | `[1, 2, 3]` |
| `dict` | Key-value pairs | `{"name": "Alice", "age": 30}` |
| `set` | Unique, unordered collection | `{1, 2, 3}` |

#### 3.3 Type Characteristics
- **Dynamic typing**: Variables can hold values of any type
- **Type checking**: Performed at runtime
- **Type conversion**: Built-in functions for explicit conversion

### 4. Variables and Scoping

#### 4.1 Variable Declaration
```aether
let x = 10        // Variable declaration
let name = "Aether"
let items = [1, 2, 3]
```

#### 4.2 Scoping Rules
- **Block scope**: Variables are scoped to their enclosing block `{}`
- **Function scope**: Function parameters and local variables
- **Global scope**: Variables declared outside functions

```aether
let global_var = 100  // Global scope

fn example() {
    let x = 10  // Function scope

    if (x > 5) {
        let y = 20  // Block scope - only visible within if block
    }
    // y is not accessible here
}
```

### 5. Functions

#### 5.1 Function Declaration
```aether
fn function_name(param1, param2) {
    // function body
    return value
}
```

#### 5.2 Function Features
- **First-class functions**: Functions can be assigned to variables, passed as arguments, and returned
- **Closures**: Functions can capture variables from enclosing scope
- **Anonymous functions/lambdas**: Supported (syntax TBD in detailed design)

### 6. Operators

#### 6.1 Arithmetic Operators
| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `a + b` |
| `-` | Subtraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Modulo | `a % b` |

#### 6.2 Comparison Operators
| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal to | `a == b` |
| `!=` | Not equal to | `a != b` |
| `<` | Less than | `a < b` |
| `>` | Greater than | `a > b` |
| `<=` | Less than or equal | `a <= b` |
| `>=` | Greater than or equal | `a >= b` |

#### 6.3 Logical Operators
| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `a && b` |
| `\|\|` | Logical OR | `a \|\| b` |
| `!` | Logical NOT | `!a` |

#### 6.4 Assignment Operators
| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Assignment | `a = 10` |
| `+=` | Add and assign | `a += 5` |
| `-=` | Subtract and assign | `a -= 3` |
| `*=` | Multiply and assign | `a *= 2` |
| `/=` | Divide and assign | `a /= 4` |

### 7. Control Flow

#### 7.1 Conditional Statements
```aether
if (condition) {
    // code
} else if (another_condition) {
    // code
} else {
    // code
}
```

#### 7.2 Loops

**While Loop:**
```aether
while (condition) {
    // code
    if (some_condition) {
        break    // Exit loop
    }
    if (other_condition) {
        continue // Skip to next iteration
    }
}
```

**For Loop (Range-based):**
```aether
for i in range(0, 10) {
    // i goes from 0 to 9
}
```

**For-Each Loop:**
```aether
for item in array {
    // iterate over each item
}

for key, value in dict {
    // iterate over dictionary entries
}
```

### 8. Strings

#### 8.1 String Features
- **Immutable**: Strings cannot be modified after creation
- **UTF-8**: Full Unicode support
- **Interpolation**: Using `${}` syntax

```aether
let name = "World"
let greeting = "Hello ${name}!"  // "Hello World!"
```

#### 8.2 Escape Sequences
| Sequence | Description |
|----------|-------------|
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Backslash |
| `\"` | Double quote |

### 9. Built-in Functions

#### 9.1 Global Functions
| Function | Description | Example |
|----------|-------------|---------|
| `print(value)` | Output without newline | `print("Hello")` |
| `println(value)` | Output with newline | `println("Hello")` |
| `input()` | Read user input | `let name = input()` |
| `type(value)` | Get type of value | `type(42) // "int"` |
| `len(collection)` | Get length | `len([1,2,3]) // 3` |
| `int(value)` | Convert to integer | `int("42") // 42` |
| `float(value)` | Convert to float | `float("3.14") // 3.14` |
| `str(value)` | Convert to string | `str(42) // "42"` |
| `bool(value)` | Convert to boolean | `bool(1) // true` |
| `range(start, end)` | Generate range | `range(0, 5) // 0,1,2,3,4` |

#### 9.2 Methods

**Array Methods:**
- `array.push(item)` - Add item to end
- `array.pop()` - Remove and return last item
- `array.length` - Get array length

**Dictionary Methods:**
- `dict.keys()` - Get all keys
- `dict.values()` - Get all values
- `dict.get(key)` - Get value for key

**String Methods:**
- `string.length` - Get string length
- `string.upper()` - Convert to uppercase
- `string.lower()` - Convert to lowercase

### 10. Module System

#### 10.1 Import Statements
```aether
import module_name
import module_name as alias
from module_name import function_name
```

### 11. Future Considerations

Features to be added in future versions:
- **Objects/Classes**: Object-oriented programming support
- **Error Handling**: Exception mechanism or Result types
- **Pattern Matching**: Match expressions
- **Async/Await**: Asynchronous programming
- **Package Manager**: Dependency management
- **Standard Library**: Extended built-in functionality

## Implementation Phases

### Phase 1: Core Interpreter
1. Lexer (tokenization)
2. Parser (AST generation)
3. Basic interpreter (tree-walking)
4. REPL

### Phase 2: Basic Features
1. All primitive types
2. Variables and scoping
3. Functions
4. Basic control flow

### Phase 3: Collections & Built-ins
1. Arrays, dictionaries, sets
2. Built-in functions
3. String interpolation
4. For-each loops

### Phase 4: Module System
1. Import mechanism
2. Module resolution
3. Standard library modules

## Example Program

```aether
// Fibonacci sequence example
fn fibonacci(n) {
    if (n <= 1) {
        return n
    }

    let a = 0
    let b = 1

    for i in range(2, n + 1) {
        let temp = a + b
        a = b
        b = temp
    }

    return b
}

fn main() {
    println("Fibonacci Sequence")

    for i in range(0, 10) {
        let result = fibonacci(i)
        println("fib(${i}) = ${result}")
    }
}
```

## Design Decisions Summary

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Typing | Dynamic | Flexibility and ease of use |
| Memory | Garbage collected | Automatic management reduces errors |
| Syntax | C-like with no semicolons | Familiar yet clean |
| Execution | Interpreted | Fast development cycle |
| Scoping | Block-scoped | Prevents variable leakage bugs |
| Strings | Immutable | Thread-safety and predictability |
| Entry point | Required main() | Clear program structure |
| Numbers | Separate int/float | Better precision control |
| Loops | Range-based and for-each | Modern and readable |
| Functions | Rust-like with fn keyword | Clear function declarations |
