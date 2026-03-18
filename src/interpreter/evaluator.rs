//! Expression evaluation for the Aether interpreter

use super::environment::{Environment, RuntimeError};
use super::value::Value;
use crate::parser::ast::{BinaryOp, Expr, UnaryOp};

/// Interpreter for evaluating expressions
pub struct Evaluator {
    /// Current environment
    pub environment: Environment,
}

impl Evaluator {
    /// Create a new evaluator with a fresh environment
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    /// Evaluate an expression
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Integer(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Identifier(name) => self.environment.get(name),
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Unary(op, operand) => self.eval_unary(*op, operand),
            Expr::Binary(left, op, right) => self.eval_binary(left, *op, right),
            Expr::Index(array, index) => self.eval_index(array, index),
            Expr::Member(_object, _member) => {
                // Member access not yet implemented
                Err(RuntimeError::InvalidOperation(
                    "Member access not yet implemented".to_string(),
                ))
            }
            Expr::Call(_callee, _args) => {
                // Function calls not yet implemented
                Err(RuntimeError::InvalidOperation(
                    "Function calls not yet implemented".to_string(),
                ))
            }
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
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
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
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
