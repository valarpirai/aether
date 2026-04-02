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
