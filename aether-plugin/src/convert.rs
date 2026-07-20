//! Type conversion between Aether Values and Rust types
//!
//! This module provides traits and implementations for converting
//! between Aether's runtime Value type and native Rust types.

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr, CString};

/// Opaque pointer to an Aether Value
pub type AetherValuePtr = *const c_void;

// ============================================================================
// External FFI functions (provided by Aether interpreter)
// ============================================================================

extern "C" {
    pub fn aether_value_is_int(ptr: AetherValuePtr) -> bool;
    pub fn aether_value_is_string(ptr: AetherValuePtr) -> bool;
    pub fn aether_value_is_array(ptr: AetherValuePtr) -> bool;
    pub fn aether_value_is_dict(ptr: AetherValuePtr) -> bool;
    pub fn aether_value_is_null(ptr: AetherValuePtr) -> bool;

    pub fn aether_value_as_int(ptr: AetherValuePtr) -> i64;
    pub fn aether_value_as_string(ptr: AetherValuePtr) -> *mut c_char;
    pub fn aether_value_array_len(ptr: AetherValuePtr) -> c_int;
    pub fn aether_value_array_get(ptr: AetherValuePtr, index: c_int) -> AetherValuePtr;
    pub fn aether_value_dict_len(ptr: AetherValuePtr) -> c_int;
    pub fn aether_value_dict_get(ptr: AetherValuePtr, key: *const c_char) -> AetherValuePtr;

    pub fn aether_value_new_int(n: i64) -> AetherValuePtr;
    pub fn aether_value_new_string(s: *const c_char) -> AetherValuePtr;
    pub fn aether_value_new_array() -> AetherValuePtr;
    pub fn aether_value_array_push(array: AetherValuePtr, elem: AetherValuePtr) -> bool;
    pub fn aether_value_new_dict() -> AetherValuePtr;
    pub fn aether_value_dict_insert(
        dict: AetherValuePtr,
        key: *const c_char,
        value: AetherValuePtr,
    ) -> bool;
    pub fn aether_value_new_null() -> AetherValuePtr;
    pub fn aether_value_free(ptr: AetherValuePtr);
    pub fn aether_string_free(s: *mut c_char);
}

// ============================================================================
// Conversion Traits
// ============================================================================

/// Convert from Aether Value to Rust type
pub trait FromAether: Sized {
    unsafe fn from_aether(ptr: AetherValuePtr) -> Result<Self, String>;
}

/// Convert from Rust type to Aether Value
pub trait ToAether {
    unsafe fn to_aether(self) -> AetherValuePtr;
}

// ============================================================================
// Implementations for primitive types
// ============================================================================

impl FromAether for i64 {
    unsafe fn from_aether(ptr: AetherValuePtr) -> Result<Self, String> {
        if aether_value_is_int(ptr) {
            Ok(aether_value_as_int(ptr))
        } else {
            Err("Expected int".to_string())
        }
    }
}

impl ToAether for i64 {
    unsafe fn to_aether(self) -> AetherValuePtr {
        aether_value_new_int(self)
    }
}

impl FromAether for String {
    unsafe fn from_aether(ptr: AetherValuePtr) -> Result<Self, String> {
        if aether_value_is_string(ptr) {
            let c_str = aether_value_as_string(ptr);
            if c_str.is_null() {
                return Err("Failed to extract string".to_string());
            }
            let result = CStr::from_ptr(c_str)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| "Invalid UTF-8".to_string());
            aether_string_free(c_str);
            result
        } else {
            Err("Expected string".to_string())
        }
    }
}

impl ToAether for String {
    unsafe fn to_aether(self) -> AetherValuePtr {
        let c_string = match CString::new(self) {
            Ok(cs) => cs,
            Err(_) => return aether_value_new_null(),
        };
        aether_value_new_string(c_string.as_ptr())
    }
}

impl FromAether for Vec<i64> {
    unsafe fn from_aether(ptr: AetherValuePtr) -> Result<Self, String> {
        if !aether_value_is_array(ptr) {
            return Err("Expected array".to_string());
        }

        let len = aether_value_array_len(ptr) as usize;
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let elem_ptr = aether_value_array_get(ptr, i as c_int);
            if elem_ptr.is_null() {
                return Err(format!("Array element {} is null", i));
            }
            let elem = i64::from_aether(elem_ptr)?;
            result.push(elem);
        }

        Ok(result)
    }
}

impl ToAether for Vec<i64> {
    unsafe fn to_aether(self) -> AetherValuePtr {
        let array_ptr = aether_value_new_array();
        for elem in self {
            let elem_ptr = elem.to_aether();
            if !aether_value_array_push(array_ptr, elem_ptr) {
                // Failed to push, free what we created
                aether_value_free(array_ptr);
                return aether_value_new_null();
            }
        }
        array_ptr
    }
}

impl FromAether for HashMap<String, i64> {
    unsafe fn from_aether(_ptr: AetherValuePtr) -> Result<Self, String> {
        // TODO: Implement dict iteration in FFI helpers
        Err("Dict iteration not yet implemented".to_string())
    }
}

impl ToAether for HashMap<String, i64> {
    unsafe fn to_aether(self) -> AetherValuePtr {
        let dict_ptr = aether_value_new_dict();
        for (key, value) in self {
            let key_c = match CString::new(key) {
                Ok(cs) => cs,
                Err(_) => {
                    aether_value_free(dict_ptr);
                    return aether_value_new_null();
                }
            };
            let value_ptr = value.to_aether();
            if !aether_value_dict_insert(dict_ptr, key_c.as_ptr(), value_ptr) {
                aether_value_free(dict_ptr);
                return aether_value_new_null();
            }
        }
        dict_ptr
    }
}

// ============================================================================
// Option and Result support
// ============================================================================

impl<T: ToAether> ToAether for Option<T> {
    unsafe fn to_aether(self) -> AetherValuePtr {
        match self {
            Some(value) => value.to_aether(),
            None => aether_value_new_null(),
        }
    }
}

impl<T: ToAether, E: ToString> ToAether for Result<T, E> {
    unsafe fn to_aether(self) -> AetherValuePtr {
        match self {
            Ok(value) => value.to_aether(),
            Err(_) => aether_value_new_null(), // Error details handled separately
        }
    }
}
