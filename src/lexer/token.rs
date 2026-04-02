/// A part of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// A literal string segment
    Literal(String),
    /// A placeholder expression (raw source text between ${ and })
    Placeholder(String),
}

/// Token types in the Aether language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    /// String with interpolation: "Hello ${name}"
    StringInterp(Vec<StringPart>),
    True,
    False,
    Null,

    // Identifiers and keywords
    Identifier(String),

    // Keywords
    Let,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Import,
    From,
    As,
    Try,
    Catch,
    Throw,

    // Operators
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    Equal,      // =
    PlusEqual,  // +=
    MinusEqual, // -=
    StarEqual,  // *=
    SlashEqual, // /=

    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    And, // &&
    Or,  // ||
    Not, // !

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Dot,          // .
    Colon,        // :

    // Special
    Newline,
    Eof,
}

/// A token with its kind, lexeme, and position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    /// Creates a new token
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}
