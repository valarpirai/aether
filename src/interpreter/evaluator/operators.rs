use crate::interpreter::environment::RuntimeError;
use crate::interpreter::value::Value;
use crate::parser::ast::{BinaryOp, Expr, UnaryOp};
use std::rc::Rc;

use super::Evaluator;

impl Evaluator {
    pub(super) fn eval_unary(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
    ) -> Result<Value, RuntimeError> {
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

    pub(super) fn eval_binary(
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

    pub(super) fn eval_add(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(Rc::new(format!("{}{}", a, b))))
            }
            (Value::String(a), right) => Ok(Value::String(Rc::new(format!("{}{}", a, right)))),
            (left, Value::String(b)) => Ok(Value::String(Rc::new(format!("{}{}", left, b)))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "number or string".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    pub(super) fn eval_arithmetic<F, G>(
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

    pub(super) fn eval_divide(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
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

    pub(super) fn eval_modulo(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
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

    pub(super) fn eval_comparison<F, G>(
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
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "comparable types".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    pub(super) fn values_equal(&self, left: &Value, right: &Value) -> bool {
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

    pub(super) fn eval_binary_values(
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
}
