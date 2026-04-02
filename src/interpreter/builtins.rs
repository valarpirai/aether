//! Built-in functions for Aether

use std::rc::Rc;

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

/// Built-in function: len(collection)
/// Returns the length of a string or array
pub fn builtin_len(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
        other => Err(RuntimeError::TypeError {
            expected: "string or array".to_string(),
            got: other.type_name().to_string(),
        }),
    }
}

/// Built-in function: type(value)
/// Returns the type name of a value as a string
pub fn builtin_type(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    Ok(Value::String(Rc::new(args[0].type_name().to_string())))
}

/// Built-in function: int(value)
/// Converts a value to an integer
pub fn builtin_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| RuntimeError::InvalidOperation(format!("Cannot convert '{}' to int", s))),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        other => Err(RuntimeError::TypeError {
            expected: "number, string, or bool".to_string(),
            got: other.type_name().to_string(),
        }),
    }
}

/// Built-in function: float(value)
/// Converts a value to a float
pub fn builtin_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            RuntimeError::InvalidOperation(format!("Cannot convert '{}' to float", s))
        }),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        other => Err(RuntimeError::TypeError {
            expected: "number, string, or bool".to_string(),
            got: other.type_name().to_string(),
        }),
    }
}

/// Built-in function: str(value)
/// Converts a value to a string
pub fn builtin_str(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    Ok(Value::String(Rc::new(format!("{}", args[0]))))
}

/// Built-in function: bool(value)
/// Converts a value to a boolean using truthiness rules
pub fn builtin_bool(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    Ok(Value::Bool(args[0].is_truthy()))
}

/// Built-in function: read_file(path)
/// Reads a file and returns its contents as a string
pub fn builtin_read_file(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    std::fs::read_to_string(&path).map(Value::string).map_err(|e| {
        RuntimeError::InvalidOperation(format!("read_file failed: {}", e))
    })
}

/// Built-in function: write_file(path, content)
/// Writes a string to a file, creating or overwriting it
pub fn builtin_write_file(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            expected: 2,
            got: args.len(),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let content = format!("{}", args[1]);
    std::fs::write(&path, content)
        .map(|_| Value::Null)
        .map_err(|e| RuntimeError::InvalidOperation(format!("write_file failed: {}", e)))
}

/// Built-in function: input(prompt)
/// Reads a line from stdin, printing an optional prompt first
pub fn builtin_input(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() > 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    if let Some(prompt) = args.first() {
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| RuntimeError::InvalidOperation(format!("input failed: {}", e)))?;
    Ok(Value::string(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
}
