//! Runtime support for Aether FFI plugins
//!
//! This crate provides the `#[aether_export]` macro and supporting types
//! for creating Aether plugins with minimal boilerplate.
//!
//! # Example
//!
//! ```rust
//! use aether_plugin::*;
//!
//! #[aether_export]
//! fn add(a: i64, b: i64) -> i64 {
//!     a + b
//! }
//!
//! #[aether_export]
//! fn multiply(a: i64, b: i64) -> i64 {
//!     a * b
//! }
//!
//! // Auto-generated registration
//! aether_plugin_init!(add, multiply);
//! ```

pub use aether_plugin_macro::aether_export;

// Re-export paste for use in the macro
#[doc(hidden)]
pub use paste;

// Re-export conversion traits and types
pub mod convert;
pub use convert::{AetherValuePtr, FromAether, ToAether};

use std::ffi::c_char;
use std::ffi::c_int;

/// FFI-compatible function pointer type
pub type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;

/// Plugin metadata structure
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const PluginFnPtr,
}

/// Macro to generate plugin initialization function
///
/// # Example
///
/// ```rust
/// use aether_plugin::*;
///
/// #[aether_export]
/// fn add(a: i64, b: i64) -> i64 { a + b }
///
/// #[aether_export]
/// fn sub(a: i64, b: i64) -> i64 { a - b }
///
/// aether_plugin_init!(add, sub);
/// ```
#[macro_export]
macro_rules! aether_plugin_init {
    ($($fn_name:ident),+ $(,)?) => {
        static FUNC_NAMES_STORAGE: &[&[u8]] = &[
            $(concat!(stringify!($fn_name), "\0").as_bytes()),+
        ];

        static FUNC_PTRS: &[$crate::PluginFnPtr] = &[
            $($crate::paste::paste! { [<$fn_name _ffi>] }),+
        ];

        #[no_mangle]
        pub extern "C" fn aether_plugin_init() -> *const $crate::PluginMetadata {
            let name_ptrs: Vec<*const std::ffi::c_char> = FUNC_NAMES_STORAGE
                .iter()
                .map(|s| s.as_ptr() as *const std::ffi::c_char)
                .collect();

            let names_box = Box::leak(name_ptrs.into_boxed_slice());

            Box::into_raw(Box::new($crate::PluginMetadata {
                version: 1,
                function_count: FUNC_NAMES_STORAGE.len() as std::ffi::c_int,
                function_names: names_box.as_ptr(),
                function_ptrs: FUNC_PTRS.as_ptr(),
            }))
        }
    };
}
