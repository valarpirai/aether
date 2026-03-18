//! Aether Programming Language
//!
//! A general-purpose, dynamically typed programming language with automatic memory management.
//!
//! This crate provides:
//! - Lexer for tokenization
//! - Parser for AST generation
//! - Interpreter for execution
//! - REPL for interactive development

#![allow(missing_docs)]
#![warn(clippy::all)]

/// Module for lexical analysis (tokenization)
pub mod lexer;

/// Module for parsing tokens into an Abstract Syntax Tree
pub mod parser;

/// Module for interpreting and executing the AST
pub mod interpreter;
