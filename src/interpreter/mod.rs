//! Interpreter module for executing Aether code

pub mod builtins;
pub mod environment;
pub mod evaluator;
pub mod io_pool;
pub mod stdlib;
pub mod value;

#[cfg(test)]
mod interpreter_tests;

#[cfg(test)]
mod builtins_tests;

pub use environment::{Environment, RuntimeError};
pub use evaluator::Evaluator;
pub use value::Value;
