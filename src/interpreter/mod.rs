//! Interpreter module for executing Aether code

pub mod value;
pub mod environment;

#[cfg(test)]
mod interpreter_tests;

pub use value::Value;
pub use environment::{Environment, RuntimeError};
