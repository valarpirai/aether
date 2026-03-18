//! Interpreter module for executing Aether code

pub mod value;
pub mod environment;
pub mod evaluator;
pub mod builtins;

#[cfg(test)]
mod interpreter_tests;

#[cfg(test)]
mod builtins_tests;

pub use value::Value;
pub use environment::{Environment, RuntimeError};
pub use evaluator::Evaluator;
