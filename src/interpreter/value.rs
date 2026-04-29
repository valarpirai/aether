//! Runtime value types for the Aether interpreter

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::environment::{RuntimeError, StackFrame};

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
    /// Function with closure (body is Rc to avoid deep-cloning AST on env clone)
    Function {
        params: Vec<String>,
        body: Rc<crate::parser::ast::Stmt>,
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
    /// Async function — calling it returns a Promise instead of executing
    AsyncFunction {
        params: Vec<String>,
        body: Rc<crate::parser::ast::Stmt>,
        closure: Rc<super::environment::Environment>,
    },
    /// Deferred result of calling an async function
    Promise(Rc<RefCell<PromiseState>>),
    /// Runtime error object bound in catch blocks — e.message, e.stack_trace
    ErrorVal {
        message: String,
        stack_trace: String,
    },
    /// Lazy file-line iterator — reads one line at a time without buffering the whole file
    FileLines(Rc<RefCell<FileIterState>>),
    /// Enum type definition — holds all variant names and their field lists
    EnumDef {
        name: String,
        variants: Rc<Vec<(String, Vec<String>)>>,
    },
    /// Callable constructor for an enum variant with fields (e.g. Shape.Circle)
    EnumConstructor {
        enum_name: String,
        variant_name: String,
        fields: Vec<String>,
    },
    /// Runtime instance of an enum variant
    EnumVariant {
        enum_name: String,
        variant_name: String,
        /// Pre-computed "EnumName.VariantName" — returned by type()
        type_name: String,
        fields: Rc<Vec<(String, Value)>>,
    },
}

/// State of a Promise value
#[derive(Debug)]
pub enum PromiseState {
    /// Not yet resolved — holds the async function and pre-evaluated args
    Pending { func: Value, args: Vec<Value> },
    /// Already resolved — holds the result
    Resolved(Value),
    /// Waiting for an I/O worker to complete (Phase 2 thread pool)
    IoWaiting(std::sync::mpsc::Receiver<crate::interpreter::io_pool::IoResult>),
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

/// State for lazy file-line iteration
pub struct FileIterState {
    reader: std::io::BufReader<std::fs::File>,
    /// Pre-fetched next line; None means EOF was reached
    peeked: Option<String>,
}

impl FileIterState {
    pub fn open(path: &str) -> Result<Self, String> {
        use std::io::BufRead;
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut reader = std::io::BufReader::new(file);
        let mut buf = String::new();
        let n = reader.read_line(&mut buf).map_err(|e| e.to_string())?;
        let peeked = if n == 0 {
            None
        } else {
            Some(
                buf.trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .to_string(),
            )
        };
        Ok(Self { reader, peeked })
    }

    pub fn has_next(&self) -> bool {
        self.peeked.is_some()
    }

    /// Returns Ok(Some(line)) for the next line, Ok(None) at EOF,
    /// or Err(msg) if a read error occurs mid-iteration.
    pub fn next_line(&mut self) -> Result<Option<String>, String> {
        use std::io::BufRead;
        let current = self.peeked.take();
        let mut buf = String::new();
        match self.reader.read_line(&mut buf) {
            Ok(0) => {}
            Ok(_) => {
                self.peeked = Some(
                    buf.trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string(),
                );
            }
            Err(e) => return Err(format!("lines_iter read error: {}", e)),
        }
        Ok(current)
    }
}

impl std::fmt::Debug for FileIterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileIterState(peeked={:?})", self.peeked)
    }
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

    /// Helper: Open a file and create a lazy line iterator
    pub fn file_lines(path: &str) -> Result<Self, RuntimeError> {
        FileIterState::open(path)
            .map(|s| Value::FileLines(Rc::new(RefCell::new(s))))
            .map_err(|e| RuntimeError::InvalidOperation(format!("lines_iter failed: {}", e)))
    }

    /// Helper: Create a pending Promise wrapping a deferred async call
    pub fn promise(func: Value, args: Vec<Value>) -> Self {
        Value::Promise(Rc::new(RefCell::new(PromiseState::Pending { func, args })))
    }

    /// Build an ErrorVal from a RuntimeError message and a call-stack snapshot.
    pub fn error_val(message: String, stack: &[StackFrame], error_line: usize) -> Self {
        Value::ErrorVal {
            message,
            stack_trace: format_stack_trace(stack, error_line),
        }
    }

    /// Helper: Create an I/O-backed Promise (Phase 2 thread pool)
    pub fn promise_io(
        rx: std::sync::mpsc::Receiver<crate::interpreter::io_pool::IoResult>,
    ) -> Self {
        Value::Promise(Rc::new(RefCell::new(PromiseState::IoWaiting(rx))))
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
            Value::AsyncFunction { .. } => "async_function",
            Value::Promise(_) => "promise",
            Value::ErrorVal { .. } => "error",
            Value::FileLines(_) => "file_lines",
            Value::EnumDef { .. } => "enum",
            Value::EnumConstructor { .. } => "enum_constructor",
            Value::EnumVariant { type_name, .. } => type_name.as_str(),
        }
    }
}

/// Format a location string as "file.ae:N" or just "line N" when no file is known.
fn fmt_location(file: Option<&str>, line: usize) -> String {
    match file {
        Some(f) => format!("{}:{}", f, line),
        None => format!("line {}", line),
    }
}

/// Format a call-stack snapshot into a human-readable string.
/// The innermost frame is listed first (the function where the error occurred).
fn format_stack_trace(stack: &[StackFrame], error_line: usize) -> String {
    if stack.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    let innermost = &stack[stack.len() - 1];
    let inner_loc = fmt_location(innermost.call_site_file.as_deref(), error_line);
    lines.push(format!("  at {} ({})", innermost.fn_name, inner_loc));
    for i in (0..stack.len() - 1).rev() {
        let called_at_line = stack[i + 1].call_site_line;
        let loc = fmt_location(stack[i + 1].call_site_file.as_deref(), called_at_line);
        lines.push(format!("  at {} ({})", stack[i].fn_name, loc));
    }
    lines.join("\n")
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
            (Value::Iterator(_), Value::Iterator(_)) => false,
            (Value::AsyncFunction { .. }, Value::AsyncFunction { .. }) => false,
            (Value::Promise(_), Value::Promise(_)) => false,
            (Value::ErrorVal { message: a, .. }, Value::ErrorVal { message: b, .. }) => a == b,
            (Value::FileLines(_), Value::FileLines(_)) => false,
            (Value::EnumDef { name: a, .. }, Value::EnumDef { name: b, .. }) => a == b,
            (Value::EnumConstructor { .. }, Value::EnumConstructor { .. }) => false,
            (
                Value::EnumVariant {
                    type_name: ta,
                    fields: fa,
                    ..
                },
                Value::EnumVariant {
                    type_name: tb,
                    fields: fb,
                    ..
                },
            ) => ta == tb && fa == fb,
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
            Value::AsyncFunction { params, .. } => write!(f, "<async fn({})>", params.len()),
            Value::Promise(state) => match &*state.borrow() {
                PromiseState::Pending { .. } | PromiseState::IoWaiting(_) => {
                    write!(f, "<promise:pending>")
                }
                PromiseState::Resolved(v) => write!(f, "<promise:{}>", v),
            },
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
            Value::ErrorVal { message, .. } => write!(f, "{}", message),
            Value::FileLines(state) => {
                let s = state.borrow();
                if s.has_next() {
                    write!(f, "<file_lines:open>")
                } else {
                    write!(f, "<file_lines:eof>")
                }
            }
            Value::EnumDef { name, .. } => write!(f, "<enum {}>", name),
            Value::EnumConstructor {
                enum_name,
                variant_name,
                ..
            } => write!(f, "<constructor {}.{}>", enum_name, variant_name),
            Value::EnumVariant {
                type_name, fields, ..
            } => {
                let f_list = fields.as_ref();
                if f_list.is_empty() {
                    write!(f, "{}", type_name)
                } else {
                    write!(f, "{}(", type_name)?;
                    for (i, (_, v)) in f_list.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", v)?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
}
