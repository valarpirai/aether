//! Runtime value types for the Aether interpreter

use std::fmt;
use std::rc::Rc;

use super::RuntimeError;

/// Type for built-in function implementations
pub type BuiltinFn = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Runtime value type with reference-counted garbage collection
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value (reference counted for GC)
    String(Rc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
    /// Array of values (reference counted for GC)
    Array(Rc<Vec<Value>>),
    /// Function with closure
    Function {
        params: Vec<String>,
        body: Box<crate::parser::ast::Stmt>,
        closure: Box<super::environment::Environment>,
    },
    /// Built-in function
    BuiltinFn {
        name: String,
        arity: usize, // Number of parameters (or usize::MAX for variadic)
        func: BuiltinFn,
    },
}

impl Value {
    /// Helper: Create a string value (for convenience)
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Rc::new(s.into()))
    }

    /// Helper: Create an array value (for convenience)
    pub fn array(vec: Vec<Value>) -> Self {
        Value::Array(Rc::new(vec))
    }

    /// Check if value is truthy (for conditional expressions)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::Array(a) if a.is_empty() => false,
            _ => true,
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Function { .. } => "function",
            Value::BuiltinFn { .. } => "builtin_function",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Function { .. }, Value::Function { .. }) => false, // Functions never equal
            (Value::BuiltinFn { .. }, Value::BuiltinFn { .. }) => false, // Built-ins never equal
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Value::Function { params, .. } => {
                write!(f, "<fn({})>", params.len())
            }
            Value::BuiltinFn { name, .. } => {
                write!(f, "<builtin {}>", name)
            }
        }
    }
}
