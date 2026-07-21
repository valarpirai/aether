//! FFI plugin system for loading compiled Rust shared libraries

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::rc::Rc;

use libloading::{Library, Symbol};

use super::environment::RuntimeError;
use super::value::Value;

/// FFI-compatible function pointer type (V1 protocol - i64 only)
type PluginFnPtrV1 = unsafe extern "C" fn(*const i64, c_int) -> i64;

/// FFI-compatible function pointer type (V2 protocol - complex types)
type PluginFnPtrV2 =
    unsafe extern "C" fn(*const *const c_void, c_int, *mut *const c_void) -> *const c_void;

/// Function protocol version
#[derive(Debug, Clone, Copy)]
enum FunctionProtocol {
    V1, // Integer-only
    V2, // Complex types
}

/// Function entry with protocol information
#[derive(Clone)]
struct FunctionEntry {
    protocol: FunctionProtocol,
    v1_ptr: Option<PluginFnPtrV1>,
    v2_ptr: Option<PluginFnPtrV2>,
}

impl std::fmt::Debug for FunctionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionEntry")
            .field("protocol", &self.protocol)
            .field("v1_ptr", &self.v1_ptr.map(|_| "Some(fn)"))
            .field("v2_ptr", &self.v2_ptr.map(|_| "Some(fn)"))
            .finish()
    }
}

/// Loaded plugin with its library handle and function registry
pub struct Plugin {
    #[allow(dead_code)]
    library: Library,
    functions: HashMap<String, FunctionEntry>,
}

impl std::fmt::Debug for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugin")
            .field("functions", &self.functions.keys())
            .finish()
    }
}

impl Plugin {
    /// Load a plugin from a shared library path
    pub fn load(path: &str) -> Result<Self, RuntimeError> {
        unsafe {
            let library = Library::new(path).map_err(|e| RuntimeError::IoError {
                operation: format!("load plugin '{}'", path),
                detail: e.to_string(),
            })?;

            // Try V2 init first, then fall back to V1
            let (metadata_ptr, protocol_version) = if let Ok(init_fn_v2) =
                library.get::<Symbol<unsafe extern "C" fn() -> *const PluginMetadata>>(
                    b"aether_plugin_init_v2",
                ) {
                let ptr = init_fn_v2();
                (ptr, FunctionProtocol::V2)
            } else if let Ok(init_fn_v1) = library.get::<Symbol<
                unsafe extern "C" fn() -> *const PluginMetadata,
            >>(b"aether_plugin_init")
            {
                let ptr = init_fn_v1();
                (ptr, FunctionProtocol::V1)
            } else {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin '{}' missing aether_plugin_init or aether_plugin_init_v2",
                    path
                )));
            };

            if metadata_ptr.is_null() {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin '{}' returned null metadata",
                    path
                )));
            }

            let metadata = &*metadata_ptr;
            if metadata.version != 1 && metadata.version != 2 {
                return Err(RuntimeError::InvalidOperation(format!(
                    "Plugin '{}' version {} incompatible (expected 1 or 2)",
                    path, metadata.version
                )));
            }

            let functions = parse_function_descriptors(metadata, protocol_version);

            Ok(Plugin { library, functions })
        }
    }

    /// Call a plugin function by name
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let entry = self
            .functions
            .get(name)
            .ok_or_else(|| RuntimeError::MethodNotFound {
                type_name: "plugin".to_string(),
                method: name.to_string(),
            })?;

        match entry.protocol {
            FunctionProtocol::V1 => self.call_v1(entry, args),
            FunctionProtocol::V2 => self.call_v2(entry, args),
        }
    }

    /// Call a V1 (integer-only) function
    fn call_v1(&self, entry: &FunctionEntry, args: &[Value]) -> Result<Value, RuntimeError> {
        let func = entry
            .v1_ptr
            .ok_or_else(|| RuntimeError::InvalidOperation("V1 function missing".to_string()))?;

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
                return Err(RuntimeError::InvalidOperation(
                    "Plugin function failed (check arity/types)".to_string(),
                ));
            }

            Ok(Value::Int(result))
        }
    }

    /// Call a V2 (complex types) function
    fn call_v2(&self, entry: &FunctionEntry, args: &[Value]) -> Result<Value, RuntimeError> {
        let func = entry
            .v2_ptr
            .ok_or_else(|| RuntimeError::InvalidOperation("V2 function missing".to_string()))?;

        unsafe {
            // Convert args to AetherValuePtr array (raw pointers to Value)
            let value_ptrs: Vec<*const c_void> = args
                .iter()
                .map(|v| v as *const Value as *const c_void)
                .collect();

            let mut error_ptr: *const c_void = std::ptr::null();

            let result_ptr = func(
                value_ptrs.as_ptr(),
                value_ptrs.len() as c_int,
                &mut error_ptr as *mut *const c_void,
            );

            // Check for error
            if !error_ptr.is_null() {
                let error_value = Box::from_raw(error_ptr as *mut Value);
                let error_msg = format!("Plugin error: {}", error_value);
                drop(error_value);
                return Err(RuntimeError::InvalidOperation(error_msg));
            }

            // Check for null result
            if result_ptr.is_null() {
                return Err(RuntimeError::InvalidOperation(
                    "Plugin returned null without error".to_string(),
                ));
            }

            // Take ownership of the returned Value
            let result_value = Box::from_raw(result_ptr as *mut Value);
            Ok(*result_value)
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
    pub function_ptrs: *const *const c_void, // Generic pointer, cast based on protocol
}

fn parse_function_descriptors(
    metadata: &PluginMetadata,
    protocol: FunctionProtocol,
) -> HashMap<String, FunctionEntry> {
    let mut functions = HashMap::new();

    unsafe {
        for i in 0..metadata.function_count as isize {
            let name_ptr = *metadata.function_names.offset(i);
            let func_ptr = *metadata.function_ptrs.offset(i);

            if !name_ptr.is_null() {
                if let Ok(name) = CStr::from_ptr(name_ptr).to_str() {
                    let entry = match protocol {
                        FunctionProtocol::V1 => FunctionEntry {
                            protocol: FunctionProtocol::V1,
                            v1_ptr: Some(std::mem::transmute::<*const c_void, PluginFnPtrV1>(
                                func_ptr,
                            )),
                            v2_ptr: None,
                        },
                        FunctionProtocol::V2 => FunctionEntry {
                            protocol: FunctionProtocol::V2,
                            v1_ptr: None,
                            v2_ptr: Some(std::mem::transmute::<*const c_void, PluginFnPtrV2>(
                                func_ptr,
                            )),
                        },
                    };
                    functions.insert(name.to_string(), entry);
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
