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

/// V1 protocol function pointer (backward compat)
pub type PluginFnPtr = unsafe extern "C" fn(*const i64, c_int) -> i64;

/// Plugin metadata structure
#[repr(C)]
pub struct PluginMetadata {
    pub version: c_int,
    pub function_count: c_int,
    pub function_names: *const *const c_char,
    pub function_ptrs: *const *const std::ffi::c_void, // Generic pointers
}

/// Macro to generate V1 plugin initialization function (integer-only)
///
/// Use this for plugins with only i64 parameters and returns.
///
/// # Example
///
/// ```rust
/// use aether_plugin::*;
///
/// #[aether_export]
/// fn add(a: i64, b: i64) -> i64 { a + b }
///
/// aether_plugin_init!(add);
/// ```
#[macro_export]
macro_rules! aether_plugin_init {
    ($($fn_name:ident),+ $(,)?) => {
        static FUNC_NAMES_STORAGE: &[&[u8]] = &[
            $(concat!(stringify!($fn_name), "\0").as_bytes()),+
        ];

        static FUNC_PTRS: &[*const ::std::ffi::c_void] = &[
            $($crate::paste::paste! { [<$fn_name _ffi>] } as *const ::std::ffi::c_void),+
        ];

        #[no_mangle]
        pub extern "C" fn aether_plugin_init() -> *const $crate::PluginMetadata {
            let name_ptrs: Vec<*const ::std::ffi::c_char> = FUNC_NAMES_STORAGE
                .iter()
                .map(|s| s.as_ptr() as *const ::std::ffi::c_char)
                .collect();

            let names_box = Box::leak(name_ptrs.into_boxed_slice());

            Box::into_raw(Box::new($crate::PluginMetadata {
                version: 1,  // V1 protocol
                function_count: FUNC_NAMES_STORAGE.len() as ::std::ffi::c_int,
                function_names: names_box.as_ptr(),
                function_ptrs: FUNC_PTRS.as_ptr(),
            }))
        }
    };
}

/// Macro to generate V2 plugin initialization function (complex types)
///
/// Use this for plugins with String, Vec, or HashMap parameters/returns.
///
/// # Example
///
/// ```rust
/// use aether_plugin::*;
///
/// #[aether_export]
/// fn greet(name: String) -> String { format!("Hello, {}!", name) }
///
/// aether_plugin_init_v2!(greet);
/// ```
#[macro_export]
macro_rules! aether_plugin_init_v2 {
    ($($fn_name:ident),+ $(,)?) => {
        // Function pointers are safe to share across threads
        unsafe impl Sync for FuncPtrsV2Wrapper {}
        struct FuncPtrsV2Wrapper([*const ::std::ffi::c_void; count_tokens!($($fn_name)*)]);

        static FUNC_NAMES_STORAGE_V2: &[&[u8]] = &[
            $(concat!(stringify!($fn_name), "\0").as_bytes()),+
        ];

        static FUNC_PTRS_V2: FuncPtrsV2Wrapper = FuncPtrsV2Wrapper([
            $($crate::paste::paste! { [<$fn_name _ffi>] } as *const ::std::ffi::c_void),+
        ]);

        #[no_mangle]
        pub extern "C" fn aether_plugin_init_v2() -> *const $crate::PluginMetadata {
            let name_ptrs: Vec<*const ::std::ffi::c_char> = FUNC_NAMES_STORAGE_V2
                .iter()
                .map(|s| s.as_ptr() as *const ::std::ffi::c_char)
                .collect();

            let names_box = Box::leak(name_ptrs.into_boxed_slice());

            Box::into_raw(Box::new($crate::PluginMetadata {
                version: 2,  // V2 protocol
                function_count: FUNC_NAMES_STORAGE_V2.len() as ::std::ffi::c_int,
                function_names: names_box.as_ptr(),
                function_ptrs: FUNC_PTRS_V2.0.as_ptr(),
            }))
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count_tokens {
    () => { 0 };
    ($head:ident $($tail:ident)*) => { 1 + count_tokens!($($tail)*) };
}
