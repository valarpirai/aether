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
                        pairs.borrow_mut().insert(key, value);
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

    /// Drain the async EventLoopQueue and fire any ready callbacks.
    ///
    /// Called from the TCP dispatch loops so that `.then()` / `on_ready` callbacks
    /// registered inside TCP handlers (e.g. `await http_get(...)` inside
    /// `on_message`) fire without requiring a separate `event_loop()` call.
    pub(crate) fn tick_async_callbacks(&mut self) -> Result<(), RuntimeError> {
        let ready = self.async_rt.event_loop_queue.drain_ready();
        for (result, callback) in ready {
            match result {
                Ok(val) => self.call_value(callback, vec![val]).map(|_| ())?,
                Err(e) => eprintln!("tcp: async task failed (callback skipped): {}", e),
            }
        }
        Ok(())
    }

    /// Start the TCP server event loop (event-driven via mio).
    ///
    /// Spawns a single I/O thread that owns all `TcpStream` handles and drives
    /// them with a mio `Poll`.  The main thread receives `TcpEvent`s and
    /// dispatches them to Aether callbacks.
    ///
    /// Exits when the I/O thread closes `event_tx` (on `server.close()`, SIGINT,
    /// or double-SIGINT force exit).
    pub(crate) fn run_tcp_server(
        &mut self,
        state_rc: std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpServerState>>,
    ) -> Result<Value, RuntimeError> {
        use crate::interpreter::tcp::{
            graceful_shutdown_timeout_secs, run_server_io_loop, TcpCommand, TcpEvent, SIGINT_COUNT,
            WAKER_TOKEN,
        };
        use mio::{Poll, Waker};
        use std::sync::atomic::Ordering;
        use std::sync::mpsc::RecvTimeoutError;
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        crate::interpreter::tcp::register_sigint_handler();

        let std_listener = state_rc.borrow_mut().std_listener.take().ok_or_else(|| {
            RuntimeError::InvalidOperation("server.accept() already called".to_string())
        })?;

        let (event_tx, event_rx) = std::sync::mpsc::channel::<TcpEvent>();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<TcpCommand>();

        let poll = Poll::new().map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?;
        let waker = Arc::new(
            Waker::new(poll.registry(), WAKER_TOKEN)
                .map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?,
        );

        {
            let mut state = state_rc.borrow_mut();
            state.cmd_tx = Some(cmd_tx);
            state.waker = Some(waker);
        }

        let delimiter = state_rc.borrow().delimiter.clone();
        let shutdown = state_rc.borrow().shutdown.clone();

        std::thread::spawn(move || {
            run_server_io_loop(std_listener, event_tx, cmd_rx, poll, delimiter, shutdown);
        });

        // Fire on_listen before entering the dispatch loop
        let on_listen_cb = state_rc.borrow().on_listen.clone();
        if let Some(cb) = on_listen_cb {
            self.call_value(cb, vec![])?;
        }

        let mut graceful_start: Option<Instant> = None;

        loop {
            // Double Ctrl+C → force exit
            if SIGINT_COUNT.load(Ordering::Relaxed) >= 2 {
                break;
            }

            // First Ctrl+C → initiate graceful shutdown once
            if SIGINT_COUNT.load(Ordering::Relaxed) >= 1 && graceful_start.is_none() {
                graceful_start = Some(Instant::now());
                let state = state_rc.borrow();
                state.shutdown.store(true, Ordering::Relaxed);
                if let (Some(cmd_tx), Some(waker)) = (&state.cmd_tx, &state.waker) {
                    let _ = cmd_tx.send(TcpCommand::Shutdown);
                    let _ = waker.wake();
                }
            }

            // Graceful timeout expired
            if let Some(start) = graceful_start {
                if start.elapsed() >= Duration::from_secs(graceful_shutdown_timeout_secs()) {
                    break;
                }
            }

            // Block up to 10 ms for the next TCP event
            match event_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(evt) => self.dispatch_tcp_server_event(&state_rc, evt)?,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            // Tick the async event loop so .then() / on_ready callbacks registered
            // inside TCP handlers (e.g. await http_get inside on_message) can fire.
            self.tick_async_callbacks()?;
        }

        state_rc.borrow_mut().closed = true;
        Ok(Value::Null)
    }

    /// Dispatch a single `TcpEvent` to the appropriate Aether callback.
    fn dispatch_tcp_server_event(
        &mut self,
        state_rc: &std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpServerState>>,
        event: crate::interpreter::tcp::TcpEvent,
    ) -> Result<(), RuntimeError> {
        use crate::interpreter::tcp::{TcpConnectionState, TcpEvent};

        match event {
            TcpEvent::Connected { conn_id, peer_addr } => {
                // Build a lightweight conn handle that shares the server's I/O channel
                let (cmd_tx, waker, shutdown) = {
                    let s = state_rc.borrow();
                    (s.cmd_tx.clone(), s.waker.clone(), s.shutdown.clone())
                };
                let conn_state = TcpConnectionState {
                    conn_id,
                    addr: peer_addr,
                    is_client: false,
                    cmd_tx,
                    waker,
                    shutdown,
                    on_connect: None,
                    on_message: None,
                    on_disconnect: None,
                    on_error: None,
                    on_timeout: None,
                    closed: false,
                };
                let conn_val = Value::tcp_connection(conn_state);
                state_rc
                    .borrow_mut()
                    .active_conns
                    .insert(conn_id, conn_val.clone());
                let cb = state_rc.borrow().on_connect.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![conn_val])?;
                }
            }

            TcpEvent::Message { conn_id, data } => {
                let conn_val = state_rc.borrow().active_conns.get(&conn_id).cloned();
                let cb = state_rc.borrow().on_message.clone();
                if let (Some(conn), Some(cb)) = (conn_val, cb) {
                    let bytes: Vec<Value> = data.iter().map(|&b| Value::Int(b as i64)).collect();
                    self.call_value(cb, vec![conn, Value::array(bytes)])?;
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

    /// Connect a TCP client and run its event dispatch loop (event-driven via mio).
    pub(crate) fn run_tcp_client(
        &mut self,
        state_rc: std::rc::Rc<std::cell::RefCell<crate::interpreter::tcp::TcpConnectionState>>,
    ) -> Result<Value, RuntimeError> {
        use crate::interpreter::tcp::{
            run_client_io_loop, TcpCommand, TcpEvent, SIGINT_COUNT, WAKER_TOKEN,
        };
        use mio::{Poll, Waker};
        use std::sync::atomic::Ordering;
        use std::sync::mpsc::RecvTimeoutError;
        use std::sync::Arc;
        use std::time::Duration;

        crate::interpreter::tcp::register_sigint_handler();

        let (event_tx, event_rx) = std::sync::mpsc::channel::<TcpEvent>();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<TcpCommand>();

        let poll = Poll::new().map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?;
        let waker = Arc::new(
            Waker::new(poll.registry(), WAKER_TOKEN)
                .map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?,
        );

        {
            let mut state = state_rc.borrow_mut();
            state.cmd_tx = Some(cmd_tx);
            state.waker = Some(waker);
        }

        let addr = state_rc.borrow().addr.clone();
        let shutdown = state_rc.borrow().shutdown.clone();

        std::thread::spawn(move || {
            run_client_io_loop(&addr, event_tx, cmd_rx, poll, shutdown);
        });

        loop {
            if SIGINT_COUNT.load(Ordering::Relaxed) >= 2 {
                break;
            }

            match event_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(evt) => self.dispatch_tcp_client_event(&state_rc, evt)?,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            // Tick async event loop alongside the TCP client loop.
            self.tick_async_callbacks()?;

            if state_rc.borrow().closed {
                break;
            }
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
            TcpEvent::Connected { .. } => {
                let cb = state_rc.borrow().on_connect.clone();
                if let Some(cb) = cb {
                    self.call_value(cb, vec![])?;
                }
            }
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
        }
        Ok(())
    }

    /// Start the UDP socket event loop (event-driven via mio).
    pub(crate) fn run_udp_socket(
        &mut self,
        state_rc: std::rc::Rc<std::cell::RefCell<crate::interpreter::udp::UdpSocketState>>,
    ) -> Result<Value, RuntimeError> {
        use crate::interpreter::tcp::SIGINT_COUNT;
        use crate::interpreter::udp::{run_udp_io_loop, UdpCommand, UdpEvent, WAKER_TOKEN};
        use mio::{Poll, Waker};
        use std::sync::atomic::Ordering;
        use std::sync::mpsc::RecvTimeoutError;
        use std::sync::Arc;
        use std::time::Duration;

        crate::interpreter::tcp::register_sigint_handler();

        let std_socket = state_rc.borrow_mut().std_socket.take().ok_or_else(|| {
            RuntimeError::InvalidOperation("sock.listen() already called".to_string())
        })?;

        let (event_tx, event_rx) = std::sync::mpsc::channel::<UdpEvent>();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<UdpCommand>();

        let poll = Poll::new().map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?;
        let waker = Arc::new(
            Waker::new(poll.registry(), WAKER_TOKEN)
                .map_err(|e| RuntimeError::InvalidOperation(e.to_string()))?,
        );

        {
            let mut state = state_rc.borrow_mut();
            state.cmd_tx = Some(cmd_tx);
            state.waker = Some(waker);
        }

        let shutdown = state_rc.borrow().shutdown.clone();

        std::thread::spawn(move || {
            run_udp_io_loop(std_socket, event_tx, cmd_rx, poll, shutdown);
        });

        loop {
            if SIGINT_COUNT.load(Ordering::Relaxed) >= 2 {
                break;
            }

            match event_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(evt) => self.dispatch_udp_event(&state_rc, evt)?,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            self.tick_async_callbacks()?;

            if state_rc.borrow().closed {
                break;
            }
        }

        state_rc.borrow_mut().closed = true;
        Ok(Value::Null)
    }

    fn dispatch_udp_event(
        &mut self,
        state_rc: &std::rc::Rc<std::cell::RefCell<crate::interpreter::udp::UdpSocketState>>,
        event: crate::interpreter::udp::UdpEvent,
    ) -> Result<(), RuntimeError> {
        use crate::interpreter::udp::UdpEvent;

        match event {
            UdpEvent::Message { data, addr } => {
                let cb = state_rc.borrow().on_message.clone();
                if let Some(cb) = cb {
                    let bytes: Vec<Value> = data.iter().map(|&b| Value::Int(b as i64)).collect();
                    self.call_value(cb, vec![Value::array(bytes), Value::string(addr)])?;
                }
            }
            UdpEvent::Error(msg) => {
                eprintln!("udp: {}", msg);
            }
        }
        Ok(())
    }
}
