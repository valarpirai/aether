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
            UnaryOp::BitwiseNot => match value {
                Value::Int(n) => Ok(Value::Int(!n)),
                _ => Err(RuntimeError::TypeError {
                    expected: "int".to_string(),
                    got: value.type_name().to_string(),
                }),
            },
        }
    }

    pub(super) fn eval_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left_val = self.eval_expr(left)?;

        // Short-circuit operators — right side evaluated only when needed
        if let BinaryOp::NullCoalesce = op {
            return if matches!(left_val, Value::Null) {
                self.eval_expr(right)
            } else {
                Ok(left_val)
            };
        }

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
            BinaryOp::Equal => Ok(Value::Bool(Self::values_equal(&left_val, &right_val))),
            BinaryOp::NotEqual => Ok(Value::Bool(!Self::values_equal(&left_val, &right_val))),
            BinaryOp::Less => self.eval_comparison(
                left_val,
                right_val,
                |a, b| a < b,
                |a, b| a < b,
                |a, b| a < b,
            ),
            BinaryOp::Greater => self.eval_comparison(
                left_val,
                right_val,
                |a, b| a > b,
                |a, b| a > b,
                |a, b| a > b,
            ),
            BinaryOp::LessEqual => self.eval_comparison(
                left_val,
                right_val,
                |a, b| a <= b,
                |a, b| a <= b,
                |a, b| a <= b,
            ),
            BinaryOp::GreaterEqual => self.eval_comparison(
                left_val,
                right_val,
                |a, b| a >= b,
                |a, b| a >= b,
                |a, b| a >= b,
            ),
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
            BinaryOp::NullCoalesce => unreachable!("handled above"),
            BinaryOp::Power => self.eval_power(left_val, right_val),
            BinaryOp::BitwiseAnd => Self::eval_bitwise(left_val, right_val, |a, b| a & b),
            BinaryOp::BitwiseOr => Self::eval_bitwise(left_val, right_val, |a, b| a | b),
            BinaryOp::BitwiseXor => Self::eval_bitwise(left_val, right_val, |a, b| a ^ b),
            BinaryOp::ShiftLeft => Self::eval_shift(left_val, right_val, "<<", |a, b| a << b),
            BinaryOp::ShiftRight => Self::eval_shift(left_val, right_val, ">>", |a, b| a >> b),
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

    pub(super) fn eval_comparison<F, G, H>(
        &self,
        left: Value,
        right: Value,
        int_op: F,
        float_op: G,
        str_op: H,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(i64, i64) -> bool,
        G: FnOnce(f64, f64) -> bool,
        H: FnOnce(&str, &str) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(int_op(a, b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(float_op(a, b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(float_op(a as f64, b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(float_op(a, b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(str_op(a.as_str(), b.as_str()))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "comparable types".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    pub(super) fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
            (Value::Instance { fields: fa, .. }, Value::Instance { fields: fb, .. }) => {
                Rc::ptr_eq(fa, fb)
            }
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
            ) => {
                ta == tb
                    && fa.len() == fb.len()
                    && fa
                        .iter()
                        .zip(fb.iter())
                        .all(|((_, va), (_, vb))| Self::values_equal(va, vb))
            }
            _ => false,
        }
    }

    /// Structural deep equality used by `.equals()`.
    /// Arrays and dicts recurse; nested structs stop at == (identity).
    pub(super) fn deep_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Array(a), Value::Array(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let a = a.borrow();
                let b = b.borrow();
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::deep_equal(x, y))
            }
            (Value::Dict(a), Value::Dict(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let a = a.borrow();
                let b = b.borrow();
                a.len() == b.len()
                    && a.iter().zip(b.iter()).all(|((k1, v1), (k2, v2))| {
                        Self::deep_equal(k1, k2) && Self::deep_equal(v1, v2)
                    })
            }
            // Nested structs use identity (== semantics, depth-1 stop)
            (Value::Instance { fields: fa, .. }, Value::Instance { fields: fb, .. }) => {
                Rc::ptr_eq(fa, fb)
            }
            _ => Self::values_equal(left, right),
        }
    }

    /// Structural equality for structs: same type + fields compared with deep_equal.
    pub(super) fn struct_equals(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (
                Value::Instance {
                    type_name: ta,
                    fields: fa,
                    ..
                },
                Value::Instance {
                    type_name: tb,
                    fields: fb,
                    ..
                },
            ) => {
                if ta != tb {
                    return false;
                }
                if Rc::ptr_eq(fa, fb) {
                    return true;
                }
                let fa = fa.borrow();
                let fb = fb.borrow();
                if fa.len() != fb.len() {
                    return false;
                }
                fa.iter()
                    .all(|(k, v)| fb.get(k).is_some_and(|ov| Self::values_equal(v, ov)))
            }
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

    fn eval_power(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) if b >= 0 => Ok(Value::Int(a.wrapping_pow(b as u32))),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Float((a as f64).powi(b as i32))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powi(b as i32))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn eval_bitwise<F>(left: Value, right: Value, op: F) -> Result<Value, RuntimeError>
    where
        F: FnOnce(i64, i64) -> i64,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(op(a, b))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "int".to_string(),
                got: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn eval_shift<F>(left: Value, right: Value, op_name: &str, op: F) -> Result<Value, RuntimeError>
    where
        F: FnOnce(i64, u32) -> i64,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) if (0..64).contains(&b) => {
                Ok(Value::Int(op(a, b as u32)))
            }
            (Value::Int(_), Value::Int(b)) => Err(RuntimeError::InvalidOperation(format!(
                "shift amount {} out of range (0..63)",
                b
            ))),
            (left, right) => Err(RuntimeError::TypeError {
                expected: "int".to_string(),
                got: format!(
                    "{} requires int operands, got {} and {}",
                    op_name,
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}
