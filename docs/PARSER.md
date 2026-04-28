---
layout: default
title: Aether Parser Documentation
---

# Aether Parser Documentation

## Overview

Recursive descent parser that converts tokens into an Abstract Syntax Tree (AST).

**Location**: `src/parser/`
**Status**: ✅ Complete (53 unit tests passing)

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

## Common Tasks

### How to Add a New Binary Operator

**Example**: Adding the `**` (exponentiation) operator

**Step 1**: Add operator to `ast.rs`
```rust
pub enum BinaryOp {
    // ... existing operators
    Exponent,  // **
}
```

**Step 2**: Determine precedence level in `parse.rs`
- Higher precedence than `*` and `/`
- Create new precedence method or add to existing

```rust
fn exponentiation(&mut self) -> Result<Expr, ParseError> {
    let mut expr = self.unary()?;

    while self.match_token(&[TokenKind::StarStar]) {
        let op = BinaryOp::Exponent;
        let right = self.unary()?;  // Right associative
        expr = Expr::Binary(Box::new(expr), op, Box::new(right));
    }

    Ok(expr)
}
```

**Step 3**: Update precedence chain
```rust
fn multiplication(&mut self) -> Result<Expr, ParseError> {
    let mut expr = self.exponentiation()?;  // Call higher precedence
    // ... rest of multiplication logic
}
```

**Step 4**: Write tests
```rust
#[test]
fn test_exponentiation() {
    let ast = parse("2 ** 3");
    // Verify AST structure
}

#[test]
fn test_exponentiation_precedence() {
    let ast = parse("2 + 3 ** 4");  // Should be 2 + (3 ** 4)
    // Verify correct precedence
}
```

### How to Add a New Statement Type

**Example**: Adding a `switch` statement

**Step 1**: Add statement variant to `ast.rs`
```rust
pub enum Stmt {
    // ... existing statements
    Switch {
        expr: Expr,
        cases: Vec<(Expr, Box<Stmt>)>,
        default: Option<Box<Stmt>>,
    },
}
```

**Step 2**: Add parsing method in `parse.rs`
```rust
fn switch_statement(&mut self) -> Result<Stmt, ParseError> {
    self.consume(TokenKind::Switch, "Expected 'switch'")?;
    self.consume(TokenKind::LeftParen, "Expected '('")?;

    let expr = self.expression()?;

    self.consume(TokenKind::RightParen, "Expected ')'")?;
    self.consume(TokenKind::LeftBrace, "Expected '{'")?;

    let mut cases = Vec::new();

    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
        if self.match_token(&[TokenKind::Case]) {
            let case_expr = self.expression()?;
            self.consume(TokenKind::Colon, "Expected ':'")?;
            let stmt = self.statement()?;
            cases.push((case_expr, Box::new(stmt)));
        }
        // ... handle default case
    }

    self.consume(TokenKind::RightBrace, "Expected '}'")?;

    Ok(Stmt::Switch { expr, cases, default })
}
```

**Step 3**: Call from main statement parser
```rust
fn statement(&mut self) -> Result<Stmt, ParseError> {
    if self.match_token(&[TokenKind::Switch]) {
        return self.switch_statement();
    }
    // ... other statement types
}
```

**Step 4**: Write comprehensive tests
```rust
#[test]
fn test_switch_statement() {
    let ast = parse("switch (x) { case 1: print(1) }");
    // Verify AST
}
```

### How to Add a New Expression Type

**Example**: Adding ternary operator `? :`

**Step 1**: Add expression variant to `ast.rs`
```rust
pub enum Expr {
    // ... existing expressions
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),  // condition ? true : false
}
```

**Step 2**: Add parsing at correct precedence level
```rust
fn ternary(&mut self) -> Result<Expr, ParseError> {
    let mut expr = self.logical_or()?;

    if self.match_token(&[TokenKind::Question]) {
        let then_expr = self.expression()?;
        self.consume(TokenKind::Colon, "Expected ':' in ternary")?;
        let else_expr = self.ternary()?;  // Right associative
        expr = Expr::Ternary(Box::new(expr), Box::new(then_expr), Box::new(else_expr));
    }

    Ok(expr)
}
```

**Step 3**: Update expression chain
```rust
fn expression(&mut self) -> Result<Expr, ParseError> {
    self.ternary()  // Start with lowest precedence
}
```

**Step 4**: Test thoroughly
```rust
#[test]
fn test_ternary() {
    let ast = parse("x > 5 ? 10 : 20");
    // Verify structure
}

#[test]
fn test_nested_ternary() {
    let ast = parse("a ? b : c ? d : e");
    // Should parse as: a ? b : (c ? d : e)
}
```

### Debugging Tips

**Problem**: Infinite loop during parsing
- **Check**: Are you advancing tokens in loops?
- **Check**: Do you have proper termination conditions?

**Problem**: Wrong operator precedence
- **Check**: Are methods called in correct order (lowest to highest)?
- **Check**: Did you update the precedence chain?

**Problem**: Syntax errors not caught
- **Check**: Are you using `consume()` for required tokens?
- **Check**: Do you have error recovery logic?

**Problem**: AST structure incorrect
- **Check**: Print AST with `{:#?}` formatting
- **Check**: Draw expected tree on paper first

### Testing Checklist

When adding new syntax:
- ✅ Parse valid syntax correctly
- ✅ Test operator precedence
- ✅ Test associativity (left/right)
- ✅ Test error cases (missing tokens)
- ✅ Test nested/complex cases
- ✅ Test edge cases (empty, single item)

## Usage

```rust
let tokens = scanner.scan_tokens()?;
let mut parser = Parser::new(tokens);
let program = parser.parse()?;
```

---

**Last Updated**: April 17, 2026
**Status**: 53 unit tests passing — no changes since initial implementation
