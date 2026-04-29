use crate::interpreter::environment::RuntimeError;
use crate::interpreter::io_pool::IoResult;
use crate::interpreter::value::{PromiseState, Value};
use crate::parser::ast::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Evaluator;

impl Evaluator {
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
                                return Err(RuntimeError::InvalidSpread {
                                    got: other.type_name().to_string(),
                                })
                            }
                        }
                    } else {
                        values.push(self.eval_expr(elem)?);
                    }
                }
                Ok(Value::Array(Rc::new(values)))
            }
            Expr::FunctionExpr(params, body) => Ok(Value::Function {
                params: params.clone(),
                body: Rc::clone(body),
                closure: Rc::new(self.environment.clone()),
            }),
            Expr::AsyncFunctionExpr(params, body) => Ok(Value::AsyncFunction {
                params: params.clone(),
                body: Rc::clone(body),
                closure: Rc::new(self.environment.clone()),
            }),
            Expr::Await(inner) => {
                let val = self.eval_expr(inner)?;
                self.await_value(val)
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
            Expr::OptionalMember(obj, member) => {
                let val = self.eval_expr(obj)?;
                if matches!(val, Value::Null) {
                    Ok(Value::Null)
                } else {
                    self.eval_member(obj, member)
                }
            }
            Expr::OptionalCall(obj, method, args) => {
                let val = self.eval_expr(obj)?;
                if matches!(val, Value::Null) {
                    Ok(Value::Null)
                } else {
                    self.eval_method_call(obj, method, args)
                }
            }
            Expr::Spread(_) => Err(RuntimeError::InvalidOperation(
                "spread operator is only valid inside array literals".to_string(),
            )),
            Expr::StructInit { name, fields } => {
                let struct_def = self.environment.get(name)?;
                match struct_def {
                    Value::StructDef {
                        fields: def_fields,
                        methods,
                        ..
                    } => {
                        let mut field_map: HashMap<String, Value> = def_fields
                            .iter()
                            .map(|f| (f.clone(), Value::Null))
                            .collect();
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

    pub(super) fn eval_index(&mut self, array: &Expr, index: &Expr) -> Result<Value, RuntimeError> {
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
                let chars: Vec<char> = s.chars().collect();
                if idx < 0 || idx as usize >= chars.len() {
                    Err(RuntimeError::IndexOutOfBounds {
                        index: idx,
                        length: chars.len(),
                    })
                } else {
                    Ok(Value::string(chars[idx as usize].to_string()))
                }
            }
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

    pub(super) fn eval_slice(
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

    /// Core await logic: resolves a Promise or returns non-Promise values unchanged.
    /// Used by Expr::Await and Promise.all.
    pub(super) fn await_value(&mut self, val: Value) -> Result<Value, RuntimeError> {
        match val {
            Value::Promise(state_rc) => {
                // Move state out before any self.* calls to avoid borrow conflicts
                let pending = {
                    let mut state = state_rc.borrow_mut();
                    match &*state {
                        PromiseState::Resolved(v) => return Ok(v.clone()),
                        PromiseState::Pending { .. } | PromiseState::IoWaiting(_) => {}
                    }
                    std::mem::replace(&mut *state, PromiseState::Resolved(Value::Null))
                }; // borrow_mut guard dropped here
                match pending {
                    PromiseState::Pending { func, args } => {
                        // Execute the async function body directly (not via call_value,
                        // which would wrap it in another Promise)
                        let result = self.exec_async_body(func, args)?;
                        *state_rc.borrow_mut() = PromiseState::Resolved(result.clone());
                        Ok(result)
                    }
                    PromiseState::IoWaiting(rx) => {
                        // Block main thread until I/O worker completes
                        let io_result = rx.recv().map_err(|_| RuntimeError::ChannelClosed)?;
                        let value = match io_result {
                            IoResult::Str(Ok(s)) => Value::string(s),
                            IoResult::Str(Err(e)) => {
                                return Err(RuntimeError::IoError {
                                    operation: "async I/O".to_string(),
                                    detail: e,
                                })
                            }
                            IoResult::Unit(Ok(())) => Value::Null,
                            IoResult::Unit(Err(e)) => {
                                return Err(RuntimeError::IoError {
                                    operation: "async I/O".to_string(),
                                    detail: e,
                                })
                            }
                        };
                        *state_rc.borrow_mut() = PromiseState::Resolved(value.clone());
                        Ok(value)
                    }
                    PromiseState::Resolved(v) => Ok(v),
                }
            }
            other => Ok(other), // await non-Promise is identity
        }
    }
}
