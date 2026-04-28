use crate::interpreter::environment::{Environment, RuntimeError};
use crate::interpreter::value::Value;
use crate::parser::ast::Expr;
use std::rc::Rc;

use super::{ControlFlow, Evaluator};

impl Evaluator {
    /// Call a Value with already-evaluated arguments.
    pub(super) fn call_value(&mut self, func: Value, arg_values: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Function { params, body, closure } => {
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
                // Swap instead of clone — O(1) vs O(n) environment copy
                let mut call_env = Environment::with_parent((*closure).clone());
                for (param, value) in params.iter().zip(padded) {
                    call_env.define(param.clone(), value);
                }
                std::mem::swap(&mut self.environment, &mut call_env);
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => Err(e),
                };
                std::mem::swap(&mut self.environment, &mut call_env);
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
            Value::AsyncFunction { params, body, closure } => {
                if arg_values.len() > params.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: arg_values.len(),
                    });
                }
                let mut padded = arg_values;
                while padded.len() < params.len() {
                    padded.push(Value::Null);
                }
                Ok(Value::promise(Value::AsyncFunction { params, body, closure }, padded))
            }
            other => Err(RuntimeError::InvalidOperation(format!(
                "Cannot call value of type '{}'",
                other.type_name()
            ))),
        }
    }

    /// Execute an async function body directly (used by Expr::Await to resolve Promises).
    /// Unlike call_value, this never wraps AsyncFunction in another Promise.
    pub(super) fn exec_async_body(&mut self, func: Value, arg_values: Vec<Value>) -> Result<Value, RuntimeError> {
        let (params, body, closure) = match func {
            Value::AsyncFunction { params, body, closure } => (params, body, closure),
            Value::Function { params, body, closure } => (params, body, closure),
            Value::BuiltinFn { arity, func, .. } => {
                if arity != usize::MAX && arity != arg_values.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: arity,
                        got: arg_values.len(),
                    });
                }
                return func(&arg_values);
            }
            other => return Err(RuntimeError::InvalidOperation(format!(
                "Cannot await value of type '{}'", other.type_name()
            ))),
        };

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
        let mut call_env = Environment::with_parent((*closure).clone());
        for (param, value) in params.iter().zip(padded) {
            call_env.define(param.clone(), value);
        }
        std::mem::swap(&mut self.environment, &mut call_env);
        let result = match self.exec_stmt_internal(&body) {
            Ok(ControlFlow::Return(val)) => Ok(val),
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(e),
        };
        std::mem::swap(&mut self.environment, &mut call_env);
        self.call_depth -= 1;
        result
    }

    pub(super) fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
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

        let func_val = self.eval_expr(callee)?;
        let func_val_clone = func_val.clone();

        match func_val {
            Value::Function { params, body, closure } => {
                self.call_depth += 1;
                if self.call_depth > self.max_call_depth {
                    self.call_depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.call_depth + 1,
                        limit: self.max_call_depth,
                    });
                }

                if args.len() > params.len() {
                    self.call_depth -= 1;
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }

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

                while arg_values.len() < params.len() {
                    arg_values.push(Value::Null);
                }

                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());

                // Define function in its own scope for recursion
                if let Some(name) = func_name {
                    self.environment.define(name, func_val_clone);
                }

                for (param, value) in params.iter().zip(arg_values) {
                    self.environment.define(param.clone(), value);
                }

                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => {
                        self.environment = saved_env;
                        self.call_depth -= 1;
                        return Err(e);
                    }
                };

                self.environment = saved_env;
                self.call_depth -= 1;

                result
            }
            Value::BuiltinFn { name: _, arity, func } => {
                if arity != usize::MAX && arity != args.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: arity,
                        got: args.len(),
                    });
                }

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }

                func(&arg_values)
            }
            Value::AsyncFunction { params, body, closure } => {
                if args.len() > params.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                while arg_values.len() < params.len() {
                    arg_values.push(Value::Null);
                }
                Ok(Value::promise(Value::AsyncFunction { params, body, closure }, arg_values))
            }
            _ => Err(RuntimeError::TypeError {
                expected: "function".to_string(),
                got: func_val.type_name().to_string(),
            }),
        }
    }

    pub(super) fn assign_target(&mut self, target: &Expr, value: Value) -> Result<(), RuntimeError> {
        match target {
            Expr::Identifier(name) => {
                self.environment.set(name, value)?;
                Ok(())
            }
            Expr::Index(array, index) => {
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
                        let mut new_elements = (**elements).to_vec();
                        new_elements[idx as usize] = value;

                        if let Expr::Identifier(name) = &**array {
                            self.environment
                                .set(name, Value::Array(Rc::new(new_elements)))?;
                        }
                        Ok(())
                    }
                    (Value::Dict(pairs), key) => {
                        let mut new_pairs = (*pairs).to_vec();
                        let mut found = false;
                        for (k, v) in new_pairs.iter_mut() {
                            if k == &key {
                                *v = value.clone();
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            new_pairs.push((key, value));
                        }
                        if let Expr::Identifier(name) = &**array {
                            self.environment
                                .set(name, Value::Dict(Rc::new(new_pairs)))?;
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
}
