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
                self.calls.depth += 1;
                if self.calls.depth > self.calls.max_depth {
                    self.calls.depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.calls.depth + 1,
                        limit: self.calls.max_depth,
                    });
                }
                if arg_values.len() > params.len() {
                    self.calls.depth -= 1;
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: arg_values.len(),
                    });
                }
                let mut padded = arg_values;
                while padded.len() < params.len() {
                    padded.push(Value::Null);
                }
                self.calls.stack.push(StackFrame {
                    fn_name: "<anonymous>".to_string(),
                    call_site_line: self.calls.current_line,
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
                        self.calls.depth -= 1;
                        return Err(e);
                    }
                };
                std::mem::swap(&mut self.environment, &mut call_env);
                self.calls.stack.pop();
                self.calls.depth -= 1;
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
            Value::EnumConstructor {
                enum_name,
                variant_name,
                fields,
            } => {
                if arg_values.len() != fields.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: fields.len(),
                        got: arg_values.len(),
                    });
                }
                let named: Vec<(String, Value)> = fields.into_iter().zip(arg_values).collect();
                Ok(Value::EnumVariant {
                    type_name: format!("{}.{}", enum_name, variant_name),
                    enum_name,
                    variant_name,
                    fields: Rc::new(named),
                })
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
            other => Err(RuntimeError::NotCallable {
                type_name: other.type_name().to_string(),
            }),
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
                return Err(RuntimeError::NotCallable {
                    type_name: other.type_name().to_string(),
                })
            }
        };

        self.calls.depth += 1;
        if self.calls.depth > self.calls.max_depth {
            self.calls.depth -= 1;
            return Err(RuntimeError::StackOverflow {
                depth: self.calls.depth + 1,
                limit: self.calls.max_depth,
            });
        }
        if arg_values.len() > params.len() {
            self.calls.depth -= 1;
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
        self.calls.depth -= 1;
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
                self.calls.depth += 1;
                if self.calls.depth > self.calls.max_depth {
                    self.calls.depth -= 1;
                    return Err(RuntimeError::StackOverflow {
                        depth: self.calls.depth + 1,
                        limit: self.calls.max_depth,
                    });
                }

                if args.len() > params.len() {
                    self.calls.depth -= 1;
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
                            self.calls.depth -= 1;
                            return Err(e);
                        }
                    }
                }

                while arg_values.len() < params.len() {
                    arg_values.push(Value::Null);
                }

                self.calls.stack.push(StackFrame {
                    fn_name: func_name.clone(),
                    call_site_line: self.calls.current_line,
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
                        self.calls.depth -= 1;
                        return Err(e);
                    }
                };

                self.environment = saved_env;
                self.calls.stack.pop();
                self.calls.depth -= 1;

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
                    self.async_rt.io_pool = Some(Arc::new(IoPool::new(n)));
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
                        self.async_rt.event_loop_timeout
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
                            self.async_rt.event_loop_queue.set_limit(n as usize);
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
                            self.async_rt.event_loop_timeout = None;
                            return Ok(Value::Null);
                        }
                        Value::Int(n) if n > 0 => {
                            self.async_rt.event_loop_timeout = Some(n as f64);
                            return Ok(Value::Null);
                        }
                        Value::Float(f) if f > 0.0 => {
                            self.async_rt.event_loop_timeout = Some(f);
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
                if let Some(pool) = self.async_rt.io_pool.clone() {
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
                        let len = elements.borrow().len();
                        if idx < 0 || idx as usize >= len {
                            return Err(RuntimeError::IndexOutOfBounds {
                                index: idx,
                                length: len,
                            });
                        }
                        elements.borrow_mut()[idx as usize] = value;
                        Ok(())
                    }
                    (Value::Dict(pairs), key) => {
                        let mut found = false;
                        for (k, v) in pairs.borrow_mut().iter_mut() {
                            if k == &key {
                                *v = value.clone();
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            pairs.borrow_mut().push((key, value));
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
    pub(super) fn register_on_ready(
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
                        let deadline = self.async_rt.event_loop_timeout.map(|secs| {
                            std::time::Instant::now() + std::time::Duration::from_secs_f64(secs)
                        });
                        let limit = self.async_rt.event_loop_queue.limit();
                        self.async_rt
                            .event_loop_queue
                            .push(rx, callback, deadline)
                            .map_err(|_| RuntimeError::QueueFull { limit })?;
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
            if self.async_rt.event_loop_queue.is_empty() {
                break;
            }

            // Global loop cap (event_loop(secs) arg) — separate from per-task timeouts
            if let Some(end) = loop_end {
                if std::time::Instant::now() >= end {
                    break;
                }
            }

            let ready = self.async_rt.event_loop_queue.drain_ready();

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

    /// Start the TCP server accept loop and event dispatch.
    ///
    /// Spawns a background accept thread and a per-connection reader thread for
    /// each incoming client. Runs a polling event loop on the main thread that
    /// dispatches `TcpEvent`s to the registered Aether callbacks.
    ///
    /// Exits when:
    /// - `server.close()` is called from a callback, or
    /// - Ctrl+C (SIGINT) is received (graceful drain then exit).
    pub(crate) fn run_tcp_server(
        &mut self,
        state_rc: std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpServerState>>,
    ) -> Result<Value, RuntimeError> {
        use crate::interpreter::tcp::{graceful_shutdown_timeout_secs, SIGINT_COUNT};
        use std::sync::atomic::Ordering;
        use std::thread;
        use std::time::{Duration, Instant};

        crate::interpreter::tcp::register_sigint_handler();

        // Fire on_listen callback before starting the accept thread
        let on_listen_cb = state_rc.borrow().on_listen.clone();
        if let Some(cb) = on_listen_cb {
            self.call_value(cb, vec![])?;
        }

        // Spawn the accept thread
        {
            let state = state_rc.borrow();
            let listener = state.listener.clone();
            let event_tx = state.event_tx.clone();
            let shutdown = state.shutdown.clone();
            let delimiter = state.delimiter.clone();

            thread::spawn(move || {
                tcp_accept_loop(listener, event_tx, shutdown, delimiter);
            });
        }

        // Main event dispatch loop
        loop {
            // Check server.close() or double-SIGINT (force exit)
            {
                let state = state_rc.borrow();
                if state.closed || SIGINT_COUNT.load(Ordering::Relaxed) >= 2 {
                    break;
                }
            }

            // Graceful shutdown on first SIGINT
            if SIGINT_COUNT.load(Ordering::Relaxed) >= 1 {
                state_rc.borrow().shutdown.store(true, Ordering::Relaxed);
                let deadline =
                    Instant::now() + Duration::from_secs(graceful_shutdown_timeout_secs());
                while Instant::now() < deadline {
                    let event = state_rc.borrow_mut().event_rx.try_recv().ok();
                    match event {
                        Some(evt) => self.dispatch_tcp_server_event(&state_rc, evt)?,
                        None => {
                            if state_rc.borrow().active_conns.is_empty() {
                                break;
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
                break;
            }

            // Normal poll for events
            loop {
                let event = state_rc.borrow_mut().event_rx.try_recv().ok();
                match event {
                    Some(evt) => self.dispatch_tcp_server_event(&state_rc, evt)?,
                    None => break,
                }
            }

            thread::sleep(Duration::from_millis(1));
        }

        state_rc.borrow_mut().closed = true;
        Ok(Value::Null)
    }

    /// Dispatch a single `TcpEvent` to the appropriate Aether callback on the server.
    fn dispatch_tcp_server_event(
        &mut self,
        state_rc: &std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpServerState>>,
        event: crate::interpreter::tcp::TcpEvent,
    ) -> Result<(), RuntimeError> {
        use crate::interpreter::tcp::TcpConnectionState;
        use crate::interpreter::tcp::TcpEvent;
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        match event {
            TcpEvent::Connected {
                conn_id,
                stream,
                peer_addr,
            } => {
                // Build a TcpConnection value representing this client
                let (conn_tx, conn_rx) = std::sync::mpsc::channel();
                let conn_shutdown = Arc::new(AtomicBool::new(false));
                let conn_state = TcpConnectionState {
                    addr: peer_addr,
                    stream: Some(stream),
                    event_tx: conn_tx,
                    event_rx: conn_rx,
                    on_connect: None,
                    on_message: None,
                    on_disconnect: None,
                    on_error: None,
                    on_timeout: None,
                    closed: false,
                    shutdown: conn_shutdown,
                };
                let conn_val = Value::tcp_connection(conn_state);
                state_rc
                    .borrow_mut()
                    .active_conns
                    .insert(conn_id, conn_val.clone());
                // Release borrow before calling into Aether (callback may mutate state)
                let cb = state_rc.borrow().on_connect.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![conn_val])?;
                }
            }

            TcpEvent::Message { conn_id, data } => {
                // Release borrow before calling into Aether
                let conn_val = state_rc.borrow().active_conns.get(&conn_id).cloned();
                let cb = state_rc.borrow().on_message.clone();
                if let (Some(conn), Some(cb)) = (conn_val, cb) {
                    let bytes: Vec<Value> = data.iter().map(|&b| Value::Int(b as i64)).collect();
                    let data_val = Value::array(bytes);
                    self.call_value(cb, vec![conn, data_val])?;
                }
            }

            TcpEvent::Disconnected { conn_id } => {
                let conn_val = state_rc.borrow_mut().active_conns.remove(&conn_id);
                let cb = state_rc.borrow().on_disconnect.clone();
                if let (Some(conn), Some(cb)) = (conn_val, cb) {
                    self.call_value(cb, vec![conn])?;
                }
            }

            TcpEvent::Error(msg) => {
                let cb = state_rc.borrow().on_error.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![Value::string(msg)])?;
                }
            }

            TcpEvent::Timeout { conn_id } => {
                let conn_val = state_rc.borrow().active_conns.get(&conn_id).cloned();
                let cb = state_rc.borrow().on_timeout.clone();
                if let (Some(conn), Some(cb)) = (conn_val, cb) {
                    self.call_value(cb, vec![conn])?;
                }
            }
        }
        Ok(())
    }

    /// Connect a client connection and start its reader loop.
    pub(crate) fn run_tcp_client(
        &mut self,
        state_rc: std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpConnectionState>>,
    ) -> Result<Value, RuntimeError> {
        use crate::interpreter::tcp::{graceful_shutdown_timeout_secs, SIGINT_COUNT};
        use std::sync::atomic::Ordering;
        use std::thread;
        use std::time::{Duration, Instant};

        crate::interpreter::tcp::register_sigint_handler();

        // Establish the TCP connection
        let addr = state_rc.borrow().addr.clone();
        let stream = std::net::TcpStream::connect(&addr).map_err(|e| {
            RuntimeError::InvalidOperation(format!(
                "tcp_connect: cannot connect to {}: {}",
                addr, e
            ))
        })?;
        let write_arc = std::sync::Arc::new(stream.try_clone().map_err(|e| {
            RuntimeError::InvalidOperation(format!("tcp_connect: stream clone failed: {}", e))
        })?);
        state_rc.borrow_mut().stream = Some(write_arc);

        // Fire on_connect
        let on_connect_cb = state_rc.borrow().on_connect.clone();
        if let Some(cb) = on_connect_cb {
            self.call_value(cb, vec![])?;
        }

        // Spawn reader thread
        {
            let state = state_rc.borrow();
            let event_tx = state.event_tx.clone();
            let shutdown = state.shutdown.clone();
            thread::spawn(move || {
                tcp_client_read_loop(stream, event_tx, shutdown, 0);
            });
        }

        // Main event dispatch loop for the client
        loop {
            {
                let state = state_rc.borrow();
                if state.closed || SIGINT_COUNT.load(Ordering::Relaxed) >= 2 {
                    break;
                }
            }

            if SIGINT_COUNT.load(Ordering::Relaxed) >= 1 {
                state_rc.borrow().shutdown.store(true, Ordering::Relaxed);
                let deadline =
                    Instant::now() + Duration::from_secs(graceful_shutdown_timeout_secs());
                while Instant::now() < deadline {
                    let event = state_rc.borrow_mut().event_rx.try_recv().ok();
                    match event {
                        Some(evt) => self.dispatch_tcp_client_event(&state_rc, evt)?,
                        None => break,
                    }
                }
                break;
            }

            loop {
                let event = state_rc.borrow_mut().event_rx.try_recv().ok();
                match event {
                    Some(evt) => self.dispatch_tcp_client_event(&state_rc, evt)?,
                    None => break,
                }
            }

            thread::sleep(Duration::from_millis(1));
        }

        state_rc.borrow_mut().closed = true;
        Ok(Value::Null)
    }

    fn dispatch_tcp_client_event(
        &mut self,
        state_rc: &std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpConnectionState>>,
        event: crate::interpreter::tcp::TcpEvent,
    ) -> Result<(), RuntimeError> {
        use crate::interpreter::tcp::TcpEvent;
        use std::sync::atomic::Ordering;

        match event {
            TcpEvent::Message { data, .. } => {
                let cb = state_rc.borrow().on_message.clone();
                if let Some(cb) = cb {
                    let bytes: Vec<Value> = data.iter().map(|&b| Value::Int(b as i64)).collect();
                    self.call_value(cb, vec![Value::array(bytes)])?;
                }
            }
            TcpEvent::Disconnected { .. } => {
                let cb = state_rc.borrow().on_disconnect.clone();
                {
                    let mut s = state_rc.borrow_mut();
                    s.closed = true;
                    s.shutdown.store(true, Ordering::Relaxed);
                }
                if let Some(cb) = cb {
                    self.call_value(cb, vec![])?;
                }
            }
            TcpEvent::Error(msg) => {
                let cb = state_rc.borrow().on_error.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![Value::string(msg)])?;
                }
            }
            TcpEvent::Timeout { .. } => {
                let cb = state_rc.borrow().on_timeout.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![])?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Background thread: accept incoming connections and spawn a reader per connection.
fn tcp_accept_loop(
    listener: std::sync::Arc<std::net::TcpListener>,
    event_tx: std::sync::mpsc::Sender<crate::interpreter::tcp::TcpEvent>,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    delimiter: Option<String>,
) {
    use crate::interpreter::tcp::TcpEvent;
    use std::io::ErrorKind;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    let mut next_id: u64 = 0;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        match listener.accept() {
            Ok((stream, addr)) => {
                let conn_id = next_id;
                next_id += 1;

                let write_stream = match stream.try_clone() {
                    Ok(s) => std::sync::Arc::new(s),
                    Err(e) => {
                        let _ =
                            event_tx.send(TcpEvent::Error(format!("stream clone failed: {}", e)));
                        continue;
                    }
                };

                let _ = event_tx.send(TcpEvent::Connected {
                    conn_id,
                    stream: write_stream,
                    peer_addr: addr.to_string(),
                });

                let tx = event_tx.clone();
                let sd = shutdown.clone();
                let delim = delimiter.clone();
                thread::spawn(move || {
                    tcp_client_read_loop(stream, tx, sd, conn_id);
                    drop(delim);
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = event_tx.send(TcpEvent::Error(e.to_string()));
                break;
            }
        }
    }
}

/// Background thread: read data from a connection and send events.
fn tcp_client_read_loop(
    mut stream: std::net::TcpStream,
    event_tx: std::sync::mpsc::Sender<crate::interpreter::tcp::TcpEvent>,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
    conn_id: u64,
) {
    use crate::interpreter::tcp::TcpEvent;
    use std::io::Read;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok();

    let mut buf = vec![0u8; 4096];

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        match stream.read(&mut buf) {
            Ok(0) => {
                let _ = event_tx.send(TcpEvent::Disconnected { conn_id });
                break;
            }
            Ok(n) => {
                let _ = event_tx.send(TcpEvent::Message {
                    conn_id,
                    data: buf[..n].to_vec(),
                });
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock =>
            {
                continue;
            }
            Err(_) => {
                let _ = event_tx.send(TcpEvent::Disconnected { conn_id });
                break;
            }
        }
    }
}
