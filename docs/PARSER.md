# Aether Parser Documentation

## Overview

Recursive descent parser that converts tokens into an Abstract Syntax Tree (AST).

**Location**: `src/parser/`
**Status**: ✅ Complete (53 tests passing)

## Components

- `ast.rs` - AST node definitions
- `parse.rs` - Recursive descent parser
- `parser_tests.rs` - 53 tests

## AST Nodes

### Expressions (Expr)
- `Integer(i64)`, `Float(f64)`, `String(String)`, `Bool(bool)`, `Null`
- `Identifier(String)`
- `Binary(Box<Expr>, BinaryOp, Box<Expr>)`
- `Unary(UnaryOp, Box<Expr>)`
- `Call(Box<Expr>, Vec<Expr>)` - Function calls
- `Array(Vec<Expr>)` - Array literals
- `Index(Box<Expr>, Box<Expr>)` - Array indexing
- `Member(Box<Expr>, String)` - Member access

### Statements (Stmt)
- `Expr(Expr)` - Expression statement
- `Let(String, Expr)` - Variable declaration
- `Assign(Expr, Expr)` - Assignment
- `CompoundAssign(Expr, BinaryOp, Expr)` - `+=`, `-=`, etc.
- `Block(Vec<Stmt>)` - Block of statements
- `If(Expr, Box<Stmt>, Option<Box<Stmt>>)` - If/else
- `While(Expr, Box<Stmt>)` - While loop
- `For(String, Expr, Box<Stmt>)` - For-in loop
- `Return(Option<Expr>)` - Return statement
- `Break`, `Continue` - Loop control
- `Function(String, Vec<String>, Box<Stmt>)` - Function declaration

## Operator Precedence (Lowest to Highest)

1. Logical OR (`||`)
2. Logical AND (`&&`)
3. Equality (`==`, `!=`)
4. Comparison (`<`, `>`, `<=`, `>=`)
5. Addition/Subtraction (`+`, `-`)
6. Multiplication/Division/Modulo (`*`, `/`, `%`)
7. Unary (`-`, `!`)
8. Postfix (calls, indexing, member access)
9. Primary (literals, identifiers, grouping)

## Parser Methods

```rust
parse() → declaration() → statement() → expression()
                                          ↓
                                    logical_or()
                                          ↓
                                    logical_and()
                                          ↓
                                      equality()
                                          ↓
                                     comparison()
                                          ↓
                                      addition()
                                          ↓
                                   multiplication()
                                          ↓
                                       unary()
                                          ↓
                                        call()
                                          ↓
                                      primary()
```

## Examples

**Input**: `1 + 2 * 3`
**AST**:
```
Binary(
  Integer(1),
  Add,
  Binary(Integer(2), Multiply, Integer(3))
)
```

**Input**: `arr[0]`
**AST**: `Index(Identifier("arr"), Integer(0))`

**Input**: `fn add(a, b) { return a + b }`
**AST**: `Function("add", ["a", "b"], Block([Return(Binary(...))]))`

## Usage

```rust
let tokens = scanner.scan_tokens()?;
let mut parser = Parser::new(tokens);
let program = parser.parse()?;
```
