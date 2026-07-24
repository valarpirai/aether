//! Example Aether plugin demonstrating FFI
//!
//! This is a manual implementation showing the plugin protocol.
//! For MVP, plugins work with simple i64 values only.

use std::ffi::{c_char, c_int, CString};
use std::ptr;

/// FFI-compatible function pointer type
/// Takes array of i64 args, returns i64 result (or negative for error)
type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;

/// Plugin metadata structure
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const PluginFnPtr,
}

/// Static function names
static FUNC_NAMES_STORAGE: &[&[u8]] = &[b"add\0", b"multiply\0", b"power\0", b"is_even\0"];

/// Static function pointers
static FUNC_PTRS: &[PluginFnPtr] = &[add_impl, multiply_impl, power_impl, is_even_impl];

/// Plugin initialization function
#[no_mangle]
pub extern "C" fn aether_plugin_init() -> *const PluginMetadata {
    // Convert to C pointers
    let name_ptrs: Vec<*const c_char> = FUNC_NAMES_STORAGE
        .iter()
        .map(|s| s.as_ptr() as *const c_char)
        .collect();

    let names_box = Box::leak(name_ptrs.into_boxed_slice());

    let metadata = PluginMetadata {
        version: 1,
        function_count: FUNC_NAMES_STORAGE.len() as c_int,
        function_names: names_box.as_ptr(),
        function_ptrs: FUNC_PTRS.as_ptr(),
    };

    Box::into_raw(Box::new(metadata))
}

/// add(a, b) -> a + b
unsafe extern "C" fn add_impl(args: *const i64, argc: c_int) -> i64 {
    if argc != 2 {
        return i64::MIN; // Error: wrong arity
    }
    let a = *args.offset(0);
    let b = *args.offset(1);
    a.wrapping_add(b)
}

/// multiply(a, b) -> a * b
unsafe extern "C" fn multiply_impl(args: *const i64, argc: c_int) -> i64 {
    if argc != 2 {
        return i64::MIN;
    }
    let a = *args.offset(0);
    let b = *args.offset(1);
    a.wrapping_mul(b)
}

/// power(base, exp) -> base^exp (returns 0 for negative exp)
unsafe extern "C" fn power_impl(args: *const i64, argc: c_int) -> i64 {
    if argc != 2 {
        return i64::MIN;
    }
    let base = *args.offset(0);
    let exp = *args.offset(1);

    if exp < 0 {
        return 0; // Can't do negative exponents with integers
    }

    (base as f64).powi(exp as i32) as i64
}

/// is_even(n) -> 1 if even, 0 if odd
unsafe extern "C" fn is_even_impl(args: *const i64, argc: c_int) -> i64 {
    if argc != 1 {
        return i64::MIN;
    }
    let n = *args.offset(0);
    if n % 2 == 0 { 1 } else { 0 }
}
