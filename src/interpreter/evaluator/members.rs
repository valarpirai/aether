use crate::interpreter::environment::{Environment, RuntimeError};
use crate::interpreter::value::{IteratorSource, Value};
use crate::parser::ast::Expr;
use std::collections::HashSet;
use std::rc::Rc;

use super::{ControlFlow, Evaluator};

impl Evaluator {
    pub(super) fn eval_member(&mut self, object: &Expr, member: &str) -> Result<Value, RuntimeError> {
        let obj_val = self.eval_expr(object)?;

        match (&obj_val, member) {
            (Value::Array(elements), "length") => Ok(Value::Int(elements.len() as i64)),
            (Value::String(s), "length") => Ok(Value::Int(s.len() as i64)),
            (Value::Set(elements), "size") => Ok(Value::Int(elements.len() as i64)),

            (Value::Dict(pairs), key) => {
                if key == "length" || key == "size" {
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

            (Value::Module { name, members }, prop) => {
                members.get(prop).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Module '{}' has no member '{}'",
                        name, prop
                    ))
                })
            }

            (Value::Instance { type_name, fields, .. }, prop) => {
                let map = fields.borrow();
                map.get(prop).cloned().ok_or_else(|| {
                    RuntimeError::InvalidOperation(format!(
                        "Field '{}' does not exist on '{}'",
                        prop, type_name
                    ))
                })
            }

            (obj, prop) => Err(RuntimeError::InvalidOperation(format!(
                "Property '{}' does not exist on type '{}'",
                prop,
                obj.type_name()
            ))),
        }
    }

    pub(super) fn eval_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, RuntimeError> {
        let obj_val = self.eval_expr(object)?;

        match (&obj_val, method) {
            // Array methods
            (Value::Array(elements), "push") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let item = self.eval_expr(&args[0])?;

                let mut new_elements = Rc::clone(elements);
                Rc::make_mut(&mut new_elements).push(item);
                let new_array = Value::Array(new_elements);

                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_array)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_array);
                    }
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

                let mut new_elements = Rc::clone(elements);
                let popped = Rc::make_mut(&mut new_elements).pop();
                let new_array = Value::Array(new_elements);
                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_array)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_array);
                    }
                }

                Ok(popped.unwrap_or(Value::Null))
            }
            (Value::Array(elements), "contains") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let needle = self.eval_expr(&args[0])?;
                let found = elements.iter().any(|elem| self.values_equal(elem, &needle));
                Ok(Value::Bool(found))
            }
            (Value::Array(elements), "sort") => {
                if args.len() > 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let mut new_elements = (**elements).to_vec();
                if args.is_empty() {
                    let sort_err: Option<RuntimeError> = None;
                    new_elements.sort_by(|a, b| {
                        if sort_err.is_some() {
                            return std::cmp::Ordering::Equal;
                        }
                        match (a, b) {
                            (Value::Int(x), Value::Int(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => {
                                x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::Int(x), Value::Float(y)) => {
                                (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::Float(x), Value::Int(y)) => {
                                x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::String(x), Value::String(y)) => x.cmp(y),
                            _ => a.type_name().cmp(b.type_name()),
                        }
                    });
                    if let Some(e) = sort_err {
                        return Err(e);
                    }
                } else {
                    let comparator = self.eval_expr(&args[0])?;
                    let mut sort_err: Option<RuntimeError> = None;
                    new_elements.sort_by(|a, b| {
                        if sort_err.is_some() {
                            return std::cmp::Ordering::Equal;
                        }
                        match self.call_value(comparator.clone(), vec![a.clone(), b.clone()]) {
                            Ok(Value::Int(n)) => {
                                if n < 0 {
                                    std::cmp::Ordering::Less
                                } else if n > 0 {
                                    std::cmp::Ordering::Greater
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            }
                            Ok(other) => {
                                sort_err = Some(RuntimeError::TypeError {
                                    expected: "int".to_string(),
                                    got: other.type_name().to_string(),
                                });
                                std::cmp::Ordering::Equal
                            }
                            Err(e) => {
                                sort_err = Some(e);
                                std::cmp::Ordering::Equal
                            }
                        }
                    });
                    if let Some(e) = sort_err {
                        return Err(e);
                    }
                }
                let new_array = Value::Array(Rc::new(new_elements));
                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_array)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_array);
                    }
                }
                Ok(Value::Null)
            }
            (Value::Array(elements), "concat") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let other_val = self.eval_expr(&args[0])?;
                match other_val {
                    Value::Array(other_elements) => {
                        let mut result = (**elements).to_vec();
                        result.extend_from_slice(&other_elements);
                        Ok(Value::Array(Rc::new(result)))
                    }
                    other => Err(RuntimeError::TypeError {
                        expected: "array".to_string(),
                        got: other.type_name().to_string(),
                    }),
                }
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

            // Set methods
            // Note: clippy::mutable_key_type warnings are false positives - see value.rs
            (Value::Set(elements), "add") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let item = self.eval_expr(&args[0])?;
                if !item.is_hashable() {
                    return Err(RuntimeError::TypeError {
                        expected: "hashable type (int, float, string, bool, null)".to_string(),
                        got: format!("{} (not hashable)", item.type_name()),
                    });
                }

                #[allow(clippy::mutable_key_type)]
                let mut new_set = (**elements).clone();
                new_set.insert(item);
                let new_set_val = Value::set(new_set);

                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_set_val)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_set_val);
                    }
                }

                Ok(Value::Null)
            }
            (Value::Set(elements), "remove") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let item = self.eval_expr(&args[0])?;

                #[allow(clippy::mutable_key_type)]
                let mut new_set = (**elements).clone();
                new_set.remove(&item);
                let new_set_val = Value::set(new_set);

                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_set_val)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_set_val);
                    }
                }

                Ok(Value::Null)
            }
            (Value::Set(elements), "contains") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let needle = self.eval_expr(&args[0])?;
                Ok(Value::Bool(elements.contains(&needle)))
            }
            (Value::Set(_elements), "clear") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }

                let new_set_val = Value::set(HashSet::new());

                if let Expr::Identifier(name) = object {
                    self.environment.set(name, new_set_val)?;
                } else if let Expr::Member(obj_expr, field) = object {
                    let owner = self.eval_expr(obj_expr)?;
                    if let Value::Instance { fields, .. } = owner {
                        fields.borrow_mut().insert(field.clone(), new_set_val);
                    }
                }

                Ok(Value::Null)
            }
            (Value::Set(elements), "to_array") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                let mut vec: Vec<Value> = elements.iter().cloned().collect();
                vec.sort_by_key(|v| format!("{}", v));
                Ok(Value::array(vec))
            }
            (Value::Set(elements), "union") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let other = self.eval_expr(&args[0])?;
                if let Value::Set(other_set) = other {
                    #[allow(clippy::mutable_key_type)]
                    let union: HashSet<Value> = elements.union(&other_set).cloned().collect();
                    Ok(Value::set(union))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "set".to_string(),
                        got: other.type_name().to_string(),
                    })
                }
            }
            (Value::Set(elements), "intersection") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let other = self.eval_expr(&args[0])?;
                if let Value::Set(other_set) = other {
                    #[allow(clippy::mutable_key_type)]
                    let intersection: HashSet<Value> =
                        elements.intersection(&other_set).cloned().collect();
                    Ok(Value::set(intersection))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "set".to_string(),
                        got: other.type_name().to_string(),
                    })
                }
            }
            (Value::Set(elements), "difference") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let other = self.eval_expr(&args[0])?;
                if let Value::Set(other_set) = other {
                    #[allow(clippy::mutable_key_type)]
                    let diff: HashSet<Value> =
                        elements.difference(&other_set).cloned().collect();
                    Ok(Value::set(diff))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "set".to_string(),
                        got: other.type_name().to_string(),
                    })
                }
            }
            (Value::Set(elements), "is_subset") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let other = self.eval_expr(&args[0])?;
                if let Value::Set(other_set) = other {
                    Ok(Value::Bool(elements.is_subset(&other_set)))
                } else {
                    Err(RuntimeError::TypeError {
                        expected: "set".to_string(),
                        got: other.type_name().to_string(),
                    })
                }
            }

            // Dict methods
            (Value::Dict(pairs), "keys") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                let keys: Vec<Value> = pairs.iter().map(|(k, _)| k.clone()).collect();
                Ok(Value::array(keys))
            }
            (Value::Dict(pairs), "values") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 0,
                        got: args.len(),
                    });
                }
                let values: Vec<Value> = pairs.iter().map(|(_, v)| v.clone()).collect();
                Ok(Value::array(values))
            }
            (Value::Dict(pairs), "contains") => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let key = self.eval_expr(&args[0])?;
                let found = pairs.iter().any(|(k, _)| k == &key);
                Ok(Value::Bool(found))
            }

            // Promise.all([p1, p2, ...]) — await all promises and return array of results
            (Value::Module { name, .. }, "all") if name.as_str() == "Promise" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let array_val = self.eval_expr(&args[0])?;
                match array_val {
                    Value::Array(promises) => {
                        let promises_vec: Vec<Value> = promises.iter().cloned().collect();
                        let mut results = Vec::new();
                        for p in promises_vec {
                            results.push(self.await_value(p)?);
                        }
                        Ok(Value::array(results))
                    }
                    other => Err(RuntimeError::TypeError {
                        expected: "array of promises".to_string(),
                        got: other.type_name().to_string(),
                    }),
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
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                self.call_value(func, arg_values)
            }

            // Instance method call: instance.method(args)
            (Value::Instance { type_name, fields, methods }, meth) => {
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
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                self.call_depth += 1;
                if self.call_depth > self.max_call_depth {
                    self.call_depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.call_depth + 1,
                        limit: self.max_call_depth,
                    });
                }
                let mut call_env = Environment::new();
                std::mem::swap(&mut self.environment, &mut call_env);
                self.environment = Environment::with_parent(call_env);
                self.environment.define("self".to_string(), instance);
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
                let parent = self.environment.take_parent().unwrap_or_default();
                self.environment = parent;
                self.call_depth -= 1;
                result
            }

            // Iterator methods
            (Value::Iterator(state), "next") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch { expected: 0, got: args.len() });
                }
                let mut st = state.borrow_mut();
                let result = match &st.source {
                    IteratorSource::Array(arr) => {
                        if st.index < arr.len() {
                            let val = arr[st.index].clone();
                            st.index += 1;
                            val
                        } else {
                            Value::Null
                        }
                    }
                    IteratorSource::DictKeys(pairs) => {
                        if st.index < pairs.len() {
                            let key = pairs[st.index].0.clone();
                            st.index += 1;
                            key
                        } else {
                            Value::Null
                        }
                    }
                    IteratorSource::Set(items) => {
                        if st.index < items.len() {
                            let val = items[st.index].clone();
                            st.index += 1;
                            val
                        } else {
                            Value::Null
                        }
                    }
                };
                Ok(result)
            }
            (Value::Iterator(state), "has_next") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch { expected: 0, got: args.len() });
                }
                let st = state.borrow();
                let has = match &st.source {
                    IteratorSource::Array(arr) => st.index < arr.len(),
                    IteratorSource::DictKeys(pairs) => st.index < pairs.len(),
                    IteratorSource::Set(items) => st.index < items.len(),
                };
                Ok(Value::Bool(has))
            }

            // iterator() factory methods on collections
            (Value::Array(elements), "iterator") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch { expected: 0, got: args.len() });
                }
                Ok(Value::iterator(IteratorSource::Array(Rc::clone(elements))))
            }
            (Value::Dict(pairs), "iterator") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch { expected: 0, got: args.len() });
                }
                Ok(Value::iterator(IteratorSource::DictKeys(Rc::clone(pairs))))
            }
            (Value::Set(elements), "iterator") => {
                if !args.is_empty() {
                    return Err(RuntimeError::ArityMismatch { expected: 0, got: args.len() });
                }
                let items: Vec<Value> = elements.iter().cloned().collect();
                Ok(Value::iterator(IteratorSource::Set(items)))
            }

            // Undefined method
            (obj, meth) => Err(RuntimeError::InvalidOperation(format!(
                "Method '{}' does not exist on type '{}'",
                meth,
                obj.type_name()
            ))),
        }
    }
}
