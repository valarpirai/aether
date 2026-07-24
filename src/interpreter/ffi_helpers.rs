//! FFI helper functions for plugin type conversion
//!
//! These functions are exported with C ABI so plugins can convert
//! between Aether Values and Rust types.
//!
//! # Safety
//!
//! All functions in this module are `unsafe` because they operate on raw pointers.
//! Callers must ensure:
//! - Pointers are valid and point to initialized Aether Values
//! - Pointers are not null (unless documented otherwise)
//! - Returned pointers are freed appropriately (via aether_value_free/aether_string_free)

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void, CString};

use super::value::Value;

/// Opaque pointer to an Aether Value
pub type AetherValuePtr = *const c_void;

// ============================================================================
// Value Type Checking
// ============================================================================

/// Check if a Value is an integer
#[no_mangle]
pub unsafe extern "C" fn aether_value_is_int(ptr: AetherValuePtr) -> bool {
    if ptr.is_null() {
        return false;
    }
    let value = &*(ptr as *const Value);
    matches!(value, Value::Int(_))
}

/// Check if a Value is a string
#[no_mangle]
pub unsafe extern "C" fn aether_value_is_string(ptr: AetherValuePtr) -> bool {
    if ptr.is_null() {
        return false;
    }
    let value = &*(ptr as *const Value);
    matches!(value, Value::String(_))
}

/// Check if a Value is an array
#[no_mangle]
pub unsafe extern "C" fn aether_value_is_array(ptr: AetherValuePtr) -> bool {
    if ptr.is_null() {
        return false;
    }
    let value = &*(ptr as *const Value);
    matches!(value, Value::Array(_))
}

/// Check if a Value is a dict
#[no_mangle]
pub unsafe extern "C" fn aether_value_is_dict(ptr: AetherValuePtr) -> bool {
    if ptr.is_null() {
        return false;
    }
    let value = &*(ptr as *const Value);
    matches!(value, Value::Dict(_))
}

/// Check if a Value is null
#[no_mangle]
pub unsafe extern "C" fn aether_value_is_null(ptr: AetherValuePtr) -> bool {
    if ptr.is_null() {
        return true;
    }
    let value = &*(ptr as *const Value);
    matches!(value, Value::Null)
}

// ============================================================================
// Value Extraction (Aether -> Rust)
// ============================================================================

/// Extract i64 from Value
#[no_mangle]
pub unsafe extern "C" fn aether_value_as_int(ptr: AetherValuePtr) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    let value = &*(ptr as *const Value);
    if let Value::Int(n) = value {
        *n
    } else {
        0
    }
}

/// Extract string from Value (returns owned CString that must be freed)
#[no_mangle]
pub unsafe extern "C" fn aether_value_as_string(ptr: AetherValuePtr) -> *mut c_char {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let value = &*(ptr as *const Value);
    if let Value::String(s) = value {
        match CString::new(s.as_ref().as_str()) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    } else {
        std::ptr::null_mut()
    }
}

/// Get array length
#[no_mangle]
pub unsafe extern "C" fn aether_value_array_len(ptr: AetherValuePtr) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    let value = &*(ptr as *const Value);
    if let Value::Array(arr) = value {
        arr.borrow().len() as c_int
    } else {
        0
    }
}

/// Get array element by index (returns borrowed pointer, valid until array is modified)
#[no_mangle]
pub unsafe extern "C" fn aether_value_array_get(
    ptr: AetherValuePtr,
    index: c_int,
) -> AetherValuePtr {
    if ptr.is_null() || index < 0 {
        return std::ptr::null();
    }
    let value = &*(ptr as *const Value);
    if let Value::Array(arr) = value {
        let borrowed = arr.borrow();
        if let Some(elem) = borrowed.get(index as usize) {
            // Return pointer to element (lifetime tied to borrow)
            elem as *const Value as AetherValuePtr
        } else {
            std::ptr::null()
        }
    } else {
        std::ptr::null()
    }
}

/// Get dict size
#[no_mangle]
pub unsafe extern "C" fn aether_value_dict_len(ptr: AetherValuePtr) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    let value = &*(ptr as *const Value);
    if let Value::Dict(dict) = value {
        dict.borrow().len() as c_int
    } else {
        0
    }
}

/// Get dict key at index (returns owned CString that must be freed).
///
/// Dicts are backed by an insertion-ordered `IndexMap`, so index access is
/// stable and lets plugins iterate every key: pair `aether_value_dict_len`
/// with this function, then look each key up via `aether_value_dict_get`.
/// Returns null if the value is not a dict, the index is out of range, or the
/// key is not a string.
#[no_mangle]
pub unsafe extern "C" fn aether_value_dict_key_at(
    ptr: AetherValuePtr,
    index: c_int,
) -> *mut c_char {
    if ptr.is_null() || index < 0 {
        return std::ptr::null_mut();
    }
    let value = &*(ptr as *const Value);
    if let Value::Dict(dict) = value {
        let borrowed = dict.borrow();
        if let Some((Value::String(s), _)) = borrowed.get_index(index as usize) {
            if let Ok(cs) = CString::new(s.as_ref().as_str()) {
                return cs.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

/// Get dict value by string key
#[no_mangle]
pub unsafe extern "C" fn aether_value_dict_get(
    ptr: AetherValuePtr,
    key: *const c_char,
) -> AetherValuePtr {
    if ptr.is_null() || key.is_null() {
        return std::ptr::null();
    }
    let value = &*(ptr as *const Value);
    if let Value::Dict(dict) = value {
        if let Ok(key_str) = std::ffi::CStr::from_ptr(key).to_str() {
            let key_value = Value::string(key_str.to_string());
            if let Some(val) = dict.borrow().get(&key_value) {
                val as *const Value as AetherValuePtr
            } else {
                std::ptr::null()
            }
        } else {
            std::ptr::null()
        }
    } else {
        std::ptr::null()
    }
}

// ============================================================================
// Value Creation (Rust -> Aether)
// ============================================================================

/// Create a new Int value (caller takes ownership)
#[no_mangle]
pub unsafe extern "C" fn aether_value_new_int(n: i64) -> AetherValuePtr {
    let value = Box::new(Value::Int(n));
    Box::into_raw(value) as AetherValuePtr
}

/// Create a new String value (caller takes ownership)
#[no_mangle]
pub unsafe extern "C" fn aether_value_new_string(s: *const c_char) -> AetherValuePtr {
    if s.is_null() {
        return Box::into_raw(Box::new(Value::Null)) as AetherValuePtr;
    }
    if let Ok(rust_str) = std::ffi::CStr::from_ptr(s).to_str() {
        let value = Box::new(Value::string(rust_str.to_string()));
        Box::into_raw(value) as AetherValuePtr
    } else {
        Box::into_raw(Box::new(Value::Null)) as AetherValuePtr
    }
}

/// Create a new empty Array value (caller takes ownership)
#[no_mangle]
pub unsafe extern "C" fn aether_value_new_array() -> AetherValuePtr {
    let value = Box::new(Value::array(Vec::new()));
    Box::into_raw(value) as AetherValuePtr
}

/// Push element to array (takes ownership of element)
#[no_mangle]
pub unsafe extern "C" fn aether_value_array_push(
    array_ptr: AetherValuePtr,
    elem_ptr: AetherValuePtr,
) -> bool {
    if array_ptr.is_null() || elem_ptr.is_null() {
        return false;
    }
    let array_value = &*(array_ptr as *const Value);
    let elem_value = Box::from_raw(elem_ptr as *mut Value);

    if let Value::Array(arr) = array_value {
        arr.borrow_mut().push(*elem_value);
        true
    } else {
        // Don't leak the element
        drop(elem_value);
        false
    }
}

/// Create a new empty Dict value (caller takes ownership)
#[no_mangle]
pub unsafe extern "C" fn aether_value_new_dict() -> AetherValuePtr {
    let value = Box::new(Value::dict(Vec::new()));
    Box::into_raw(value) as AetherValuePtr
}

/// Insert key-value pair into dict (takes ownership of value)
#[no_mangle]
pub unsafe extern "C" fn aether_value_dict_insert(
    dict_ptr: AetherValuePtr,
    key: *const c_char,
    value_ptr: AetherValuePtr,
) -> bool {
    if dict_ptr.is_null() || key.is_null() || value_ptr.is_null() {
        return false;
    }
    let dict_value = &*(dict_ptr as *const Value);
    let value = Box::from_raw(value_ptr as *mut Value);

    if let Value::Dict(dict) = dict_value {
        if let Ok(key_str) = std::ffi::CStr::from_ptr(key).to_str() {
            let key_value = Value::string(key_str.to_string());
            dict.borrow_mut().insert(key_value, *value);
            true
        } else {
            drop(value);
            false
        }
    } else {
        drop(value);
        false
    }
}

/// Create null value
#[no_mangle]
pub unsafe extern "C" fn aether_value_new_null() -> AetherValuePtr {
    let value = Box::new(Value::Null);
    Box::into_raw(value) as AetherValuePtr
}

/// Free a Value (called by plugins when done with a value they created)
#[no_mangle]
pub unsafe extern "C" fn aether_value_free(ptr: AetherValuePtr) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr as *mut Value));
    }
}

/// Free a C string (returned from aether_value_as_string)
#[no_mangle]
pub unsafe extern "C" fn aether_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
