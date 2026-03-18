# Aether Lexer Documentation

## Overview

The lexer (lexical analyzer) is the first phase of the Aether interpreter. It converts raw source code into a stream of tokens that can be parsed.

**Location**: `src/lexer/`

**Status**: ✅ Complete (14 tests passing)

## Architecture

```
Source Code (.ae file)
        ↓
    Scanner
        ↓
   Token Stream
        ↓
    To Parser
```

## Components

### 1. Token (`token.rs`)

Represents a single lexical unit in the source code.

#### TokenKind Enum

All token types supported by Aether:

**Literals**:
- `Integer(i64)` - Integer literal (e.g., `42`)
- `Float(f64)` - Float literal (e.g., `3.14`)
- `String(String)` - String literal (e.g., `"hello"`)
- `True` - Boolean true
- `False` - Boolean false
- `Null` - Null value

**Keywords**:
- `Let` - Variable declaration
- `Fn` - Function declaration
- `Return` - Return statement
- `If`, `Else` - Conditionals
- `While`, `For`, `In` - Loops
- `Break`, `Continue` - Loop control
- `Import`, `From`, `As` - Module system

**Operators**:
- Arithmetic: `Plus`, `Minus`, `Star`, `Slash`, `Percent`
- Assignment: `Equal`, `PlusEqual`, `MinusEqual`, `StarEqual`, `SlashEqual`
- Comparison: `EqualEqual`, `NotEqual`, `Less`, `Greater`, `LessEqual`, `GreaterEqual`
- Logical: `And`, `Or`, `Not`

**Delimiters**:
- `LeftParen`, `RightParen` - `(`, `)`
- `LeftBrace`, `RightBrace` - `{`, `}`
- `LeftBracket`, `RightBracket` - `[`, `]`
- `Comma`, `Dot`, `Colon` - `,`, `.`, `:`

**Special**:
- `Newline` - Line break
- `Eof` - End of file

#### Token Struct

```rust
pub struct Token {
    pub kind: TokenKind,      // Type of token
    pub lexeme: String,       // Original text
    pub line: usize,          // Line number (1-indexed)
    pub column: usize,        // Column number (1-indexed)
}
```

**Position tracking** enables helpful error messages with exact locations.

### 2. Scanner (`scanner.rs`)

The main lexer implementation that tokenizes source code.

#### Scanner Struct

```rust
pub struct Scanner {
    source: Vec<char>,        // Source code as characters
    tokens: Vec<Token>,       // Accumulated tokens
    start: usize,             // Start of current token
    current: usize,           // Current position
    line: usize,              // Current line
    column: usize,            // Current column
}
```

#### Main Method

```rust
pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LexerError>
```

Scans the entire source and returns all tokens or an error.

#### Error Types

```rust
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
}
```

Each error includes position information for debugging.

## Tokenization Process

### 1. Character-by-Character Scanning

The scanner reads one character at a time and determines what token to create:

```rust
fn scan_token(&mut self) -> Result<(), LexerError> {
    let c = self.advance();
    match c {
        ' ' | '\r' | '\t' => {} // Skip whitespace
        '\n' => { /* Track newlines */ }
        '(' => self.add_token(TokenKind::LeftParen),
        // ... more cases
    }
}
```

### 2. Number Tokenization

**Integer**: Sequence of digits
```
42 → Integer(42)
```

**Float**: Digits with decimal point
```
3.14 → Float(3.14)
```

**Process**:
1. Consume all digits
2. Check for decimal point followed by digits
3. Parse as i64 or f64
4. Return error if parsing fails

### 3. String Tokenization

**Syntax**: Text enclosed in double quotes `"text"`

**Features**:
- Escape sequences: `\n`, `\t`, `\\`, `\"`
- Multi-line strings supported
- UTF-8 encoding

**Process**:
1. Consume characters until closing `"`
2. Process escape sequences
3. Return error if unterminated

**Example**:
```
"hello\nworld" → String("hello\nworld")
```

### 4. Identifier and Keyword Tokenization

**Identifiers**: Start with letter or underscore, followed by alphanumeric or underscore

**Process**:
1. Consume all alphanumeric/underscore characters
2. Check if it's a keyword
3. Return keyword token or identifier token

**Keywords Map**:
```rust
match text.as_str() {
    "let" => TokenKind::Let,
    "fn" => TokenKind::Fn,
    "if" => TokenKind::If,
    // ... more keywords
    _ => TokenKind::Identifier(text)
}
```

### 5. Operator Tokenization

**Single-character operators**: `+`, `-`, `*`, `%`

**Multi-character operators**: Lookahead for second character
- `=` → `Equal` or `==` → `EqualEqual`
- `!` → `Not` or `!=` → `NotEqual`
- `+` → `Plus` or `+=` → `PlusEqual`

**Example**:
```rust
if self.match_char('=') {
    TokenKind::PlusEqual  // +=
} else {
    TokenKind::Plus       // +
}
```

### 6. Comment Handling

**Single-line**: `//` until end of line
```rust
if self.match_char('/') {
    // Skip until newline
    while self.peek() != '\n' && !self.is_at_end() {
        self.advance();
    }
}
```

**Multi-line**: `/* ... */`
```rust
if self.match_char('*') {
    while !self.is_at_end() {
        if self.peek() == '*' && self.peek_next() == '/' {
            self.advance(); // *
            self.advance(); // /
            return Ok(());
        }
        self.advance();
    }
}
```

## Examples

### Example 1: Simple Expression

**Input**:
```
let x = 10 + 20
```

**Output Tokens**:
```
1. Token { kind: Let, lexeme: "let", line: 1, column: 1 }
2. Token { kind: Identifier("x"), lexeme: "x", line: 1, column: 5 }
3. Token { kind: Equal, lexeme: "=", line: 1, column: 7 }
4. Token { kind: Integer(10), lexeme: "10", line: 1, column: 9 }
5. Token { kind: Plus, lexeme: "+", line: 1, column: 12 }
6. Token { kind: Integer(20), lexeme: "20", line: 1, column: 14 }
7. Token { kind: Eof, lexeme: "", line: 1, column: 16 }
```

### Example 2: Function Definition

**Input**:
```
fn add(a, b) {
    return a + b
}
```

**Output Tokens**:
```
1. Fn → "fn"
2. Identifier("add") → "add"
3. LeftParen → "("
4. Identifier("a") → "a"
5. Comma → ","
6. Identifier("b") → "b"
7. RightParen → ")"
8. LeftBrace → "{"
9. Return → "return"
10. Identifier("a") → "a"
11. Plus → "+"
12. Identifier("b") → "b"
13. RightBrace → "}"
14. Eof
```

### Example 3: String with Escapes

**Input**:
```
"hello\nworld"
```

**Output**:
```
Token {
    kind: String("hello\nworld"),  // Actual newline character
    lexeme: "\"hello\\nworld\"",   // Original text
    line: 1,
    column: 1
}
```

## Error Handling

### Unexpected Character

**Input**: `@#$`

**Error**:
```
LexerError::UnexpectedCharacter('@', 1, 1)
→ "Unexpected character '@' at line 1, column 1"
```

### Unterminated String

**Input**: `"hello`

**Error**:
```
LexerError::UnterminatedString(1, 1)
→ "Unterminated string at line 1, column 1"
```

### Invalid Number

**Input**: `123abc` (if lexer tries to parse as number)

**Error**:
```
LexerError::InvalidNumber("123abc", 1, 1)
→ "Invalid number '123abc' at line 1, column 1"
```

## Testing

**Test File**: `src/lexer/lexer_tests.rs`

**Coverage**: 14 tests

**Test Categories**:
1. **Token creation** - Basic token structure
2. **Literals** - Integers, floats, strings, booleans
3. **Escape sequences** - `\n`, `\t`, `\\`, `\"`
4. **Keywords** - All language keywords
5. **Operators** - Arithmetic, comparison, logical
6. **Identifiers** - Variable names
7. **Comments** - Single-line and multi-line
8. **Complete expressions** - Real code snippets

**Example Test**:
```rust
#[test]
fn test_tokenize_integer() {
    let mut scanner = Scanner::new("42");
    let tokens = scanner.scan_tokens().unwrap();
    assert_eq!(tokens.len(), 2); // integer + EOF
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
}
```

## Performance Considerations

### Current Implementation
- **Single pass**: Reads source once
- **Character-by-character**: Simple and correct
- **String allocation**: Each token stores its lexeme
- **Vec growth**: Tokens accumulated in vector

### Optimization Opportunities (Future)
- **String interning**: Reuse common strings
- **Arena allocation**: Reduce allocations
- **Lazy tokenization**: On-demand token generation
- **Parallel lexing**: For large files

## Usage

```rust
use aether::lexer::Scanner;

fn main() {
    let source = "let x = 42";
    let mut scanner = Scanner::new(source);

    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(error) => {
            eprintln!("Lexer error: {}", error);
        }
    }
}
```

## Implementation Notes

### Why Vec\<char\> Instead of &str?

```rust
source: Vec<char>  // Current
// vs
source: &str       // Alternative
```

**Reason**: Using `Vec<char>` allows:
- Easy indexing: `self.source[i]`
- UTF-8 handling: Each char is a valid Unicode scalar
- Lookahead: `peek()` and `peek_next()` are simple

**Trade-off**: More memory but simpler code

### Position Tracking

Both line and column are tracked for error messages:
```rust
'\n' => {
    self.line += 1;
    self.column = 1;
}
_ => {
    self.column += 1;
}
```

This enables precise error reporting.

## Integration with Parser

The lexer output flows directly into the parser:

```rust
// Lexer → Parser
let mut scanner = Scanner::new(source);
let tokens = scanner.scan_tokens()?;
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;
```

## References

- **Source**: `src/lexer/`
- **Tests**: `src/lexer/lexer_tests.rs`
- **Design**: `docs/DESIGN.md` - Token types and syntax
- **Development**: `docs/DEVELOPMENT.md` - Testing guidelines
