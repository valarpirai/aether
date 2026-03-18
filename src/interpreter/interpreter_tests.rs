//! Tests for the interpreter module

use super::environment::{Environment, RuntimeError};
use super::value::Value;

// Value tests
#[test]
fn test_value_creation() {
    let int_val = Value::Int(42);
    let float_val = Value::Float(3.14);
    let string_val = Value::String("hello".to_string());
    let bool_val = Value::Bool(true);
    let null_val = Value::Null;

    assert_eq!(int_val, Value::Int(42));
    assert_eq!(float_val, Value::Float(3.14));
    assert_eq!(string_val, Value::String("hello".to_string()));
    assert_eq!(bool_val, Value::Bool(true));
    assert_eq!(null_val, Value::Null);
}

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Int(42)), "42");
    assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
    assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::Null), "null");
    assert_eq!(
        format!("{}", Value::Array(vec![Value::Int(1), Value::Int(2)])),
        "[1, 2]"
    );
}

#[test]
fn test_value_is_truthy() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(!Value::Null.is_truthy());
    assert!(!Value::Int(0).is_truthy());
    assert!(Value::Int(1).is_truthy());
    assert!(!Value::Float(0.0).is_truthy());
    assert!(Value::Float(1.0).is_truthy());
    assert!(!Value::String("".to_string()).is_truthy());
    assert!(Value::String("hello".to_string()).is_truthy());
    assert!(!Value::Array(vec![]).is_truthy());
    assert!(Value::Array(vec![Value::Int(1)]).is_truthy());
}

#[test]
fn test_value_type_name() {
    assert_eq!(Value::Int(42).type_name(), "int");
    assert_eq!(Value::Float(3.14).type_name(), "float");
    assert_eq!(Value::String("hello".to_string()).type_name(), "string");
    assert_eq!(Value::Bool(true).type_name(), "bool");
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::Array(vec![]).type_name(), "array");
}

// Environment tests
#[test]
fn test_environment_define_and_get() {
    let mut env = Environment::new();
    env.define("x".to_string(), Value::Int(42));

    let value = env.get("x").unwrap();
    assert_eq!(value, Value::Int(42));
}

#[test]
fn test_environment_undefined_variable() {
    let env = Environment::new();
    let result = env.get("x");
    assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
}

#[test]
fn test_environment_set_existing() {
    let mut env = Environment::new();
    env.define("x".to_string(), Value::Int(42));
    env.set("x", Value::Int(100)).unwrap();

    let value = env.get("x").unwrap();
    assert_eq!(value, Value::Int(100));
}

#[test]
fn test_environment_set_undefined() {
    let mut env = Environment::new();
    let result = env.set("x", Value::Int(42));
    assert!(matches!(result, Err(RuntimeError::UndefinedVariable(_))));
}

#[test]
fn test_environment_nested_scopes() {
    let mut global = Environment::new();
    global.define("x".to_string(), Value::Int(10));

    let mut local = Environment::with_parent(global.clone());
    local.define("y".to_string(), Value::Int(20));

    // Can access both local and parent variables
    assert_eq!(local.get("y").unwrap(), Value::Int(20));
    assert_eq!(local.get("x").unwrap(), Value::Int(10));

    // Parent cannot access child variables
    assert!(global.get("y").is_err());
}

#[test]
fn test_environment_shadowing() {
    let mut global = Environment::new();
    global.define("x".to_string(), Value::Int(10));

    let mut local = Environment::with_parent(global.clone());
    local.define("x".to_string(), Value::Int(20));

    // Local shadows global
    assert_eq!(local.get("x").unwrap(), Value::Int(20));
    assert_eq!(global.get("x").unwrap(), Value::Int(10));
}
