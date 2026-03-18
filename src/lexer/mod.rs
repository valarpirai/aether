//! Lexer module for tokenizing Aether source code

pub mod token;
pub mod scanner;

#[cfg(test)]
mod lexer_tests;

pub use token::{Token, TokenKind};
pub use scanner::{Scanner, LexerError};
