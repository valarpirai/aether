//! FFI plugin system for loading compiled Rust shared libraries

use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr};
use std::rc::Rc;

use libloading::{Library, Symbol};

use super::environment::RuntimeError;
use super::value::Value;

/// FFI-compatible function pointer type
/// Simple version for MVP: takes i64 args, returns i64 result
type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;

/// Loaded plugin with its library handle and function registry
#[derive(Debug)]
pub struct Plugin {
    #[allow(dead_code)]
    library: Library,
    functions: HashMap<String, PluginFnPtr>,
}

impl Plugin {
    /// Load a plugin from a shared library path
    pub fn load(path: &str) -> Result<Self, RuntimeError> {
        unsafe {
            let library = Library::new(path).map_err(|e| RuntimeError::IoError {
                operation: format!("load plugin '{}'", path),
                detail: e.to_string(),
            })?;

            let init_fn: Symbol<unsafe extern "C" fn() -> *const PluginMetadata> =
                library.get(b"aether_plugin_init").map_err(|e| {
                    RuntimeError::InvalidOperation(format!(
                        "Plugin '{}' missing aether_plugin_init: {}",
                        path, e
                    ))
                })?;

            let metadata_ptr = init_fn();
            if metadata_ptr.is_null() {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin '{}' returned null metadata",
                    path
                )));
            }

            let metadata = &*metadata_ptr;
            if metadata.version != 1 {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin '{}' version {} incompatible (expected 1)",
                    path, metadata.version
                )));
            }

            let functions = parse_function_descriptors(metadata);

            Ok(Plugin { library, functions })
        }
    }

    /// Call a plugin function by name (MVP: int-only args/return)
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let func = self
            .functions
            .get(name)
            .ok_or_else(|| RuntimeError::MethodNotFound {
                type_name: "plugin".to_string(),
                method: name.to_string(),
            })?;

        // Convert args to i64 array
        let int_args: Result<Vec<i64>, RuntimeError> = args
            .iter()
            .map(|v| match v {
                Value::Int(n) => Ok(*n),
                other => Err(RuntimeError::TypeError {
                    expected: "int".to_string(),
                    got: other.type_name().to_string(),
                }),
            })
            .collect();
        let int_args = int_args?;

        unsafe {
            let result = func(int_args.as_ptr(), int_args.len() as c_int);

            // i64::MIN signals error
            if result == i64::MIN {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin function '{}' failed (check arity/types)",
                    name
                )));
            }

            Ok(Value::Int(result))
        }
    }

    /// List available function names
    pub fn functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

/// Plugin metadata returned by aether_plugin_init
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const PluginFnPtr,
}

fn parse_function_descriptors(metadata: &PluginMetadata) -> HashMap<String, PluginFnPtr> {
    let mut functions = HashMap::new();

    unsafe {
        for i in 0..metadata.function_count as isize {
            let name_ptr = *metadata.function_names.offset(i);
            let func_ptr = *metadata.function_ptrs.offset(i);

            if !name_ptr.is_null() {
                if let Ok(name) = CStr::from_ptr(name_ptr).to_str() {
                    functions.insert(name.to_string(), func_ptr);
                }
            }
        }
    }

    functions
}

/// Plugin registry per interpreter instance
pub struct PluginRegistry {
    plugins: HashMap<String, Rc<Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
        }
    }

    /// Load and register a plugin
    pub fn load(&mut self, name: String, path: &str) -> Result<Rc<Plugin>, RuntimeError> {
        if let Some(plugin) = self.plugins.get(&name) {
            return Ok(Rc::clone(plugin));
        }

        let plugin = Rc::new(Plugin::load(path)?);
        self.plugins.insert(name.clone(), Rc::clone(&plugin));
        Ok(plugin)
    }

    /// Get a loaded plugin by name
    pub fn get(&self, name: &str) -> Option<Rc<Plugin>> {
        self.plugins.get(name).map(Rc::clone)
    }
}
