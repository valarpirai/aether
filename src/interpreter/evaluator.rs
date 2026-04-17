//! Expression evaluation and statement execution for the Aether interpreter

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::environment::{Environment, RuntimeError};
use super::value::Value;
use crate::parser::ast::{BinaryOp, Expr, Stmt, UnaryOp};

/// Control flow signals
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    None,
    Return(Value),
    Break,
    Continue,
}

/// Interpreter for evaluating expressions
pub struct Evaluator {
    /// Current environment
    pub environment: Environment,
    /// Current call depth (for recursion limit)
    call_depth: usize,
    /// Maximum allowed call depth
    max_call_depth: usize,
    /// Module cache to prevent circular dependencies
    module_cache: HashMap<String, Environment>,
    /// Tracks modules currently being loaded (for circular dependency detection)
    loading_stack: Vec<String>,
    /// Current file being executed (for relative imports)
    current_file: Option<PathBuf>,
}

impl Evaluator {
    /// Create a new evaluator with a fresh environment (includes stdlib)
    pub fn new() -> Self {
        Self::new_with_stdlib()
    }

    /// Create a new evaluator with stdlib loaded
    pub fn new_with_stdlib() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
            call_depth: 0,
            max_call_depth: 100, // Reduced to prevent Rust stack overflow
            module_cache: HashMap::new(),
            loading_stack: Vec::new(),
            current_file: None,
        };
        evaluator.register_builtins();
        evaluator.load_stdlib();
        evaluator
    }

    /// Create a new evaluator without stdlib (faster for tests)
    pub fn new_without_stdlib() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
            call_depth: 0,
            max_call_depth: 100, // Reduced to prevent Rust stack overflow
            module_cache: HashMap::new(),
            loading_stack: Vec::new(),
            current_file: None,
        };
        evaluator.register_builtins();
        evaluator
    }

    /// Register all built-in functions in the environment
    fn register_builtins(&mut self) {
        use super::builtins;

        // I/O functions
        self.environment.define(
            "print".to_string(),
            Value::BuiltinFn {
                name: "print".to_string(),
                arity: usize::MAX, // variadic
                func: builtins::builtin_print,
            },
        );

        self.environment.define(
            "println".to_string(),
            Value::BuiltinFn {
                name: "println".to_string(),
                arity: usize::MAX, // variadic
                func: builtins::builtin_println,
            },
        );

        // Type introspection
        self.environment.define(
            "type".to_string(),
            Value::BuiltinFn {
                name: "type".to_string(),
                arity: 1,
                func: builtins::builtin_type,
            },
        );

        self.environment.define(
            "len".to_string(),
            Value::BuiltinFn {
                name: "len".to_string(),
                arity: 1,
                func: builtins::builtin_len,
            },
        );

        // Type conversions
        self.environment.define(
            "int".to_string(),
            Value::BuiltinFn {
                name: "int".to_string(),
                arity: 1,
                func: builtins::builtin_int,
            },
        );

        self.environment.define(
            "float".to_string(),
            Value::BuiltinFn {
                name: "float".to_string(),
                arity: 1,
                func: builtins::builtin_float,
            },
        );

        self.environment.define(
            "str".to_string(),
            Value::BuiltinFn {
                name: "str".to_string(),
                arity: 1,
                func: builtins::builtin_str,
            },
        );

        self.environment.define(
            "bool".to_string(),
            Value::BuiltinFn {
                name: "bool".to_string(),
                arity: 1,
                func: builtins::builtin_bool,
            },
        );

        // I/O functions
        self.environment.define(
            "read_file".to_string(),
            Value::BuiltinFn {
                name: "read_file".to_string(),
                arity: 1,
                func: builtins::builtin_read_file,
            },
        );

        self.environment.define(
            "write_file".to_string(),
            Value::BuiltinFn {
                name: "write_file".to_string(),
                arity: 2,
                func: builtins::builtin_write_file,
            },
        );

        self.environment.define(
            "input".to_string(),
            Value::BuiltinFn {
                name: "input".to_string(),
                arity: usize::MAX, // variadic: 0 or 1 args
                func: builtins::builtin_input,
            },
        );

        // Time functions
        self.environment.define(
            "clock".to_string(),
            Value::BuiltinFn {
                name: "clock".to_string(),
                arity: 0,
                func: builtins::builtin_clock,
            },
        );

        self.environment.define(
            "sleep".to_string(),
            Value::BuiltinFn {
                name: "sleep".to_string(),
                arity: 1,
                func: builtins::builtin_sleep,
            },
        );

        // JSON functions
        self.environment.define(
            "json_parse".to_string(),
            Value::BuiltinFn {
                name: "json_parse".to_string(),
                arity: 1,
                func: builtins::builtin_json_parse,
            },
        );

        self.environment.define(
            "json_stringify".to_string(),
            Value::BuiltinFn {
                name: "json_stringify".to_string(),
                arity: 1,
                func: builtins::builtin_json_stringify,
            },
        );
    }

    /// Load standard library modules
    fn load_stdlib(&mut self) {
        use super::stdlib;
        use crate::lexer::Scanner;
        use crate::parser::Parser;

        for (name, source) in stdlib::stdlib_modules() {
            // Parse the module
            let mut scanner = Scanner::new(source);
            let tokens = match scanner.scan_tokens() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to tokenize stdlib module '{}': {}",
                        name, e
                    );
                    continue;
                }
            };

            let mut parser = Parser::new(tokens);
            let program = match parser.parse() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Warning: Failed to parse stdlib module '{}': {}", name, e);
                    continue;
                }
            };

            // Execute each module in an isolated evaluator so that closures only
            // capture a small bootstrap environment, not the ever-growing main env.
            let mut module_eval = Evaluator::new_without_stdlib();
            for stmt in &program.statements {
                if let Err(e) = module_eval.exec_stmt(stmt) {
                    eprintln!("Warning: Failed to execute stdlib module '{}': {}", name, e);
                    break;
                }
            }

            // Copy only the newly defined names into the main environment
            let bindings: Vec<(String, Value)> = module_eval
                .environment
                .bindings()
                .iter()
                .filter(|(k, _)| !k.starts_with("__builtin"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            for (fname, fval) in bindings {
                self.environment.define(fname, fval);
            }
        }
    }

    /// Evaluate an expression
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Integer(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(Rc::new(s.clone()))),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Identifier(name) => self.environment.get(name),
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    if let Expr::Spread(inner) = elem {
                        match self.eval_expr(inner)? {
                            Value::Array(arr) => {
                                for item in arr.iter() {
                                    values.push(item.clone());
                                }
                            }
                            other => {
                                return Err(RuntimeError::InvalidOperation(format!(
                                    "spread operator requires an array, got {}",
                                    other.type_name()
                                )))
                            }
                        }
                    } else {
                        values.push(self.eval_expr(elem)?);
                    }
                }
                Ok(Value::Array(Rc::new(values)))
            }
            Expr::FunctionExpr(params, body) => {
                // Create function value with current environment as closure
                Ok(Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: Box::new(self.environment.clone()),
                })
            }
            Expr::StringInterp(parts) => {
                let mut result = String::new();
                for part in parts {
                    let val = self.eval_expr(part)?;
                    result.push_str(&format!("{}", val));
                }
                Ok(Value::string(result))
            }
            Expr::Unary(op, operand) => self.eval_unary(*op, operand),
            Expr::Binary(left, op, right) => self.eval_binary(left, *op, right),
            Expr::Index(array, index) => self.eval_index(array, index),
            Expr::Slice(object, start, end) => {
                self.eval_slice(object, start.as_deref(), end.as_deref())
            }
            Expr::Member(object, member) => self.eval_member(object, member),
            Expr::Call(callee, args) => self.eval_call(callee, args),
            Expr::Dict(pairs) => {
                let mut evaluated = Vec::new();
                for (k, v) in pairs {
                    evaluated.push((self.eval_expr(k)?, self.eval_expr(v)?));
                }
                Ok(Value::Dict(Rc::new(evaluated)))
            }
            Expr::Spread(_) => Err(RuntimeError::InvalidOperation(
                "spread operator is only valid inside array literals".to_string(),
            )),
            Expr::StructInit { name, fields } => {
                // Look up the struct definition
                let struct_def = self.environment.get(name)?;
                match struct_def {
                    Value::StructDef {
                        fields: def_fields,
                        methods,
                        ..
                    } => {
                        // Build field map; start with nulls for declared fields
                        let mut field_map: HashMap<String, Value> = def_fields
                            .iter()
                            .map(|f| (f.clone(), Value::Null))
                            .collect();
                        // Fill in provided values
                        for (field_name, field_expr) in fields {
                            let val = self.eval_expr(field_expr)?;
                            field_map.insert(field_name.clone(), val);
                        }
                        Ok(Value::Instance {
                            type_name: name.clone(),
                            fields: Rc::new(RefCell::new(field_map)),
                            methods,
                        })
                    }
                    other => Err(RuntimeError::InvalidOperation(format!(
                        "'{}' is not a struct (got {})",
                        name,
                        other.type_name()
                    ))),
                }
            }
        }
    }

    /// Evaluate unary operation
    fn eval_unary(&mut self, op: UnaryOp, operand: &Expr) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(operand)?;

        match op {
            UnaryOp::Negate => match value {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::TypeError {
                    expected: "number".to_string(),
                    got: value.type_name().to_string(),
                }),
            },
            UnaryOp::Not => Ok(Value::Bool(!value.is_truthy())),
        }
    }

    /// Evaluate binary operation
    fn eval_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left_val = self.eval_expr(left)?;
        let right_val = self.eval_expr(right)?;

        match op {
            BinaryOp::Add => self.eval_add(left_val, right_val),
            BinaryOp::Subtract => {
                self.eval_arithmetic(left_val, right_val, |a, b| a - b, |a, b| a - b)
            }
            BinaryOp::Multiply => {
                self.eval_arithmetic(left_val, right_val, |a, b| a * b, |a, b| a * b)
            }
            BinaryOp::Divide => self.eval_divide(left_val, right_val),
            BinaryOp::Modulo => self.eval_modulo(left_val, right_val),
            BinaryOp::Equal => Ok(Value::Bool(self.values_equal(&left_val, &right_val))),
            BinaryOp::NotEqual => Ok(Value::Bool(!self.values_equal(&left_val, &right_val))),
            BinaryOp::Less => self.eval_comparison(left_val, right_val, |a, b| a < b, |a, b| a < b),
            BinaryOp::Greater => {
                self.eval_comparison(left_val, right_val, |a, b| a > b, |a, b| a > b)
            }
            BinaryOp::LessEqual => {
                self.eval_comparison(left_val, right_val, |a, b| a <= b, |a, b| a <= b)
            }
            BinaryOp::GreaterEqual => {
                self.eval_comparison(left_val, right_val, |a, b| a >= b, |a, b| a >= b)
            }
            BinaryOp::And => {
                if !left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            }
            BinaryOp::Or => {
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            }
        }
    }

    /// Evaluate addition (handles string concatenation)
    fn eval_add(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            (left, right) => Err(RuntimeError::TypeError {
                expected: "number or string".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Evaluate arithmetic operation
    fn eval_arithmetic<F, G>(
        &self,
        left: Value,
        right: Value,
        int_op: F,
        float_op: G,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(i64, i64) -> i64,
        G: FnOnce(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(a, b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(a, b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(a as f64, b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(a, b as f64))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Evaluate division (checks for division by zero)
    fn eval_divide(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b as f64))
                }
            }
            (left, right) => Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Evaluate modulo operation
    fn eval_modulo(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            (left, right) => Err(RuntimeError::TypeError {
                expected: "integer".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Evaluate comparison operation
    fn eval_comparison<F, G>(
        &self,
        left: Value,
        right: Value,
        int_op: F,
        float_op: G,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(i64, i64) -> bool,
        G: FnOnce(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(int_op(a, b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(float_op(a, b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(float_op(a as f64, b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(float_op(a, b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)), // lexicographic
            (left, right) => Err(RuntimeError::TypeError {
                expected: "comparable types".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    /// Evaluate array/string indexing
    fn eval_index(&mut self, array: &Expr, index: &Expr) -> Result<Value, RuntimeError> {
        let array_val = self.eval_expr(array)?;
        let index_val = self.eval_expr(index)?;

        match (array_val, index_val) {
            (Value::Array(elements), Value::Int(idx)) => {
                if idx < 0 || idx as usize >= elements.len() {
                    Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: elements.len(),
                    })
                } else {
                    Ok(elements[idx as usize].clone())
                }
            }
            (Value::String(s), Value::Int(idx)) => {
                // Get character at index (UTF-8 aware)
                let chars: Vec<char> = s.chars().collect();
                if idx < 0 || idx as usize >= chars.len() {
                    Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: chars.len(),
                    })
                } else {
                    // Return single character as string
                    Ok(Value::string(chars[idx as usize].to_string()))
                }
            }
            // Dict index access: d["key"] or d[0]
            (Value::Dict(pairs), key) => {
                for (k, v) in pairs.iter() {
                    if k == &key {
                        return Ok(v.clone());
                    }
                }
                Err(RuntimeError::InvalidOperation(format!(
                    "Key {} not found in dict",
                    key
                )))
            }
            (collection, index) => Err(RuntimeError::TypeError {
                expected: "array or string with integer index".to_string(),
                got: format!("{} and {}", collection.type_name(), index.type_name()),
            }),
        }
    }

    /// Evaluate slice access: obj[start:end]
    fn eval_slice(
        &mut self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
    ) -> Result<Value, RuntimeError> {
        let obj_val = self.eval_expr(object)?;

        let start_val = match start {
            Some(s) => Some(self.eval_expr(s)?),
            None => None,
        };
        let end_val = match end {
            Some(e) => Some(self.eval_expr(e)?),
            None => None,
        };

        fn resolve_index(n: i64, len: i64) -> usize {
            if n < 0 {
                (len + n).max(0) as usize
            } else {
                n.min(len) as usize
            }
        }

        fn to_int(val: Value) -> Result<i64, RuntimeError> {
            match val {
                Value::Int(n) => Ok(n),
                other => Err(RuntimeError::TypeError {
                    expected: "int".to_string(),
                    got: other.type_name().to_string(),
                }),
            }
        }

        match obj_val {
            Value::Array(elements) => {
                let len = elements.len() as i64;
                let s = match start_val {
                    Some(v) => resolve_index(to_int(v)?, len),
                    None => 0,
                };
                let e = match end_val {
                    Some(v) => resolve_index(to_int(v)?, len),
                    None => len as usize,
                };
                let result = if s >= e {
                    vec![]
                } else {
                    elements[s..e].to_vec()
                };
                Ok(Value::array(result))
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len() as i64;
                let start_i = match start_val {
                    Some(v) => resolve_index(to_int(v)?, len),
                    None => 0,
                };
                let end_i = match end_val {
                    Some(v) => resolve_index(to_int(v)?, len),
                    None => len as usize,
                };
                let result: String = if start_i >= end_i {
                    String::new()
                } else {
                    chars[start_i..end_i].iter().collect()
                };
                Ok(Value::string(result))
            }
            other => Err(RuntimeError::InvalidOperation(format!(
                "slice not supported on {}",
                other.type_name()
            ))),
        }
    }

    /// Evaluate member access (obj.member)
    fn eval_member(&mut self, object: &Expr, member: &str) -> Result<Value, RuntimeError> {
        let obj_val = self.eval_expr(object)?;

        match (&obj_val, member) {
            // Array properties
            (Value::Array(elements), "length") => Ok(Value::Int(elements.len() as i64)),

            // String properties
            (Value::String(s), "length") => Ok(Value::Int(s.len() as i64)),

            // Dict member access: d.key (sugar for d["key"]), d.length returns count
            (Value::Dict(pairs), key) => {
                if key == "length" {
                    return Ok(Value::Int(pairs.len() as i64));
                }
                let key_val = Value::string(key.to_string());
                for (k, v) in pairs.iter() {
                    if k == &key_val {
                        return Ok(v.clone());
                    }
                }
                Err(RuntimeError::InvalidOperation(format!(
                    "Key '{}' not found in dict",
                    key
                )))
            }

            // Module member access
            (Value::Module { name, members }, prop) => {
                members.get(prop).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Module '{}' has no member '{}'",
                        name, prop
                    ))
                })
            }

            // Instance field access
            (
                Value::Instance {
                    type_name, fields, ..
                },
                prop,
            ) => {
                let map = fields.borrow();
                map.get(prop).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Field '{}' does not exist on '{}'",
                        prop, type_name
                    ))
                })
            }

            // Undefined property
            (obj, prop) => Err(RuntimeError::InvalidOperation(format!(
                "Property '{}' does not exist on type '{}'",
                prop,
                obj.type_name()
            ))),
        }
    }

    /// Evaluate method call (obj.method(args))
    /// Call a Value::Function or Value::BuiltinFn with already-evaluated arguments.
    fn call_value(&mut self, func: Value, arg_values: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Function {
                params,
                body,
                closure,
            } => {
                self.call_depth += 1;
                if self.call_depth > self.max_call_depth {
                    self.call_depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.call_depth + 1,
                        limit: self.max_call_depth,
                    });
                }
                if arg_values.len() > params.len() {
                    self.call_depth -= 1;
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: arg_values.len(),
                    });
                }
                let mut padded = arg_values;
                while padded.len() < params.len() {
                    padded.push(Value::Null);
                }
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());
                for (param, value) in params.iter().zip(padded) {
                    self.environment.define(param.clone(), value);
                }
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => Err(e),
                };
                self.environment = saved_env;
                self.call_depth -= 1;
                result
            }
            Value::BuiltinFn { arity, func, .. } => {
                if arity != usize::MAX && arity != arg_values.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: arity,
                        got: arg_values.len(),
                    });
                }
                func(&arg_values)
            }
            other => Err(RuntimeError::InvalidOperation(format!(
                "Cannot call value of type '{}'",
                other.type_name()
            ))),
        }
    }

    fn eval_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, RuntimeError> {
        let obj_val = self.eval_expr(object)?;

        match (&obj_val, method) {
            // Array methods
            (Value::Array(elements), "push") => {
                // Evaluate argument
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let item = self.eval_expr(&args[0])?;

                // Clone the inner Vec, mutate it, wrap in new Rc
                let mut new_elements = (**elements).to_vec();
                new_elements.push(item);

                // Update in environment (only works for identifiers)
                if let Expr::Identifier(name) = object {
                    self.environment
                        .set(name, Value::Array(Rc::new(new_elements)))?;
                }

                Ok(Value::Null)
            }
            (Value::Array(elements), "pop") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }

                // Clone the inner Vec, mutate it, wrap in new Rc
                let mut new_elements = (**elements).to_vec();
                let popped = new_elements.pop();

                // Update in environment (only works for identifiers)
                if let Expr::Identifier(name) = object {
                    self.environment
                        .set(name, Value::Array(Rc::new(new_elements)))?;
                }

                Ok(popped.unwrap_or(Value::Null))
            }

            // String methods
            (Value::String(s), "upper") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                Ok(Value::String(Rc::new(s.to_uppercase())))
            }
            (Value::String(s), "lower") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                Ok(Value::String(Rc::new(s.to_lowercase())))
            }
            (Value::String(s), "trim") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                Ok(Value::String(Rc::new(s.trim().to_string())))
            }
            (Value::String(s), "split") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let delimiter = self.eval_expr(&args[0])?;
                if let Value::String(delim) = delimiter {
                    let parts: Vec<Value> = if s.is_empty() {
                        vec![]
                    } else {
                        s.split(delim.as_str())
                            .map(|part| Value::String(Rc::new(part.to_string())))
                            .collect()
                    };
                    Ok(Value::Array(Rc::new(parts)))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "string".to_string(),
                        got: delimiter.type_name().to_string(),
                    })
                }
            }

            // Module member call: module.func(args)
            (Value::Module { name, members }, method) => {
                let func = members.get(method).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Module '{}' has no member '{}'",
                        name, method
                    ))
                })?;
                // Evaluate arguments then call the function
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                self.call_value(func, arg_values)
            }

            // Instance method call: instance.method(args)
            (
                Value::Instance {
                    type_name,
                    fields,
                    methods,
                },
                meth,
            ) => {
                let method = methods.get(meth).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Method '{}' does not exist on '{}'",
                        meth, type_name
                    ))
                })?;
                let (params, body) = method;
                let instance = Value::Instance {
                    type_name: type_name.clone(),
                    fields: Rc::clone(fields),
                    methods: Rc::clone(methods),
                };
                // Evaluate call arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                // Execute method with self bound and extra params
                self.call_depth += 1;
                if self.call_depth > self.max_call_depth {
                    self.call_depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.call_depth + 1,
                        limit: self.max_call_depth,
                    });
                }
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent(self.environment.clone());
                // Bind 'self' as first param
                self.environment.define("self".to_string(), instance);
                // Bind remaining params (skip first if it is 'self')
                let user_params: &[String] = if params.first().map(|s| s.as_str()) == Some("self") {
                    &params[1..]
                } else {
                    &params
                };
                let mut padded = arg_values;
                while padded.len() < user_params.len() {
                    padded.push(Value::Null);
                }
                for (param, val) in user_params.iter().zip(padded) {
                    self.environment.define(param.clone(), val);
                }
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => Err(e),
                };
                self.environment = saved_env;
                self.call_depth -= 1;
                result
            }

            // Undefined method
            (obj, meth) => Err(RuntimeError::InvalidOperation(format!(
                "Method '{}' does not exist on type '{}'",
                meth,
                obj.type_name()
            ))),
        }
    }

    /// Execute a statement (public interface)
    pub fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        self.exec_stmt_internal(stmt)?;
        Ok(())
    }

    /// Execute a statement (internal with control flow)
    fn exec_stmt_internal(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::Let(name, initializer) => {
                let value = self.eval_expr(initializer)?;
                self.environment.define(name.clone(), value);
                Ok(ControlFlow::None)
            }
            Stmt::Assign(target, value) => {
                let val = self.eval_expr(value)?;
                self.assign_target(target, val)?;
                Ok(ControlFlow::None)
            }
            Stmt::CompoundAssign(target, op, value) => {
                let current = self.eval_expr(target)?;
                let rhs = self.eval_expr(value)?;
                let result = self.eval_binary_values(current, *op, rhs)?;
                self.assign_target(target, result)?;
                Ok(ControlFlow::None)
            }
            Stmt::Block(statements) => {
                // Execute statements in block (no new scope for now - will add proper scoping with functions)
                for statement in statements {
                    let flow = self.exec_stmt_internal(statement)?;
                    if flow != ControlFlow::None {
                        return Ok(flow);
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.exec_stmt_internal(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.exec_stmt_internal(else_stmt)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While(condition, body) => {
                loop {
                    let cond_val = self.eval_expr(condition)?;
                    if !cond_val.is_truthy() {
                        break;
                    }

                    let flow = self.exec_stmt_internal(body)?;
                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::None => {}
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For(var, iterable, body) => {
                let iter_val = self.eval_expr(iterable)?;

                match iter_val {
                    Value::Array(elements) => {
                        for element in elements.iter() {
                            self.environment.define(var.clone(), element.clone());

                            let flow = self.exec_stmt_internal(body)?;
                            match flow {
                                ControlFlow::Break => break,
                                ControlFlow::Continue => continue,
                                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                                ControlFlow::None => {}
                            }
                        }
                        Ok(ControlFlow::None)
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "iterable (array)".to_string(),
                        got: iter_val.type_name().to_string(),
                    }),
                }
            }
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Function(name, params, body) => {
                // Create function with temporary closure
                let temp_func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: Box::new(self.environment.clone()),
                };

                // Define function in environment
                self.environment.define(name.clone(), temp_func);

                // Now re-create the function with updated closure that includes itself
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: Box::new(self.environment.clone()),
                };

                // Update with final version
                self.environment.set(&name, func)?;
                Ok(ControlFlow::None)
            }
            Stmt::Import(module_name) => {
                // Load module and add to environment
                self.load_module(&module_name)?;
                Ok(ControlFlow::None)
            }
            Stmt::ImportAs(module_name, alias) => {
                // Load module and add with alias
                self.load_module_as(&module_name, &alias)?;
                Ok(ControlFlow::None)
            }
            Stmt::FromImport(module_name, items) => {
                self.from_import(&module_name, &items)?;
                Ok(ControlFlow::None)
            }
            Stmt::FromImportAs(module_name, aliased_items) => {
                self.from_import_as(&module_name, &aliased_items)?;
                Ok(ControlFlow::None)
            }
            Stmt::Throw(expr) => {
                let value = self.eval_expr(expr)?;
                let msg = format!("{}", value);
                Err(RuntimeError::Thrown(msg))
            }
            Stmt::StructDecl {
                name,
                fields,
                methods,
            } => {
                let mut method_map = HashMap::new();
                for (method_name, params, body) in methods {
                    method_map.insert(method_name.clone(), (params.clone(), body.clone()));
                }
                let struct_def = Value::StructDef {
                    name: name.clone(),
                    fields: fields.clone(),
                    methods: Rc::new(method_map),
                };
                self.environment.define(name.clone(), struct_def);
                Ok(ControlFlow::None)
            }
            Stmt::TryCatch(try_body, error_var, catch_body) => {
                match self.exec_stmt_internal(try_body) {
                    Ok(flow) => Ok(flow),
                    Err(e) => {
                        // Bind the error message into the current scope and run catch body
                        self.environment
                            .define(error_var.clone(), Value::string(e.to_string()));
                        self.exec_stmt_internal(catch_body)
                    }
                }
            }
        }
    }

    /// Helper to evaluate binary operation on values (for compound assignment)
    fn eval_binary_values(
        &mut self,
        left: Value,
        op: BinaryOp,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => self.eval_add(left, right),
            BinaryOp::Subtract => self.eval_arithmetic(left, right, |a, b| a - b, |a, b| a - b),
            BinaryOp::Multiply => self.eval_arithmetic(left, right, |a, b| a * b, |a, b| a * b),
            BinaryOp::Divide => self.eval_divide(left, right),
            _ => Err(RuntimeError::InvalidOperation(
                "Invalid compound assignment operator".to_string(),
            )),
        }
    }

    /// Assign a value to a target (identifier, index, or member)
    fn assign_target(&mut self, target: &Expr, value: Value) -> Result<(), RuntimeError> {
        match target {
            Expr::Identifier(name) => {
                self.environment.set(name, value)?;
                Ok(())
            }
            Expr::Index(array, index) => {
                // Get the array
                let array_val = self.eval_expr(array)?;
                let index_val = self.eval_expr(index)?;

                match (array_val, index_val) {
                    (Value::Array(elements), Value::Int(idx)) => {
                        if idx < 0 || idx as usize >= elements.len() {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: idx,
                                length: elements.len(),
                            });
                        }
                        // Clone the inner Vec, mutate, wrap in new Rc
                        let mut new_elements = (**elements).to_vec();
                        new_elements[idx as usize] = value;

                        // Update the array in environment (only works for simple identifiers)
                        if let Expr::Identifier(name) = &**array {
                            self.environment
                                .set(name, Value::Array(Rc::new(new_elements)))?;
                        }
                        Ok(())
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "array".to_string(),
                        got: "non-array".to_string(),
                    }),
                }
            }
            Expr::Member(obj, member) => {
                let obj_val = self.eval_expr(obj)?;
                match obj_val {
                    Value::Instance { fields, .. } => {
                        fields.borrow_mut().insert(member.clone(), value);
                        Ok(())
                    }
                    other => Err(RuntimeError::InvalidOperation(format!(
                        "Cannot assign field on type '{}'",
                        other.type_name()
                    ))),
                }
            }
            _ => Err(RuntimeError::InvalidOperation(
                "Invalid assignment target".to_string(),
            )),
        }
    }

    /// Execute a program (multiple statements)
    pub fn execute_program(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in statements {
            self.exec_stmt_internal(stmt)?;
        }
        Ok(())
    }

    /// Call the top-level main() function. Returns error if main is not defined or not a function.
    pub fn call_main(&mut self) -> Result<(), RuntimeError> {
        let main_val = self.environment.get("main").map_err(|_| {
            RuntimeError::InvalidOperation(
                "No main() function defined. Every Aether program must have a main() function."
                    .to_string(),
            )
        })?;

        match main_val {
            Value::Function {
                params,
                body,
                closure,
            } => {
                if !params.is_empty() {
                    return Err(RuntimeError::InvalidOperation(
                        "main() must take no arguments".to_string(),
                    ));
                }
                self.call_depth += 1;
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(_)) | Ok(_) => Ok(()),
                    Err(e) => Err(e),
                };
                self.environment = saved_env;
                self.call_depth -= 1;
                result
            }
            _ => Err(RuntimeError::InvalidOperation(
                "main is not a function".to_string(),
            )),
        }
    }

    /// Evaluate function call
    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
        // Check if this is a method call (e.g., arr.push(1))
        if let Expr::Member(object, method) = callee {
            return self.eval_method_call(object, method, args);
        }

        // Remember the function name for recursion support
        let func_name = if let Expr::Identifier(name) = callee {
            Some(name.clone())
        } else {
            None
        };

        // Regular function call
        let func_val = self.eval_expr(callee)?;
        let func_val_clone = func_val.clone(); // Clone for recursion support

        match func_val {
            Value::Function {
                params,
                body,
                closure,
            } => {
                // Check recursion depth
                self.call_depth += 1;
                if self.call_depth > self.max_call_depth {
                    self.call_depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.call_depth + 1,
                        limit: self.max_call_depth,
                    });
                }

                // Check arity - allow fewer arguments (optional parameters)
                if args.len() > params.len() {
                    self.call_depth -= 1;
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    match self.eval_expr(arg) {
                        Ok(val) => arg_values.push(val),
                        Err(e) => {
                            self.call_depth -= 1;
                            return Err(e);
                        }
                    }
                }

                // Pad with null for missing optional parameters
                while arg_values.len() < params.len() {
                    arg_values.push(Value::Null);
                }

                // Save current environment
                let saved_env = self.environment.clone();

                // Create new environment with closure as parent
                self.environment = Environment::with_parent((*closure).clone());

                // If this is a named function, define it in the new environment for recursion
                if let Some(name) = func_name {
                    self.environment.define(name, func_val_clone);
                }

                // Bind parameters
                for (param, value) in params.iter().zip(arg_values) {
                    self.environment.define(param.clone(), value);
                }

                // Execute function body
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => {
                        // Restore environment before returning error
                        self.environment = saved_env;
                        self.call_depth -= 1;
                        return Err(e);
                    }
                };

                // Restore environment
                self.environment = saved_env;
                self.call_depth -= 1;

                result
            }
            Value::BuiltinFn {
                name: _,
                arity,
                func,
            } => {
                // Check arity (unless variadic - represented by usize::MAX)
                if arity != usize::MAX && arity != args.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: arity,
                        got: args.len(),
                    });
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }

                // Call the built-in function
                func(&arg_values)
            }
            _ => Err(RuntimeError::TypeError {
                expected: "function".to_string(),
                got: func_val.type_name().to_string(),
            }),
        }
    }

    // Module loading methods

    /// Load a module and bind it as a namespace object in the environment.
    fn load_module(&mut self, module_name: &str) -> Result<(), RuntimeError> {
        let module_val = self.load_module_as_value(module_name)?;
        self.environment.define(module_name.to_string(), module_val);
        Ok(())
    }

    /// Load a module and bind it under the given alias.
    fn load_module_as(&mut self, module_name: &str, alias: &str) -> Result<(), RuntimeError> {
        let module_val = self.load_module_as_value(module_name)?;
        self.environment.define(alias.to_string(), module_val);
        Ok(())
    }

    /// Load a module and return it as a Value::Module, using cache when available.
    fn load_module_as_value(&mut self, module_name: &str) -> Result<Value, RuntimeError> {
        // Circular dependency check
        if self.loading_stack.contains(&module_name.to_string()) {
            let cycle = self.loading_stack.join(" -> ");
            return Err(RuntimeError::InvalidOperation(format!(
                "Circular dependency detected: {} -> {}",
                cycle, module_name
            )));
        }

        // Return cached module env and build Value::Module from it
        if self.module_cache.contains_key(module_name) {
            let env = self.module_cache[module_name].clone();
            return Ok(Self::module_value_from_env(module_name, &env));
        }

        // Mark as loading
        self.loading_stack.push(module_name.to_string());

        // Resolve path and execute
        let module_path = self.resolve_module_path(module_name)?;
        let module_env = self.execute_module_file(&module_path)?;

        // Unmark loading
        self.loading_stack.retain(|m| m != module_name);

        // Cache the environment
        self.module_cache
            .insert(module_name.to_string(), module_env.clone());

        Ok(Self::module_value_from_env(module_name, &module_env))
    }

    /// Build a Value::Module from a module's environment.
    fn module_value_from_env(name: &str, env: &Environment) -> Value {
        use std::collections::HashMap;
        let members: HashMap<String, Value> = env
            .bindings()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Value::Module {
            name: name.to_string(),
            members: Rc::new(members),
        }
    }

    /// Import specific items from a module
    fn from_import(&mut self, module_name: &str, items: &[String]) -> Result<(), RuntimeError> {
        // Check if already loaded
        if !self.module_cache.contains_key(module_name) {
            // Resolve and load module
            let module_path = self.resolve_module_path(module_name)?;
            let module_env = self.execute_module_file(&module_path)?;
            self.module_cache
                .insert(module_name.to_string(), module_env);
        }

        let module_env = &self.module_cache[module_name];

        // Import only specified items
        for item in items {
            match module_env.get(item) {
                Ok(value) => {
                    self.environment.define(item.clone(), value);
                }
                Err(_) => {
                    return Err(RuntimeError::InvalidOperation(format!(
                        "Module '{}' has no function '{}'",
                        module_name, item
                    )));
                }
            }
        }

        Ok(())
    }

    /// Import specific items from a module with optional aliases.
    fn from_import_as(
        &mut self,
        module_name: &str,
        items: &[(String, String)],
    ) -> Result<(), RuntimeError> {
        if !self.module_cache.contains_key(module_name) {
            let module_path = self.resolve_module_path(module_name)?;
            let module_env = self.execute_module_file(&module_path)?;
            self.module_cache
                .insert(module_name.to_string(), module_env);
        }

        let module_env = &self.module_cache[module_name];

        for (item, alias) in items {
            match module_env.get(item) {
                Ok(value) => {
                    self.environment.define(alias.clone(), value);
                }
                Err(_) => {
                    return Err(RuntimeError::InvalidOperation(format!(
                        "Module '{}' has no member '{}'",
                        module_name, item
                    )));
                }
            }
        }

        Ok(())
    }

    /// Resolve module name to file path
    fn resolve_module_path(&self, module_name: &str) -> Result<PathBuf, RuntimeError> {
        // Try current directory first
        let mut path = PathBuf::from(format!("{}.ae", module_name));
        if path.exists() {
            return Ok(path);
        }

        // Try relative to current file
        if let Some(ref current) = self.current_file {
            if let Some(parent) = current.parent() {
                path = parent.join(format!("{}.ae", module_name));
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Try modules subdirectory
        path = PathBuf::from(format!("modules/{}.ae", module_name));
        if path.exists() {
            return Ok(path);
        }

        Err(RuntimeError::InvalidOperation(format!(
            "Module not found: '{}'",
            module_name
        )))
    }

    /// Execute a module file and return its environment
    fn execute_module_file(&mut self, path: &PathBuf) -> Result<Environment, RuntimeError> {
        use crate::lexer::Scanner;
        use crate::parser::Parser;
        use std::fs;

        // Read module file
        let source = fs::read_to_string(path)
            .map_err(|e| RuntimeError::InvalidOperation(format!("Failed to read module: {}", e)))?;

        // Parse module
        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens().map_err(|e| {
            RuntimeError::InvalidOperation(format!("Failed to tokenize module: {}", e))
        })?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| {
            RuntimeError::InvalidOperation(format!("Failed to parse module: {}", e))
        })?;

        // Save current environment and file
        let saved_env = self.environment.clone();
        let saved_file = self.current_file.clone();

        // Create fresh environment for module
        self.environment = Environment::new();
        self.current_file = Some(path.clone());

        // Execute module
        for stmt in &program.statements {
            if let Err(e) = self.exec_stmt(stmt) {
                // Restore and return error
                self.environment = saved_env;
                self.current_file = saved_file;
                return Err(e);
            }
        }

        // Get module environment
        let module_env = self.environment.clone();

        // Restore original environment and file
        self.environment = saved_env;
        self.current_file = saved_file;

        Ok(module_env)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
