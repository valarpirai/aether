//! Scanner for tokenizing Aether source code

use super::token::{Token, TokenKind};
use std::fmt;

/// Lexer error types
#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(ch, line, col) => {
                write!(f, "Unexpected character '{}' at line {}, column {}", ch, line, col)
            }
            LexerError::UnterminatedString(line, col) => {
                write!(f, "Unterminated string at line {}, column {}", line, col)
            }
            LexerError::InvalidNumber(s, line, col) => {
                write!(f, "Invalid number '{}' at line {}, column {}", s, line, col)
            }
        }
    }
}

impl std::error::Error for LexerError {}

/// Scanner for tokenizing source code
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Scanner {
    /// Creates a new scanner from source code
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Scans all tokens from the source
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LexerError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            String::new(),
            self.line,
            self.column,
        ));

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), LexerError> {
        let c = self.advance();
        let start_column = self.column - 1;

        match c {
            ' ' | '\r' | '\t' => {} // Skip whitespace
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            '(' => self.add_token(TokenKind::LeftParen, start_column),
            ')' => self.add_token(TokenKind::RightParen, start_column),
            '{' => self.add_token(TokenKind::LeftBrace, start_column),
            '}' => self.add_token(TokenKind::RightBrace, start_column),
            '[' => self.add_token(TokenKind::LeftBracket, start_column),
            ']' => self.add_token(TokenKind::RightBracket, start_column),
            ',' => self.add_token(TokenKind::Comma, start_column),
            '.' => self.add_token(TokenKind::Dot, start_column),
            ':' => self.add_token(TokenKind::Colon, start_column),
            '%' => self.add_token(TokenKind::Percent, start_column),
            '+' => {
                let kind = if self.match_char('=') {
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                };
                self.add_token(kind, start_column);
            }
            '-' => {
                let kind = if self.match_char('=') {
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                };
                self.add_token(kind, start_column);
            }
            '*' => {
                let kind = if self.match_char('=') {
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                };
                self.add_token(kind, start_column);
            }
            '/' => {
                if self.match_char('/') {
                    // Single-line comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    // Multi-line comment
                    self.skip_multiline_comment()?;
                } else if self.match_char('=') {
                    self.add_token(TokenKind::SlashEqual, start_column);
                } else {
                    self.add_token(TokenKind::Slash, start_column);
                }
            }
            '!' => {
                let kind = if self.match_char('=') {
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                };
                self.add_token(kind, start_column);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind, start_column);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind, start_column);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind, start_column);
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenKind::And, start_column);
                } else {
                    return Err(LexerError::UnexpectedCharacter(c, self.line, start_column));
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenKind::Or, start_column);
                } else {
                    return Err(LexerError::UnexpectedCharacter(c, self.line, start_column));
                }
            }
            '"' => self.scan_string(start_column)?,
            _ => {
                if c.is_ascii_digit() {
                    self.scan_number(start_column)?;
                } else if c.is_alphabetic() || c == '_' {
                    self.scan_identifier(start_column);
                } else {
                    return Err(LexerError::UnexpectedCharacter(c, self.line, start_column));
                }
            }
        }

        Ok(())
    }

    fn skip_multiline_comment(&mut self) -> Result<(), LexerError> {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.advance();
        }
        Ok(())
    }

    fn scan_string(&mut self, start_column: usize) -> Result<(), LexerError> {
        let mut value = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(LexerError::UnterminatedString(self.line, start_column));
                }
                let escaped = match self.peek() {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    _ => self.peek(),
                };
                value.push(escaped);
                self.advance();
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(self.line, start_column));
        }

        self.advance(); // closing "

        self.add_token(TokenKind::String(value), start_column);
        Ok(())
    }

    fn scan_number(&mut self, start_column: usize) -> Result<(), LexerError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let mut is_float = false;
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let text: String = self.source[self.start..self.current].iter().collect();

        if is_float {
            match text.parse::<f64>() {
                Ok(value) => self.add_token(TokenKind::Float(value), start_column),
                Err(_) => {
                    return Err(LexerError::InvalidNumber(text, self.line, start_column));
                }
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => self.add_token(TokenKind::Integer(value), start_column),
                Err(_) => {
                    return Err(LexerError::InvalidNumber(text, self.line, start_column));
                }
            }
        }

        Ok(())
    }

    fn scan_identifier(&mut self, start_column: usize) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let kind = match text.as_str() {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            _ => TokenKind::Identifier(text.clone()),
        };

        self.add_token(kind, start_column);
    }

    fn add_token(&mut self, kind: TokenKind, column: usize) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(kind, text, self.line, column));
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
