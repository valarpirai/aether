use crate::interpreter::builtins::parse_http_opts;
use crate::interpreter::environment::{Environment, RuntimeError, StackFrame};
use crate::interpreter::io_pool::{HttpOptions, IoPool, IoTask};
use crate::interpreter::value::{PromiseState, Value};
use crate::parser::ast::Expr;
use std::rc::Rc;
use std::sync::Arc;

use super::{ControlFlow, Evaluator};

impl Evaluator {
    /// Call a Value with already-evaluated arguments.
    pub(super) fn call_value(
        &mut self,
        func: Value,
        arg_values: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
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
                self.call_stack.push(StackFrame {
                    fn_name: "<anonymous>".to_string(),
                    call_site_line: self.current_line,
                    call_site_file: self.current_file_name(),
                });
                // Swap instead of clone — O(1) vs O(n) environment copy
                let mut call_env = Environment::with_parent((*closure).clone());
                for (param, value) in params.iter().zip(padded) {
                    call_env.define(param.clone(), value);
                }
                std::mem::swap(&mut self.environment, &mut call_env);
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => {
                        std::mem::swap(&mut self.environment, &mut call_env);
                        // Don't pop call_stack on error — TryCatch captures the snapshot first
                        self.call_depth -= 1;
                        return Err(e);
                    }
                };
                std::mem::swap(&mut self.environment, &mut call_env);
                self.call_stack.pop();
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
            Value::AsyncFunction {
                params,
                body,
                closure,
            } => {
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
                Ok(Value::promise(
                    Value::AsyncFunction {
                        params,
                        body,
                        closure,
                    },
                    padded,
                ))
            }
            other => Err(RuntimeError::InvalidOperation(format!(
                "Cannot call value of type '{}'",
                other.type_name()
            ))),
        }
    }

    /// Execute an async function body directly (used by Expr::Await to resolve Promises).
    /// Unlike call_value, this never wraps AsyncFunction in another Promise.
    pub(super) fn exec_async_body(
        &mut self,
        func: Value,
        arg_values: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let (params, body, closure) = match func {
            Value::AsyncFunction {
                params,
                body,
                closure,
            } => (params, body, closure),
            Value::Function {
                params,
                body,
                closure,
            } => (params, body, closure),
            Value::BuiltinFn { arity, func, .. } => {
                if arity != usize::MAX && arity != arg_values.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: arity,
                        got: arg_values.len(),
                    });
                }
                return func(&arg_values);
            }
            other => {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Cannot await value of type '{}'",
                    other.type_name()
                )))
            }
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

    pub(super) fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<Value, RuntimeError> {
        // Check if this is a method call (e.g., arr.push(1))
        if let Expr::Member(object, method) = callee {
            return self.eval_method_call(object, method, args);
        }

        // Remember the function name for recursion support and stack traces
        let func_name = if let Expr::Identifier(name) = callee {
            name.clone()
        } else {
            "<anonymous>".to_string()
        };

        let func_val = self.eval_expr(callee)?;
        let func_val_clone = func_val.clone();

        match func_val {
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

                self.call_stack.push(StackFrame {
                    fn_name: func_name.clone(),
                    call_site_line: self.current_line,
                    call_site_file: self.current_file_name(),
                });
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());

                // Define function in its own scope for recursion
                if func_name != "<anonymous>" {
                    self.environment.define(func_name, func_val_clone);
                }

                for (param, value) in params.iter().zip(arg_values) {
                    self.environment.define(param.clone(), value);
                }

                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(val)) => Ok(val),
                    Ok(_) => Ok(Value::Null),
                    Err(e) => {
                        self.environment = saved_env;
                        // Don't pop call_stack on error — TryCatch captures the snapshot first
                        self.call_depth -= 1;
                        return Err(e);
                    }
                };

                self.environment = saved_env;
                self.call_stack.pop();
                self.call_depth -= 1;

                result
            }
            Value::BuiltinFn { name, arity, func } => {
                // set_workers(n) — replaces the I/O thread pool at runtime
                if name == "set_workers" {
                    if args.len() != 1 {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 1,
                            got: args.len(),
                        });
                    }
                    let n_val = self.eval_expr(&args[0])?;
                    let n = match n_val {
                        Value::Int(n) if n > 0 => n as usize,
                        Value::Int(_) => {
                            return Err(RuntimeError::InvalidOperation(
                                "set_workers requires a positive integer".to_string(),
                            ))
                        }
                        other => {
                            return Err(RuntimeError::TypeError {
                                expected: "positive int".to_string(),
                                got: other.type_name().to_string(),
                            })
                        }
                    };
                    self.io_pool = Some(Arc::new(IoPool::new(n)));
                    return Ok(Value::Null);
                }

                // on_ready(promise, callback) — register callback in event loop queue
                if name == "on_ready" {
                    if args.len() != 2 {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 2,
                            got: args.len(),
                        });
                    }
                    let promise_val = self.eval_expr(&args[0])?;
                    let callback = self.eval_expr(&args[1])?;
                    return self.register_on_ready(promise_val, callback);
                }

                // event_loop(?timeout_secs) — run until queue empty or timeout
                if name == "event_loop" {
                    if args.len() > 1 {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 1,
                            got: args.len(),
                        });
                    }
                    // Explicit arg overrides env var; no arg uses env var default (or None)
                    let timeout = if args.is_empty() {
                        self.event_loop_timeout
                    } else {
                        match self.eval_expr(&args[0])? {
                            Value::Int(n) => Some(n as f64),
                            Value::Float(f) => Some(f),
                            other => {
                                return Err(RuntimeError::TypeError {
                                    expected: "number".to_string(),
                                    got: other.type_name().to_string(),
                                })
                            }
                        }
                    };
                    return self.run_event_loop(timeout);
                }

                // set_queue_limit(n) — cap the event loop queue for backpressure
                if name == "set_queue_limit" {
                    if args.len() != 1 {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 1,
                            got: args.len(),
                        });
                    }
                    match self.eval_expr(&args[0])? {
                        Value::Int(n) if n > 0 => {
                            self.event_loop_queue.set_limit(n as usize);
                            return Ok(Value::Null);
                        }
                        Value::Int(_) => {
                            return Err(RuntimeError::InvalidOperation(
                                "set_queue_limit requires a positive integer".to_string(),
                            ))
                        }
                        other => {
                            return Err(RuntimeError::TypeError {
                                expected: "positive int".to_string(),
                                got: other.type_name().to_string(),
                            })
                        }
                    }
                }

                // set_task_timeout(secs|null) — per-task deadline for on_ready callbacks
                if name == "set_task_timeout" {
                    if args.len() != 1 {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 1,
                            got: args.len(),
                        });
                    }
                    match self.eval_expr(&args[0])? {
                        Value::Null => {
                            self.event_loop_timeout = None;
                            return Ok(Value::Null);
                        }
                        Value::Int(n) if n > 0 => {
                            self.event_loop_timeout = Some(n as f64);
                            return Ok(Value::Null);
                        }
                        Value::Float(f) if f > 0.0 => {
                            self.event_loop_timeout = Some(f);
                            return Ok(Value::Null);
                        }
                        Value::Int(_) | Value::Float(_) => {
                            return Err(RuntimeError::InvalidOperation(
                                "set_task_timeout requires a positive number or null".to_string(),
                            ))
                        }
                        other => {
                            return Err(RuntimeError::TypeError {
                                expected: "positive number or null".to_string(),
                                got: other.type_name().to_string(),
                            })
                        }
                    }
                }

                // Async I/O dispatch when pool is active
                if let Some(pool) = self.io_pool.clone() {
                    if let Some(promise) = self.try_submit_io_task(&name, args, &pool)? {
                        return Ok(promise);
                    }
                }

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
            Value::AsyncFunction {
                params,
                body,
                closure,
            } => {
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
                Ok(Value::promise(
                    Value::AsyncFunction {
                        params,
                        body,
                        closure,
                    },
                    arg_values,
                ))
            }
            _ => Err(RuntimeError::TypeError {
                expected: "function".to_string(),
                got: func_val.type_name().to_string(),
            }),
        }
    }

    pub(super) fn assign_target(
        &mut self,
        target: &Expr,
        value: Value,
    ) -> Result<(), RuntimeError> {
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

    /// Try to submit a known I/O builtin as an async task to the pool.
    /// Returns Some(Promise) if submitted, None if the name is not an async I/O builtin.
    fn try_submit_io_task(
        &mut self,
        name: &str,
        args: &[Expr],
        pool: &Arc<IoPool>,
    ) -> Result<Option<Value>, RuntimeError> {
        let (tx, rx) = std::sync::mpsc::channel::<crate::interpreter::io_pool::IoResult>();

        match name {
            "http_get" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let url = self.require_string_arg(&args[0], "http_get")?;
                let opts = if args.len() == 2 {
                    let v = self.eval_expr(&args[1])?;
                    parse_http_opts(&v)?
                } else {
                    HttpOptions::default()
                };
                pool.submit(IoTask::HttpGet { url, opts, tx });
                Ok(Some(Value::promise_io(rx)))
            }
            "http_post" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 2,
                        got: args.len(),
                    });
                }
                let url = self.require_string_arg(&args[0], "http_post")?;
                let body = self.require_string_arg(&args[1], "http_post")?;
                let opts = if args.len() == 3 {
                    let v = self.eval_expr(&args[2])?;
                    parse_http_opts(&v)?
                } else {
                    HttpOptions::default()
                };
                pool.submit(IoTask::HttpPost {
                    url,
                    body,
                    opts,
                    tx,
                });
                Ok(Some(Value::promise_io(rx)))
            }
            "sleep" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let secs = match self.eval_expr(&args[0])? {
                    Value::Float(f) => f,
                    Value::Int(n) => n as f64,
                    other => {
                        return Err(RuntimeError::TypeError {
                            expected: "number".to_string(),
                            got: other.type_name().to_string(),
                        })
                    }
                };
                pool.submit(IoTask::Sleep { secs, tx });
                Ok(Some(Value::promise_io(rx)))
            }
            "read_file" => {
                if args.len() != 1 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    });
                }
                let path = self.require_string_arg(&args[0], "read_file")?;
                pool.submit(IoTask::ReadFile { path, tx });
                Ok(Some(Value::promise_io(rx)))
            }
            "write_file" => {
                if args.len() != 2 {
                    return Err(RuntimeError::ArityMismatch {
                        expected: 2,
                        got: args.len(),
                    });
                }
                let path = self.require_string_arg(&args[0], "write_file")?;
                let content = self.require_string_arg(&args[1], "write_file")?;
                pool.submit(IoTask::WriteFile { path, content, tx });
                Ok(Some(Value::promise_io(rx)))
            }
            _ => Ok(None),
        }
    }

    fn require_string_arg(&mut self, arg: &Expr, fn_name: &str) -> Result<String, RuntimeError> {
        match self.eval_expr(arg)? {
            Value::String(s) => Ok(s.as_ref().clone()),
            other => Err(RuntimeError::TypeError {
                expected: format!("{} expects string argument", fn_name),
                got: other.type_name().to_string(),
            }),
        }
    }

    /// Register a callback to fire when a promise resolves.
    /// If the promise is already resolved, the callback fires immediately.
    /// If it is IoWaiting, the receiver is moved into the event loop queue.
    fn register_on_ready(
        &mut self,
        promise_val: Value,
        callback: Value,
    ) -> Result<Value, RuntimeError> {
        match promise_val {
            Value::Promise(state_rc) => {
                let state = {
                    let mut s = state_rc.borrow_mut();
                    std::mem::replace(&mut *s, PromiseState::Resolved(Value::Null))
                };
                match state {
                    PromiseState::IoWaiting(rx) => {
                        // Attach per-task deadline from AETHER_EVENT_LOOP_TIMEOUT / set_task_timeout
                        let deadline = self.event_loop_timeout.map(|secs| {
                            std::time::Instant::now() + std::time::Duration::from_secs_f64(secs)
                        });
                        self.event_loop_queue
                            .push(rx, callback, deadline)
                            .map_err(RuntimeError::InvalidOperation)?;
                    }
                    PromiseState::Resolved(val) => {
                        self.call_value(callback, vec![val])?;
                    }
                    PromiseState::Pending { func, args } => {
                        let result = self.exec_async_body(func, args)?;
                        self.call_value(callback, vec![result])?;
                    }
                }
            }
            other => {
                // Non-promise: call callback immediately with the value
                self.call_value(callback, vec![other])?;
            }
        }
        Ok(Value::Null)
    }

    /// Run the event loop until all queued callbacks have fired.
    ///
    /// `loop_deadline`: optional wall-clock cap for the entire loop (from `event_loop(secs)`).
    ///   Exits early if the deadline is reached, regardless of pending tasks.
    ///
    /// Per-task timeouts are independent: each entry carries its own deadline set
    /// at on_ready() time (from AETHER_EVENT_LOOP_TIMEOUT / set_task_timeout).
    /// A timed-out task is aborted in drain_ready() and logged; other tasks continue.
    ///
    /// Error isolation: I/O errors and callback exceptions do NOT abort the
    /// loop — they are logged to stderr and the remaining callbacks continue.
    pub(crate) fn run_event_loop(
        &mut self,
        loop_deadline: Option<f64>,
    ) -> Result<Value, RuntimeError> {
        let loop_end = loop_deadline
            .map(|s| std::time::Instant::now() + std::time::Duration::from_secs_f64(s));

        loop {
            if self.event_loop_queue.is_empty() {
                break;
            }

            // Global loop cap (event_loop(secs) arg) — separate from per-task timeouts
            if let Some(end) = loop_end {
                if std::time::Instant::now() >= end {
                    break;
                }
            }

            let ready = self.event_loop_queue.drain_ready();

            if ready.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }

            for (result, callback) in ready {
                match result {
                    Ok(val) => {
                        if let Err(e) = self.call_value(callback, vec![val]) {
                            eprintln!("event_loop: callback error: {}", e);
                        }
                    }
                    Err(e) => {
                        // Covers both I/O errors and per-task timeouts ("task timed out")
                        eprintln!("event_loop: task failed (callback skipped): {}", e);
                    }
                }
            }
        }

        Ok(Value::Null)
    }
}
