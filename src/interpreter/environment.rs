//! Variable environment with scoping support

use super::value::Value;
use std::collections::HashMap;

/// Runtime error types
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Undefined variable
    UndefinedVariable(String),
    /// Type mismatch in operation
    TypeError { expected: String, got: String },
    /// Division by zero
    DivisionByZero,
    /// Index out of bounds
    IndexOutOfBounds { index: i64, length: usize },
    /// Invalid operation
    InvalidOperation(String),
    /// Arity mismatch in function call
    ArityMismatch { expected: usize, got: usize },
    /// Recursion depth exceeded
    StackOverflow { depth: usize, limit: usize },
    /// User-thrown error (via throw statement)
    Thrown(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::TypeError { expected, got } => {
                write!(f, "Type error: expected {}, got {}", expected, got)
            }
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds { index, length } => {
                write!(
                    f,
                    "Index {} out of bounds for array of length {}",
                    index, length
                )
            }
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RuntimeError::ArityMismatch { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
            RuntimeError::StackOverflow { depth, limit } => {
                write!(
                    f,
                    "Maximum recursion depth exceeded: {} (limit: {})",
                    depth, limit
                )
            }
            RuntimeError::Thrown(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// One frame in the call stack — captured when building error objects
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Name of the function being called (or "<anonymous>")
    pub fn_name: String,
    /// Line in the caller where this call was made
    pub call_site_line: usize,
    /// File (and module) where the call was made, if known
    pub call_site_file: Option<String>,
}

/// Variable environment with lexical scoping
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variable bindings in this scope
    values: HashMap<String, Value>,
    /// Parent environment (for nested scopes)
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Create a new global environment
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent scope
    pub fn with_parent(parent: Environment) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a new variable in the current scope
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// Get a variable's value (searches parent scopes)
    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    /// Return all bindings defined directly in this (top-level) scope
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.values
    }

    /// Take the parent environment, leaving None in its place
    pub fn take_parent(&mut self) -> Option<Environment> {
        self.parent.take().map(|b| *b)
    }

    /// Set an existing variable's value (searches parent scopes)
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
