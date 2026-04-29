//! Tests for built-in functions

use super::builtins::*;
use super::Value;

#[test]
fn test_builtin_print_returns_null() {
    let result = builtin_print(&[]).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_builtin_println_returns_null() {
    let result = builtin_println(&[]).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_builtin_print_multiple_args() {
    let args = vec![Value::Int(1), Value::string("hello"), Value::Bool(true)];
    let result = builtin_print(&args).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_builtin_len_string() {
    let result = builtin_len(&[Value::string("hello")]).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_builtin_len_array() {
    let arr = Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = builtin_len(&[arr]).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_builtin_type() {
    assert_eq!(
        builtin_type(&[Value::Int(42)]).unwrap(),
        Value::string("int")
    );
    assert_eq!(
        builtin_type(&[Value::Float(3.14)]).unwrap(),
        Value::string("float")
    );
    assert_eq!(
        builtin_type(&[Value::Bool(true)]).unwrap(),
        Value::string("bool")
    );
    assert_eq!(
        builtin_type(&[Value::string("hi")]).unwrap(),
        Value::string("string")
    );
}

#[test]
fn test_builtin_int_conversions() {
    assert_eq!(builtin_int(&[Value::Int(42)]).unwrap(), Value::Int(42));
    assert_eq!(builtin_int(&[Value::Float(3.7)]).unwrap(), Value::Int(3));
    assert_eq!(
        builtin_int(&[Value::string("123")]).unwrap(),
        Value::Int(123)
    );
    assert_eq!(builtin_int(&[Value::Bool(true)]).unwrap(), Value::Int(1));
    assert_eq!(builtin_int(&[Value::Bool(false)]).unwrap(), Value::Int(0));
}

#[test]
fn test_builtin_float_conversions() {
    assert_eq!(
        builtin_float(&[Value::Int(42)]).unwrap(),
        Value::Float(42.0)
    );
    assert_eq!(
        builtin_float(&[Value::Float(3.14)]).unwrap(),
        Value::Float(3.14)
    );
    assert_eq!(
        builtin_float(&[Value::string("3.14")]).unwrap(),
        Value::Float(3.14)
    );
}

#[test]
fn test_builtin_str_conversions() {
    assert_eq!(builtin_str(&[Value::Int(42)]).unwrap(), Value::string("42"));
    assert_eq!(
        builtin_str(&[Value::Bool(true)]).unwrap(),
        Value::string("true")
    );
    assert_eq!(builtin_str(&[Value::Null]).unwrap(), Value::string("null"));
}

#[test]
fn test_builtin_bool_conversions() {
    assert_eq!(builtin_bool(&[Value::Int(0)]).unwrap(), Value::Bool(false));
    assert_eq!(builtin_bool(&[Value::Int(42)]).unwrap(), Value::Bool(true));
    assert_eq!(
        builtin_bool(&[Value::string("")]).unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        builtin_bool(&[Value::string("hi")]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(builtin_bool(&[Value::Null]).unwrap(), Value::Bool(false));
}

// --- arity and type errors ---

#[test]
fn test_builtin_len_type_error() {
    let err = builtin_len(&[Value::Int(5)]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::TypeError { .. }));
}

#[test]
fn test_builtin_len_arity_error() {
    let err = builtin_len(&[]).unwrap_err();
    assert!(matches!(
        err,
        super::RuntimeError::ArityMismatch {
            expected: 1,
            got: 0
        }
    ));
}

#[test]
fn test_builtin_int_invalid_string() {
    let err = builtin_int(&[Value::string("abc")]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::ConversionError { .. }));
}

#[test]
fn test_builtin_int_type_error() {
    let err = builtin_int(&[Value::Null]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::TypeError { .. }));
}

#[test]
fn test_builtin_float_invalid_string() {
    let err = builtin_float(&[Value::string("xyz")]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::ConversionError { .. }));
}

#[test]
fn test_builtin_type_arity_error() {
    let err = builtin_type(&[]).unwrap_err();
    assert!(matches!(
        err,
        super::RuntimeError::ArityMismatch {
            expected: 1,
            got: 0
        }
    ));
}

// --- len edge cases ---

#[test]
fn test_builtin_len_empty_string() {
    let result = builtin_len(&[Value::string("")]).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_builtin_len_empty_array() {
    let result = builtin_len(&[Value::array(vec![])]).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_builtin_len_unicode_string() {
    // len counts bytes, not chars for Rc<String>
    let result = builtin_len(&[Value::string("abc")]).unwrap();
    assert_eq!(result, Value::Int(3));
}

// --- json_parse / json_stringify ---

#[test]
fn test_builtin_json_parse_object() {
    let result = builtin_json_parse(&[Value::string(r#"{"x": 1, "y": 2}"#)]).unwrap();
    assert!(matches!(result, Value::Dict(_)));
}

#[test]
fn test_builtin_json_parse_array() {
    let result = builtin_json_parse(&[Value::string("[1, 2, 3]")]).unwrap();
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn test_builtin_json_parse_invalid() {
    let err = builtin_json_parse(&[Value::string("{bad json}")]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::ParseError { .. }));
}

#[test]
fn test_builtin_json_stringify_int() {
    let result = builtin_json_stringify(&[Value::Int(42)]).unwrap();
    assert_eq!(result, Value::string("42"));
}

#[test]
fn test_builtin_json_stringify_null() {
    let result = builtin_json_stringify(&[Value::Null]).unwrap();
    assert_eq!(result, Value::string("null"));
}

#[test]
fn test_builtin_json_stringify_bool() {
    assert_eq!(
        builtin_json_stringify(&[Value::Bool(true)]).unwrap(),
        Value::string("true")
    );
    assert_eq!(
        builtin_json_stringify(&[Value::Bool(false)]).unwrap(),
        Value::string("false")
    );
}

// --- clock ---

#[test]
fn test_builtin_clock_returns_float() {
    let result = builtin_clock(&[]).unwrap();
    assert!(matches!(result, Value::Float(f) if f > 0.0));
}

#[test]
fn test_builtin_clock_arity_error() {
    let err = builtin_clock(&[Value::Int(1)]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::ArityMismatch { .. }));
}

// --- file builtins ---

#[test]
fn test_builtin_read_file_missing() {
    let err = builtin_read_file(&[Value::string("/nonexistent_aether_xyz.txt")]).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("read_file"), "msg={}", msg);
    assert!(msg.contains("nonexistent_aether_xyz"), "msg={}", msg);
}

#[test]
fn test_builtin_file_exists_false() {
    let result = builtin_file_exists(&[Value::string("/nonexistent_xyz_12345.txt")]).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_builtin_is_file_false_for_dir() {
    let result = builtin_is_file(&[Value::string("/tmp")]).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_builtin_is_dir_true_for_tmp() {
    let result = builtin_is_dir(&[Value::string("/tmp")]).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_builtin_path_join_basic() {
    let result = builtin_path_join(&[Value::string("/tmp"), Value::string("file.txt")]).unwrap();
    assert_eq!(result, Value::string("/tmp/file.txt"));
}

#[test]
fn test_builtin_path_join_arity_error() {
    let err = builtin_path_join(&[Value::string("/tmp")]).unwrap_err();
    assert!(matches!(err, super::RuntimeError::ArityMismatch { .. }));
}

#[test]
fn test_builtin_list_dir_missing() {
    let err = builtin_list_dir(&[Value::string("/nonexistent_dir_xyz")]).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("list_dir"), "msg={}", msg);
}
