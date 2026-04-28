//! Expression evaluation and statement execution for the Aether interpreter

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::environment::{Environment, RuntimeError};
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
    Break,
    Continue,
}

/// Tree-walking interpreter for Aether programs
pub struct Evaluator {
    /// Current environment (variables in scope)
    pub environment: Environment,
    /// Current call depth (for recursion limit)
    call_depth: usize,
    /// Maximum allowed call depth
    max_call_depth: usize,
    /// Module cache to prevent re-execution on repeated imports
    module_cache: HashMap<String, Environment>,
    /// Tracks modules currently being loaded (for circular dependency detection)
    loading_stack: Vec<String>,
    /// Current file being executed (for relative imports)
    current_file: Option<PathBuf>,
    /// Optional I/O thread pool for async-native builtins (Phase 2)
    pub(crate) io_pool: Option<Arc<IoPool>>,
}

impl Evaluator {
    /// Create a new evaluator with a fresh environment (includes stdlib)
    pub fn new() -> Self {
        Self::new_with_stdlib()
    }

    /// Create a new evaluator with stdlib loaded
    pub fn new_with_stdlib() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
            call_depth: 0,
            max_call_depth: 100,
            module_cache: HashMap::new(),
            loading_stack: Vec::new(),
            current_file: None,
            io_pool: None,
        };
        evaluator.register_builtins();
        evaluator.load_stdlib();
        evaluator
    }

    /// Create a new evaluator without stdlib (faster for tests)
    pub fn new_without_stdlib() -> Self {
        let mut evaluator = Self {
            environment: Environment::new(),
            call_depth: 0,
            max_call_depth: 100,
            module_cache: HashMap::new(),
            loading_stack: Vec::new(),
            current_file: None,
            io_pool: None,
        };
        evaluator.register_builtins();
        evaluator
    }

    /// Create a new evaluator with an I/O thread pool (Phase 2)
    pub fn new_with_pool(workers: usize) -> Self {
        let mut evaluator = Self::new_with_stdlib();
        evaluator.io_pool = Some(Arc::new(IoPool::new(workers)));
        evaluator
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
                arity: 1,
                func: builtins::builtin_int,
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

        // Collection functions
        self.environment.define(
            "set".to_string(),
            Value::BuiltinFn {
                name: "set".to_string(),
                arity: 1,
                func: builtins::builtin_set,
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

        // set_workers(n) — registered as placeholder; handled by name in eval_call
        self.environment.define(
            "set_workers".to_string(),
            Value::BuiltinFn {
                name: "set_workers".to_string(),
                arity: 1,
                func: |_| Ok(Value::Null), // intercepted in eval_call before reaching here
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
            Value::Function { params, body, closure } => {
                if !params.is_empty() {
                    return Err(RuntimeError::InvalidOperation(
                        "main() must take no arguments".to_string(),
                    ));
                }
                self.call_depth += 1;
                let saved_env = self.environment.clone();
                self.environment = Environment::with_parent((*closure).clone());
                let result = match self.exec_stmt_internal(&body) {
                    Ok(ControlFlow::Return(_)) | Ok(_) => Ok(()),
                    Err(e) => Err(e),
                };
                self.environment = saved_env;
                self.call_depth -= 1;
                result
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
