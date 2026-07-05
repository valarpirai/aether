//! Expression evaluation and statement execution for the Aether interpreter

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::environment::{Environment, RuntimeError, StackFrame};
use super::event_loop::EventLoopQueue;
use super::io_pool::IoPool;
use super::value::Value;
use crate::parser::ast::Stmt;

mod expressions;
mod functions;
mod members;
mod modules;
mod operators;
mod statements;

/// Control flow signals returned from statement execution
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    None,
    Return(Value),
    /// Break, with optional target label
    Break(Option<String>),
    /// Continue, with optional target label
    Continue(Option<String>),
}

/// Call-depth tracking, stack frames, and current line — managed together during function entry/exit
pub(crate) struct CallContext {
    pub(crate) depth: usize,
    pub(crate) max_depth: usize,
    pub(crate) stack: Vec<StackFrame>,
    pub(crate) current_line: usize,
}

impl CallContext {
    fn new(max_depth: usize) -> Self {
        Self {
            depth: 0,
            max_depth,
            stack: Vec::new(),
            current_line: 0,
        }
    }
}

/// Module cache and in-progress loading set — only accessed during import resolution
pub(crate) struct ModuleLoader {
    pub(crate) cache: HashMap<String, Environment>,
    pub(crate) loading_stack: Vec<String>,
}

impl ModuleLoader {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            loading_stack: Vec::new(),
        }
    }
}

/// I/O thread pool and event loop state — only accessed for async I/O and on_ready/event_loop
pub(crate) struct AsyncRuntime {
    pub(crate) io_pool: Option<Arc<IoPool>>,
    pub(crate) event_loop_queue: EventLoopQueue,
    pub(crate) event_loop_timeout: Option<f64>,
}

impl AsyncRuntime {
    fn new(queue: EventLoopQueue, timeout: Option<f64>) -> Self {
        Self {
            io_pool: None,
            event_loop_queue: queue,
            event_loop_timeout: timeout,
        }
    }
}

/// Controls whether execution pauses at the next Stmt::Line marker
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StepMode {
    /// Normal execution — debugger is not active
    Running,
    /// Waiting at a breakpoint for a user command
    Paused,
    /// Pause at the very next Stmt::Line (step into)
    Step,
    /// Pause at the next Stmt::Line where call depth <= the stored depth (step over)
    Next(usize),
}

pub(crate) struct DebugState {
    pub(crate) mode: StepMode,
}

impl DebugState {
    fn new() -> Self {
        Self {
            mode: StepMode::Running,
        }
    }
}

/// Tree-walking interpreter for Aether programs
pub struct Evaluator {
    /// Current environment (variables in scope)
    pub environment: Environment,
    /// Current file being executed (for relative imports and stack traces)
    pub current_file: Option<PathBuf>,
    /// Call depth, stack frames, and line tracking
    pub(crate) calls: CallContext,
    /// Module cache and circular-import detection
    pub(crate) modules: ModuleLoader,
    /// I/O thread pool and event loop queue
    pub(crate) async_rt: AsyncRuntime,
    /// Debugger step mode — checked on every Stmt::Line
    pub(crate) debug: DebugState,
}

impl Evaluator {
    /// Read AETHER_EVENT_LOOP_TIMEOUT from the environment (positive float, in seconds).
    fn env_event_loop_timeout() -> Option<f64> {
        std::env::var("AETHER_EVENT_LOOP_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|&v| v > 0.0)
    }

    /// Read AETHER_QUEUE_LIMIT from the environment (positive integer).
    fn env_queue_limit() -> Option<usize> {
        std::env::var("AETHER_QUEUE_LIMIT")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v > 0)
    }

    /// Create a new evaluator with a fresh environment (includes stdlib)
    pub fn new() -> Self {
        Self::new_with_stdlib()
    }

    /// Create a new evaluator with stdlib loaded
    pub fn new_with_stdlib() -> Self {
        let mut evaluator = Self::new_base();
        evaluator.register_builtins();
        evaluator.load_stdlib();
        evaluator
    }

    /// Create a new evaluator without stdlib (faster for tests)
    pub fn new_without_stdlib() -> Self {
        let mut evaluator = Self::new_base();
        evaluator.register_builtins();
        evaluator
    }

    /// Create a new evaluator with an I/O thread pool (Phase 2)
    pub fn new_with_pool(workers: usize) -> Self {
        let mut evaluator = Self::new_with_stdlib();
        evaluator.async_rt.io_pool = Some(Arc::new(IoPool::new(workers)));
        evaluator
    }

    fn new_base() -> Self {
        let mut queue = EventLoopQueue::new();
        if let Some(limit) = Self::env_queue_limit() {
            queue.set_limit(limit);
        }
        Self {
            environment: Environment::new(),
            current_file: None,
            calls: CallContext::new(100),
            modules: ModuleLoader::new(),
            async_rt: AsyncRuntime::new(queue, Self::env_event_loop_timeout()),
            debug: DebugState::new(),
        }
    }

    /// Override the maximum recursion depth (used by AETHER_CALL_DEPTH env var)
    pub fn set_max_call_depth(&mut self, depth: usize) {
        self.calls.max_depth = depth;
    }

    /// Populate the global `args` array with the script's command-line arguments.
    /// Each element is a string. Called by the CLI after creating the evaluator.
    pub fn set_script_args(&mut self, script_args: &[String]) {
        let values: Vec<Value> = script_args.iter().map(Value::string).collect();
        self.environment
            .define("args".to_string(), Value::array(values));
    }

    /// Most recently seen source line (updated by Stmt::Line markers)
    pub fn current_line(&self) -> usize {
        self.calls.current_line
    }

    /// Return the current file's name (e.g. "main.ae") for stack frames, or None.
    pub(crate) fn current_file_name(&self) -> Option<String> {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    }

    /// Read up to `radius` lines around `target` from `path`. Returns None if the file
    /// cannot be read. Each entry is (line_number, line_text).
    fn read_source_context(
        path: &std::path::Path,
        target: usize,
        radius: usize,
    ) -> Option<Vec<(usize, String)>> {
        let source = std::fs::read_to_string(path).ok()?;
        let lines: Vec<&str> = source.lines().collect();
        let first = target.saturating_sub(radius + 1);
        let last = (target + radius).min(lines.len());
        Some(
            lines[first..last]
                .iter()
                .enumerate()
                .map(|(i, l)| (first + i + 1, l.to_string()))
                .collect(),
        )
    }

    /// Print source context centered on the current line, with a `>` marker.
    fn print_source_context(&self) {
        let line = self.calls.current_line;
        let file_path = match &self.current_file {
            Some(p) => p.clone(),
            None => {
                eprintln!("[source not available]");
                return;
            }
        };
        match Self::read_source_context(&file_path, line, 2) {
            None => eprintln!("[source not available]"),
            Some(ctx) => {
                eprintln!();
                for (n, text) in &ctx {
                    if *n == line {
                        eprintln!(">  {}: {}", n, text);
                    } else {
                        eprintln!("   {}: {}", n, text);
                    }
                }
                eprintln!();
            }
        }
    }

    /// Pause execution, show context, and loop on stdin until the user resumes.
    /// Sets self.debug.mode before returning so the evaluator knows how to continue.
    pub(crate) fn trigger_debugger(&mut self) {
        use crate::lexer::Scanner;
        use crate::parser::Parser;
        use std::io::{BufRead, Write};

        let file_display = self
            .current_file
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("<repl>")
            .to_string();
        let line = self.calls.current_line;

        eprintln!("[debugger] Paused at {}:{}", file_display, line);
        self.print_source_context();
        eprintln!("Commands: c/continue  n/next  s/step  bt/backtrace  vars  q/quit  <expr>");

        let stdin = std::io::stdin();

        loop {
            print!("(dbg) ");
            std::io::stdout().flush().ok();

            let mut input = String::new();
            match stdin.lock().read_line(&mut input) {
                Ok(0) | Err(_) => {
                    // EOF — resume so non-interactive runs (tests, pipes) aren't blocked
                    self.debug.mode = StepMode::Running;
                    break;
                }
                Ok(_) => {}
            }

            match input.trim() {
                "" => continue,

                "c" | "continue" => {
                    self.debug.mode = StepMode::Running;
                    break;
                }

                "n" | "next" => {
                    self.debug.mode = StepMode::Next(self.calls.depth);
                    break;
                }

                "s" | "step" => {
                    self.debug.mode = StepMode::Step;
                    break;
                }

                "q" | "quit" => std::process::exit(0),

                "bt" | "backtrace" => {
                    if self.calls.stack.is_empty() {
                        eprintln!("  (top level)");
                    } else {
                        for frame in self.calls.stack.iter().rev() {
                            let f = frame.call_site_file.as_deref().unwrap_or("<native>");
                            eprintln!("  {} at {}:{}", frame.fn_name, f, frame.call_site_line);
                        }
                    }
                }

                "vars" | "env" => {
                    let bindings = self.environment.bindings();
                    if bindings.is_empty() {
                        eprintln!("  (no local variables)");
                    } else {
                        let mut pairs: Vec<_> = bindings.iter().collect();
                        pairs.sort_by_key(|(k, _)| k.as_str());
                        for (name, val) in pairs {
                            eprintln!("  {} = {}", name, val);
                        }
                    }
                }

                expr => {
                    let mut scanner = Scanner::new(expr);
                    let tokens = match scanner.scan_tokens() {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("[error] {}", e);
                            continue;
                        }
                    };
                    let mut parser = Parser::new(tokens);
                    let program = match parser.parse() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("[error] {}", e);
                            continue;
                        }
                    };
                    if program.statements.is_empty() {
                        continue;
                    }
                    let stmts = program.statements;
                    let last = stmts.last().unwrap().clone();
                    for stmt in &stmts[..stmts.len() - 1] {
                        if let Err(e) = self.exec_stmt(stmt) {
                            eprintln!("[error] {}", e);
                            break;
                        }
                    }
                    match &last {
                        Stmt::Expr(e) => match self.eval_expr(e) {
                            Ok(val) if !matches!(val, Value::Null) => eprintln!("{}", val),
                            Ok(_) => {}
                            Err(e) => eprintln!("[error] {}", e),
                        },
                        _ => {
                            if let Err(e) = self.exec_stmt(&last) {
                                eprintln!("[error] {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Register all built-in functions in the environment
    fn register_builtins(&mut self) {
        use super::builtins;

        // I/O functions
        self.environment.define(
            "print".to_string(),
            Value::BuiltinFn {
                name: "print".to_string(),
                arity: usize::MAX,
                func: builtins::builtin_print,
            },
        );

        self.environment.define(
            "println".to_string(),
            Value::BuiltinFn {
                name: "println".to_string(),
                arity: usize::MAX,
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
                arity: usize::MAX, // 1 or 2 args
                func: builtins::builtin_int,
            },
        );
        self.environment.define(
            "hex".to_string(),
            Value::BuiltinFn {
                name: "hex".to_string(),
                arity: 1,
                func: builtins::builtin_hex,
            },
        );
        self.environment.define(
            "oct".to_string(),
            Value::BuiltinFn {
                name: "oct".to_string(),
                arity: 1,
                func: builtins::builtin_oct,
            },
        );
        self.environment.define(
            "bin".to_string(),
            Value::BuiltinFn {
                name: "bin".to_string(),
                arity: 1,
                func: builtins::builtin_bin,
            },
        );
        self.environment.define(
            "base64_encode".to_string(),
            Value::BuiltinFn {
                name: "base64_encode".to_string(),
                arity: 1,
                func: builtins::builtin_base64_encode,
            },
        );
        self.environment.define(
            "base64_decode".to_string(),
            Value::BuiltinFn {
                name: "base64_decode".to_string(),
                arity: 1,
                func: builtins::builtin_base64_decode,
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

        // I/O functions
        self.environment.define(
            "read_file".to_string(),
            Value::BuiltinFn {
                name: "read_file".to_string(),
                arity: 1,
                func: builtins::builtin_read_file,
            },
        );

        self.environment.define(
            "write_file".to_string(),
            Value::BuiltinFn {
                name: "write_file".to_string(),
                arity: 2,
                func: builtins::builtin_write_file,
            },
        );

        self.environment.define(
            "read_lines".to_string(),
            Value::BuiltinFn {
                name: "read_lines".to_string(),
                arity: 1,
                func: builtins::builtin_read_lines,
            },
        );

        self.environment.define(
            "append_file".to_string(),
            Value::BuiltinFn {
                name: "append_file".to_string(),
                arity: 2,
                func: builtins::builtin_append_file,
            },
        );

        self.environment.define(
            "file_exists".to_string(),
            Value::BuiltinFn {
                name: "file_exists".to_string(),
                arity: 1,
                func: builtins::builtin_file_exists,
            },
        );

        self.environment.define(
            "is_file".to_string(),
            Value::BuiltinFn {
                name: "is_file".to_string(),
                arity: 1,
                func: builtins::builtin_is_file,
            },
        );

        self.environment.define(
            "is_dir".to_string(),
            Value::BuiltinFn {
                name: "is_dir".to_string(),
                arity: 1,
                func: builtins::builtin_is_dir,
            },
        );

        self.environment.define(
            "mkdir".to_string(),
            Value::BuiltinFn {
                name: "mkdir".to_string(),
                arity: 1,
                func: builtins::builtin_mkdir,
            },
        );

        self.environment.define(
            "lines_iter".to_string(),
            Value::BuiltinFn {
                name: "lines_iter".to_string(),
                arity: 1,
                func: builtins::builtin_lines_iter,
            },
        );

        self.environment.define(
            "read_bytes".to_string(),
            Value::BuiltinFn {
                name: "read_bytes".to_string(),
                arity: 1,
                func: builtins::builtin_read_bytes,
            },
        );

        self.environment.define(
            "write_bytes".to_string(),
            Value::BuiltinFn {
                name: "write_bytes".to_string(),
                arity: 2,
                func: builtins::builtin_write_bytes,
            },
        );
        self.environment.define(
            "list_dir".to_string(),
            Value::BuiltinFn {
                name: "list_dir".to_string(),
                arity: 1,
                func: builtins::builtin_list_dir,
            },
        );
        self.environment.define(
            "path_join".to_string(),
            Value::BuiltinFn {
                name: "path_join".to_string(),
                arity: usize::MAX,
                func: builtins::builtin_path_join,
            },
        );
        self.environment.define(
            "rename".to_string(),
            Value::BuiltinFn {
                name: "rename".to_string(),
                arity: 2,
                func: builtins::builtin_rename,
            },
        );
        self.environment.define(
            "rm".to_string(),
            Value::BuiltinFn {
                name: "rm".to_string(),
                arity: 1,
                func: builtins::builtin_rm,
            },
        );

        self.environment.define(
            "input".to_string(),
            Value::BuiltinFn {
                name: "input".to_string(),
                arity: usize::MAX,
                func: builtins::builtin_input,
            },
        );

        // Time functions
        self.environment.define(
            "clock".to_string(),
            Value::BuiltinFn {
                name: "clock".to_string(),
                arity: 0,
                func: builtins::builtin_clock,
            },
        );

        self.environment.define(
            "sleep".to_string(),
            Value::BuiltinFn {
                name: "sleep".to_string(),
                arity: 1,
                func: builtins::builtin_sleep,
            },
        );

        // Random number functions
        self.environment.define(
            "random".to_string(),
            Value::BuiltinFn {
                name: "random".to_string(),
                arity: 0,
                func: builtins::builtin_random,
            },
        );

        self.environment.define(
            "rand_int".to_string(),
            Value::BuiltinFn {
                name: "rand_int".to_string(),
                arity: 1,
                func: builtins::builtin_rand_int,
            },
        );

        self.environment.define(
            "format".to_string(),
            Value::BuiltinFn {
                name: "format".to_string(),
                arity: usize::MAX,
                func: builtins::builtin_format,
            },
        );

        // Collection functions
        self.environment.define(
            "set".to_string(),
            Value::BuiltinFn {
                name: "set".to_string(),
                arity: 1,
                func: builtins::builtin_set,
            },
        );

        // Weak reference functions (for breaking GC cycles)
        self.environment.define(
            "make_weak".to_string(),
            Value::BuiltinFn {
                name: "make_weak".to_string(),
                arity: 1,
                func: builtins::builtin_make_weak,
            },
        );
        self.environment.define(
            "upgrade_weak".to_string(),
            Value::BuiltinFn {
                name: "upgrade_weak".to_string(),
                arity: 1,
                func: builtins::builtin_upgrade_weak,
            },
        );
        self.environment.define(
            "is_weak".to_string(),
            Value::BuiltinFn {
                name: "is_weak".to_string(),
                arity: 1,
                func: builtins::builtin_is_weak,
            },
        );
        self.environment.define(
            "id".to_string(),
            Value::BuiltinFn {
                name: "id".to_string(),
                arity: 1,
                func: builtins::builtin_id,
            },
        );
        self.environment.define(
            "copy".to_string(),
            Value::BuiltinFn {
                name: "copy".to_string(),
                arity: 1,
                func: builtins::builtin_copy,
            },
        );

        // JSON functions
        self.environment.define(
            "json_parse".to_string(),
            Value::BuiltinFn {
                name: "json_parse".to_string(),
                arity: 1,
                func: builtins::builtin_json_parse,
            },
        );

        self.environment.define(
            "json_stringify".to_string(),
            Value::BuiltinFn {
                name: "json_stringify".to_string(),
                arity: 1,
                func: builtins::builtin_json_stringify,
            },
        );

        // CSV functions
        self.environment.define(
            "csv_parse".to_string(),
            Value::BuiltinFn {
                name: "csv_parse".to_string(),
                arity: usize::MAX, // 1 or 2 args
                func: builtins::builtin_csv_parse,
            },
        );
        self.environment.define(
            "csv_stringify".to_string(),
            Value::BuiltinFn {
                name: "csv_stringify".to_string(),
                arity: usize::MAX, // 1 or 2 args
                func: builtins::builtin_csv_stringify,
            },
        );

        // HTTP functions
        self.environment.define(
            "http_get".to_string(),
            Value::BuiltinFn {
                name: "http_get".to_string(),
                arity: 1,
                func: builtins::builtin_http_get,
            },
        );

        self.environment.define(
            "http_post".to_string(),
            Value::BuiltinFn {
                name: "http_post".to_string(),
                arity: 2,
                func: builtins::builtin_http_post,
            },
        );

        // TCP networking
        self.environment.define(
            "tcp_listen".to_string(),
            Value::BuiltinFn {
                name: "tcp_listen".to_string(),
                arity: usize::MAX, // 1 or 2 args
                func: builtins::builtin_tcp_listen,
            },
        );
        self.environment.define(
            "tcp_connect".to_string(),
            Value::BuiltinFn {
                name: "tcp_connect".to_string(),
                arity: 1,
                func: builtins::builtin_tcp_connect,
            },
        );
        self.environment.define(
            "udp_bind".to_string(),
            Value::BuiltinFn {
                name: "udp_bind".to_string(),
                arity: 1,
                func: builtins::builtin_udp_bind,
            },
        );

        // set_workers(n) — registered as placeholder; handled by name in eval_call
        self.environment.define(
            "set_workers".to_string(),
            Value::BuiltinFn {
                name: "set_workers".to_string(),
                arity: 1,
                func: |_| Ok(Value::Null), // intercepted in eval_call before reaching here
            },
        );

        // on_ready(promise, callback) — intercepted in eval_call
        self.environment.define(
            "on_ready".to_string(),
            Value::BuiltinFn {
                name: "on_ready".to_string(),
                arity: 2,
                func: |_| Ok(Value::Null),
            },
        );

        // event_loop(?timeout_secs) — intercepted in eval_call
        self.environment.define(
            "event_loop".to_string(),
            Value::BuiltinFn {
                name: "event_loop".to_string(),
                arity: usize::MAX, // 0 or 1 args; validated in intercept
                func: |_| Ok(Value::Null),
            },
        );

        // set_queue_limit(n) — cap the event loop queue; intercepted in eval_call
        self.environment.define(
            "set_queue_limit".to_string(),
            Value::BuiltinFn {
                name: "set_queue_limit".to_string(),
                arity: 1,
                func: |_| Ok(Value::Null),
            },
        );

        // set_task_timeout(secs|null) — per-task deadline; intercepted in eval_call
        self.environment.define(
            "set_task_timeout".to_string(),
            Value::BuiltinFn {
                name: "set_task_timeout".to_string(),
                arity: 1,
                func: |_| Ok(Value::Null),
            },
        );

        // Promise module — provides Promise.all([p1, p2]) syntax
        use std::collections::HashMap as StdHashMap;
        use std::rc::Rc as StdRc;
        self.environment.define(
            "Promise".to_string(),
            Value::Module {
                name: "Promise".to_string(),
                members: StdRc::new(StdHashMap::new()), // all() is handled in eval_method_call
            },
        );
    }

    /// Load standard library modules
    fn load_stdlib(&mut self) {
        use super::stdlib;
        use crate::lexer::Scanner;
        use crate::parser::Parser;

        for (name, source) in stdlib::stdlib_modules() {
            let mut scanner = Scanner::new(source);
            let tokens = match scanner.scan_tokens() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to tokenize stdlib module '{}': {}",
                        name, e
                    );
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

            // Execute each module in an isolated evaluator so closures only
            // capture a small bootstrap environment, not the ever-growing main env.
            let mut module_eval = Evaluator::new_without_stdlib();
            for stmt in &program.statements {
                if let Err(e) = module_eval.exec_stmt(stmt) {
                    eprintln!("Warning: Failed to execute stdlib module '{}': {}", name, e);
                    break;
                }
            }

            // Copy only the newly defined names into the main environment
            let bindings: Vec<(String, Value)> = module_eval
                .environment
                .bindings()
                .iter()
                .filter(|(k, _)| !k.starts_with("__builtin"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            for (fname, fval) in bindings {
                self.environment.define(fname, fval);
            }
        }
    }

    /// Execute a single statement (public interface)
    pub fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        self.exec_stmt_internal(stmt)?;
        Ok(())
    }

    /// Execute a program (multiple statements)
    pub fn execute_program(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in statements {
            self.exec_stmt_internal(stmt)?;
        }
        Ok(())
    }

    /// Call the top-level main() function. Returns error if main is not defined or not a function.
    pub fn call_main(&mut self) -> Result<(), RuntimeError> {
        let main_val = self.environment.get("main").map_err(|_| {
            RuntimeError::InvalidOperation(
                "No main() function defined. Every Aether program must have a main() function."
                    .to_string(),
            )
        })?;

        match main_val {
            Value::Function {
                params,
                body,
                closure,
            } => {
                if !params.is_empty() {
                    return Err(RuntimeError::InvalidOperation(
                        "main() must take no arguments".to_string(),
                    ));
                }
                self.calls.depth += 1;
                self.calls.stack.push(StackFrame {
                    fn_name: "main".to_string(),
                    call_site_line: 0,
                    call_site_file: self.current_file_name(),
                });
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(_)) | Ok(_) => Ok(()),
                    Err(e) => Err(e),
                };
                self.environment = saved_env;
                self.calls.stack.pop();
                self.calls.depth -= 1;
                result?;
                // Auto-drain any on_ready callbacks registered but event_loop() never called.
                // Mirrors Node.js keeping the process alive for pending async work.
                if !self.async_rt.event_loop_queue.is_empty() {
                    self.run_event_loop(None)?;
                }
                Ok(())
            }
            _ => Err(RuntimeError::InvalidOperation(
                "main is not a function".to_string(),
            )),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
