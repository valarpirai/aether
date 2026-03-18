//! Built-in functions for Aether

use super::{RuntimeError, Value};

/// Built-in function: print(...values)
/// Prints values to stdout without a newline
pub fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    Ok(Value::Null)
}

/// Built-in function: println(...values)
/// Prints values to stdout with a newline
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let args = vec![Value::Int(1), Value::String("hello".to_string()), Value::Bool(true)];
        let result = builtin_print(&args).unwrap();
        assert_eq!(result, Value::Null);
    }
}
