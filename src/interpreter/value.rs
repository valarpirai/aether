//! Runtime value types for the Aether interpreter

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::RuntimeError;

/// Type for built-in function implementations
pub type BuiltinFn = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Method map type: method name → (param names, body)
pub type MethodMap = Rc<HashMap<String, (Vec<String>, Box<crate::parser::ast::Stmt>)>>;

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
        closure: Rc<super::environment::Environment>,
    },
    /// Built-in function
    BuiltinFn {
        name: String,
        arity: usize, // Number of parameters (or usize::MAX for variadic)
        func: BuiltinFn,
    },
    /// Module namespace (name -> value map)
    Module {
        name: String,
        members: Rc<HashMap<String, Value>>,
    },
    /// Dictionary (ordered by insertion, key must be string/int/bool)
    Dict(Rc<Vec<(Value, Value)>>),
    /// Set (unique, unordered collection - only hashable types allowed)
    Set(Rc<HashSet<Value>>),
    /// Struct type definition (blueprint)
    StructDef {
        name: String,
        fields: Vec<String>,
        methods: MethodMap,
    },
    /// Struct instance (runtime object with mutable fields)
    Instance {
        type_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: MethodMap,
    },
    /// Iterator for lazy iteration over collections
    Iterator(Rc<RefCell<IteratorState>>),
}

/// Iterator state for sequential access to elements
#[derive(Debug, Clone)]
pub struct IteratorState {
    /// Source data being iterated
    pub source: IteratorSource,
    /// Current position in iteration
    pub index: usize,
}

/// Source data for iterators
#[derive(Debug, Clone)]
pub enum IteratorSource {
    /// Array iterator
    Array(Rc<Vec<Value>>),
    /// Dict iterator (over keys)
    DictKeys(Rc<Vec<(Value, Value)>>),
    /// Set iterator
    Set(Vec<Value>), // Convert HashSet to Vec for iteration
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

    /// Helper: Create a set value (for convenience)
    ///
    /// # Clippy Note
    /// The `mutable_key_type` warning is a false positive here. While `Value` contains
    /// `Rc` (which has interior mutability via refcounts), our `Hash` implementation
    /// only hashes the actual immutable data (int, float, string, bool, null), not the
    /// Rc pointers. The hash is stable and correct.
    #[allow(clippy::mutable_key_type)]
    pub fn set(set: HashSet<Value>) -> Self {
        Value::Set(Rc::new(set))
    }

    /// Helper: Create an iterator from a source
    pub fn iterator(source: IteratorSource) -> Self {
        Value::Iterator(Rc::new(RefCell::new(IteratorState { source, index: 0 })))
    }

    /// Check if value is hashable (can be used in sets/dict keys)
    pub fn is_hashable(&self) -> bool {
        matches!(
            self,
            Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Bool(_) | Value::Null
        )
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
            Value::Set(s) if s.is_empty() => false,
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
            Value::Module { .. } => "module",
            Value::Dict(_) => "dict",
            Value::Set(_) => "set",
            Value::StructDef { .. } => "struct",
            Value::Instance { type_name, .. } => type_name.as_str(),
            Value::Iterator(_) => "iterator",
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
            (Value::Function { .. }, Value::Function { .. }) => false,
            (Value::BuiltinFn { .. }, Value::BuiltinFn { .. }) => false,
            (Value::Module { name: a, .. }, Value::Module { name: b, .. }) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::StructDef { name: a, .. }, Value::StructDef { name: b, .. }) => a == b,
            (Value::Iterator(_), Value::Iterator(_)) => false, // Iterators never equal
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Int(n) => {
                0u8.hash(state);
                n.hash(state);
            }
            Value::Float(f) => {
                1u8.hash(state);
                f.to_bits().hash(state);
            }
            Value::String(s) => {
                2u8.hash(state);
                s.hash(state);
            }
            Value::Bool(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            Value::Null => {
                4u8.hash(state);
            }
            _ => {
                // Non-hashable types - shouldn't reach here
                // We'll prevent these at runtime
                panic!("Attempted to hash non-hashable value: {}", self.type_name());
            }
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
            Value::Module { name, .. } => {
                write!(f, "<module {}>", name)
            }
            Value::Dict(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(elements) => {
                write!(f, "set(")?;
                let mut sorted: Vec<&Value> = elements.iter().collect();
                sorted.sort_by_key(|v| format!("{}", v));
                for (i, elem) in sorted.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            Value::Iterator(_) => write!(f, "<iterator>"),
            Value::StructDef { name, .. } => write!(f, "<struct {}>", name),
            Value::Instance {
                type_name, fields, ..
            } => {
                write!(f, "{} {{ ", type_name)?;
                let map = fields.borrow();
                let mut sorted: Vec<(&String, &Value)> = map.iter().collect();
                sorted.sort_by_key(|(k, _)| k.as_str());
                for (i, (k, v)) in sorted.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, " }}")
            }
        }
    }
}
