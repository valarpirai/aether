//! Lexer module for tokenizing Aether source code

pub mod scanner;
pub mod token;

#[cfg(test)]
mod lexer_tests;

pub use scanner::{LexerError, Scanner};
pub use token::{Token, TokenKind};
