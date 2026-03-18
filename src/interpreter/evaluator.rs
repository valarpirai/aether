//! Expression evaluation and statement execution for the Aether interpreter

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
}

impl Evaluator {
    /// Create a new evaluator with a fresh environment
    pub fn new() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
        };
        evaluator.register_builtins();
        evaluator.load_stdlib();
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
    }

    /// Load standard library modules
    fn load_stdlib(&mut self) {
        use crate::lexer::Scanner;
        use crate::parser::Parser;
        use super::stdlib;

        for (name, source) in stdlib::stdlib_modules() {
            // Parse the module
            let mut scanner = Scanner::new(source);
            let tokens = match scanner.scan_tokens() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Warning: Failed to tokenize stdlib module '{}': {}", name, e);
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

            // Execute all statements in the module
            for stmt in &program.statements {
                if let Err(e) = self.exec_stmt(stmt) {
                    eprintln!("Warning: Failed to execute stdlib module '{}': {}", name, e);
                    break;
                }
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
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(Rc::new(values)))
            }
            Expr::Unary(op, operand) => self.eval_unary(*op, operand),
            Expr::Binary(left, op, right) => self.eval_binary(left, *op, right),
            Expr::Index(array, index) => self.eval_index(array, index),
            Expr::Member(object, member) => self.eval_member(object, member),
            Expr::Call(callee, args) => self.eval_call(callee, args),
            Expr::Dict(_) => {
                // Dictionaries not yet implemented
                Err(RuntimeError::InvalidOperation(
                    "Dictionaries not yet implemented".to_string(),
                ))
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
            BinaryOp::Subtract => self.eval_arithmetic(left_val, right_val, |a, b| a - b, |a, b| a - b),
            BinaryOp::Multiply => self.eval_arithmetic(left_val, right_val, |a, b| a * b, |a, b| a * b),
            BinaryOp::Divide => self.eval_divide(left_val, right_val),
            BinaryOp::Modulo => self.eval_modulo(left_val, right_val),
            BinaryOp::Equal => Ok(Value::Bool(self.values_equal(&left_val, &right_val))),
            BinaryOp::NotEqual => Ok(Value::Bool(!self.values_equal(&left_val, &right_val))),
            BinaryOp::Less => self.eval_comparison(left_val, right_val, |a, b| a < b, |a, b| a < b),
            BinaryOp::Greater => self.eval_comparison(left_val, right_val, |a, b| a > b, |a, b| a > b),
            BinaryOp::LessEqual => self.eval_comparison(left_val, right_val, |a, b| a <= b, |a, b| a <= b),
            BinaryOp::GreaterEqual => self.eval_comparison(left_val, right_val, |a, b| a >= b, |a, b| a >= b),
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
            (Value::String(a), Value::String(b)) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
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

    /// Evaluate array indexing
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
            (array, index) => Err(RuntimeError::TypeError {
                expected: "array and integer index".to_string(),
                got: format!("{} and {}", array.type_name(), index.type_name()),
            }),
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

            // Undefined property
            (obj, prop) => Err(RuntimeError::InvalidOperation(
                format!("Property '{}' does not exist on type '{}'", prop, obj.type_name()),
            )),
        }
    }

    /// Evaluate method call (obj.method(args))
    fn eval_method_call(&mut self, object: &Expr, method: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
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
                    self.environment.set(name, Value::Array(Rc::new(new_elements)))?;
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
                    self.environment.set(name, Value::Array(Rc::new(new_elements)))?;
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
                        s.split(delim.as_str()).map(|part| Value::String(Rc::new(part.to_string()))).collect()
                    };
                    Ok(Value::Array(Rc::new(parts)))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "string".to_string(),
                        got: delimiter.type_name().to_string(),
                    })
                }
            }

            // Undefined method
            (obj, meth) => Err(RuntimeError::InvalidOperation(
                format!("Method '{}' does not exist on type '{}'", meth, obj.type_name()),
            )),
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
                // Create function value with current environment as closure
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure: Box::new(self.environment.clone()),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
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
                            self.environment.set(name, Value::Array(Rc::new(new_elements)))?;
                        }
                        Ok(())
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "array".to_string(),
                        got: "non-array".to_string(),
                    }),
                }
            }
            Expr::Member(_obj, _member) => {
                Err(RuntimeError::InvalidOperation(
                    "Member assignment not yet implemented".to_string(),
                ))
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

    /// Evaluate function call
    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
        // Check if this is a method call (e.g., arr.push(1))
        if let Expr::Member(object, method) = callee {
            return self.eval_method_call(object, method, args);
        }

        // Regular function call
        let func_val = self.eval_expr(callee)?;

        match func_val {
            Value::Function { params, body, closure } => {
                // Check arity - allow fewer arguments (optional parameters)
                if args.len() > params.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }

                // Pad with null for missing optional parameters
                while arg_values.len() < params.len() {
                    arg_values.push(Value::Null);
                }

                // Save current environment
                let saved_env = self.environment.clone();

                // Create new environment with closure as parent
                self.environment = Environment::with_parent((*closure).clone());

                // Bind parameters
                for (param, value) in params.iter().zip(arg_values) {
                    self.environment.define(param.clone(), value);
                }

                // Execute function body
                let result = match self.exec_stmt_internal(&body)? {
                    ControlFlow::Return(val) => val,
                    _ => Value::Null,
                };

                // Restore environment
                self.environment = saved_env;

                Ok(result)
            }
            Value::BuiltinFn { name: _, arity, func } => {
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
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
