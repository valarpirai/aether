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

**Literals:**
- `Integer(i64)`, `Float(f64)`, `Bool(bool)`, `Null`
- `StringLit(String)` — plain string literal
- `StringInterp(Vec<StringPart>)` — interpolated `"text ${expr} text"`

**Variables and Access:**
- `Identifier(String)`
- `Member(Box<Expr>, String)` — `obj.field`
- `OptionalMember(Box<Expr>, String)` — `obj?.field` (null-safe)
- `Index(Box<Expr>, Box<Expr>)` — `arr[idx]`
- `Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>)` — `s[1:3]`

**Operators:**
- `Binary(Box<Expr>, BinaryOp, Box<Expr>)`
- `Unary(UnaryOp, Box<Expr>)`
- `NullCoalesce(Box<Expr>, Box<Expr>)` — `a ?? b`

**Calls:**
- `Call(Box<Expr>, Vec<Expr>)` — function call
- `OptionalCall(Box<Expr>, String, Vec<Expr>)` — `obj?.method(args)` (null-safe)

**Constructors:**
- `Array(Vec<Expr>)` — array literal, may include `Spread(Box<Expr>)` elements
- `Dict(Vec<(Expr, Expr)>)` — dict literal `{key: val}`
- `StructInit(String, Vec<(String, Expr)>)` — `Point(x: 1, y: 2)`

**Functions:**
- `FunctionExpr(Vec<String>, Box<Stmt>)` — `fn(params) { body }`
- `AsyncFunctionExpr(Vec<String>, Box<Stmt>)` — `async fn(params) { body }`
- `Await(Box<Expr>)` — `await expr`

### Statements (Stmt)
- `Expr(Expr)` — expression statement
- `Let(String, Expr)` — variable declaration
- `Assign(Expr, Expr)` — assignment
- `CompoundAssign(Expr, BinaryOp, Expr)` — `+=`, `-=`, etc.
- `Block(Vec<Stmt>)` — block of statements
- `If(Expr, Box<Stmt>, Option<Box<Stmt>>)` — if/else
- `While(Expr, Box<Stmt>)` — while loop
- `For(String, Expr, Box<Stmt>)` — for-in single binding
- `ForKV(String, String, Expr, Box<Stmt>)` — for key, value in dict
- `Labeled(String, Box<Stmt>)` — labeled loop for break/continue targets
- `Break(Option<String>)` — break with optional label
- `Continue(Option<String>)` — continue with optional label
- `Return(Option<Expr>)` — return statement
- `Function(String, Vec<(String, Option<Expr>)>, Box<Stmt>)` — named function (params may have defaults)
- `AsyncFunction(String, Vec<(String, Option<Expr>)>, Box<Stmt>)` — `async fn`
- `Struct(String, Vec<String>, Vec<Stmt>)` — struct definition
- `Import(String, Option<String>)` — `import mod` or `import mod as alias`
- `FromImport(String, Vec<(String, Option<String>)>)` — `from mod import fn`
- `TryCatch(Box<Stmt>, String, Box<Stmt>, Option<Box<Stmt>>)` — try/catch/finally
- `Throw(Expr)` — throw expression

## Operator Precedence (Lowest to Highest)

1. Null coalescing (`??`)
2. Logical OR (`||`)
3. Logical AND (`&&`)
4. Equality (`==`, `!=`)
5. Comparison (`<`, `>`, `<=`, `>=`)
6. Addition/Subtraction (`+`, `-`)
7. Multiplication/Division/Modulo (`*`, `/`, `%`)
8. Unary (`-`, `!`)
9. Postfix (calls, indexing, member access, optional chaining `?.`)
10. Primary (literals, identifiers, grouping, `await`)

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

**Last Updated**: April 29, 2026
**Status**: 53 unit tests passing
