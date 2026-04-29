//! Variable environment with scoping support

use super::value::Value;
use std::collections::HashMap;

/// Runtime error types
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    // --- Core language ---
    UndefinedVariable(String),
    TypeError {
        expected: String,
        got: String,
    },
    DivisionByZero,
    IndexOutOfBounds {
        index: i64,
        length: usize,
    },
    ArityMismatch {
        expected: usize,
        got: usize,
    },
    StackOverflow {
        depth: usize,
        limit: usize,
    },
    /// User-thrown error (via throw statement)
    Thrown(String),
    /// Catch-all for rare cases not yet categorised
    InvalidOperation(String),

    // --- Type conversion ---
    ConversionError {
        from_type: String,
        to_type: String,
        value: String,
    },

    // --- I/O ---
    IoError {
        operation: String,
        detail: String,
    },
    ChannelClosed,

    // --- HTTP / network ---
    HttpError {
        url: String,
        detail: String,
    },

    // --- Parsing ---
    ParseError {
        format: String,
        detail: String,
    },

    // --- Member / property access ---
    PropertyNotFound {
        type_name: String,
        property: String,
    },
    MethodNotFound {
        type_name: String,
        method: String,
    },
    EnumVariantNotFound {
        enum_name: String,
        variant: String,
    },
    DictKeyNotFound(String),

    // --- Modules / imports ---
    CircularImport {
        module: String,
    },
    ModuleNotFound {
        module: String,
    },
    ModuleLoadError {
        module: String,
        reason: String,
    },

    // --- Function calls / async ---
    NotCallable {
        type_name: String,
    },
    AsyncNotAwaited {
        fn_name: String,
    },
    QueueFull {
        limit: usize,
    },

    // --- Operators / expressions ---
    InvalidSpread {
        got: String,
    },
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
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RuntimeError::ConversionError {
                from_type,
                to_type,
                value,
            } => {
                write!(f, "Cannot convert {} '{}' to {}", from_type, value, to_type)
            }
            RuntimeError::IoError { operation, detail } => {
                write!(f, "I/O error in {}: {}", operation, detail)
            }
            RuntimeError::ChannelClosed => write!(f, "I/O channel closed unexpectedly"),
            RuntimeError::HttpError { url, detail } => {
                write!(f, "HTTP error for '{}': {}", url, detail)
            }
            RuntimeError::ParseError { format, detail } => {
                write!(f, "Parse error ({}): {}", format, detail)
            }
            RuntimeError::PropertyNotFound {
                type_name,
                property,
            } => {
                write!(f, "'{}' has no property '{}'", type_name, property)
            }
            RuntimeError::MethodNotFound { type_name, method } => {
                write!(
                    f,
                    "Method '{}' does not exist on type '{}'",
                    method, type_name
                )
            }
            RuntimeError::EnumVariantNotFound { enum_name, variant } => {
                write!(f, "Enum '{}' has no variant '{}'", enum_name, variant)
            }
            RuntimeError::DictKeyNotFound(key) => {
                write!(f, "Key '{}' not found in dict", key)
            }
            RuntimeError::CircularImport { module } => {
                write!(f, "Circular import detected: '{}'", module)
            }
            RuntimeError::ModuleNotFound { module } => {
                write!(f, "Module not found: '{}'", module)
            }
            RuntimeError::ModuleLoadError { module, reason } => {
                write!(f, "Failed to load module '{}': {}", module, reason)
            }
            RuntimeError::NotCallable { type_name } => {
                write!(f, "Value of type '{}' is not callable", type_name)
            }
            RuntimeError::AsyncNotAwaited { fn_name } => {
                write!(
                    f,
                    "Async function '{}' must be called with 'await'",
                    fn_name
                )
            }
            RuntimeError::QueueFull { limit } => {
                write!(f, "Event loop queue is full (limit: {})", limit)
            }
            RuntimeError::InvalidSpread { got } => {
                write!(f, "Spread operator requires an array, got '{}'", got)
            }
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
