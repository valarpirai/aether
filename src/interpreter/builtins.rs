//! Built-in functions for Aether

use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    io_pool::{build_http_client_with_opts, HttpOptions},
    RuntimeError, Value,
};
use serde_json::Value as JsonValue;

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
    std::fs::read_to_string(&path)
        .map(Value::string)
        .map_err(|e| RuntimeError::InvalidOperation(format!("read_file '{}': {}", path, e)))
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
        .map_err(|e| RuntimeError::InvalidOperation(format!("write_file '{}': {}", path, e)))
}

/// Built-in function: read_lines(path)
/// Reads a file and returns an array of strings, one per line (newlines stripped)
pub fn builtin_read_lines(args: &[Value]) -> Result<Value, RuntimeError> {
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
    let content = std::fs::read_to_string(&path)
        .map_err(|e| RuntimeError::InvalidOperation(format!("read_lines '{}': {}", path, e)))?;
    let lines: Vec<Value> = content
        .lines()
        .map(|l| Value::string(l.to_string()))
        .collect();
    Ok(Value::array(lines))
}

/// Built-in function: append_file(path, content)
/// Appends a string to a file, creating it if it does not exist
pub fn builtin_append_file(args: &[Value]) -> Result<Value, RuntimeError> {
    use std::io::Write;
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
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|e| RuntimeError::InvalidOperation(format!("append_file '{}': {}", path, e)))?;
    file.write_all(content.as_bytes())
        .map(|_| Value::Null)
        .map_err(|e| RuntimeError::InvalidOperation(format!("append_file '{}': {}", path, e)))
}

/// Built-in function: file_exists(path) -> bool
pub fn builtin_file_exists(args: &[Value]) -> Result<Value, RuntimeError> {
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
    match std::fs::metadata(&path) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(RuntimeError::InvalidOperation(format!(
            "file_exists '{}': {}",
            path, e
        ))),
    }
}

/// Built-in function: mkdir(path) — creates directory recursively
pub fn builtin_mkdir(args: &[Value]) -> Result<Value, RuntimeError> {
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
    std::fs::create_dir_all(&path)
        .map(|_| Value::Null)
        .map_err(|e| RuntimeError::InvalidOperation(format!("mkdir '{}': {}", path, e)))
}

/// Built-in function: is_file(path) -> bool
pub fn builtin_is_file(args: &[Value]) -> Result<Value, RuntimeError> {
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
    match std::fs::metadata(&path) {
        Ok(m) => Ok(Value::Bool(m.is_file())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(RuntimeError::InvalidOperation(format!(
            "is_file '{}': {}",
            path, e
        ))),
    }
}

/// Built-in function: is_dir(path) -> bool
pub fn builtin_is_dir(args: &[Value]) -> Result<Value, RuntimeError> {
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
    match std::fs::metadata(&path) {
        Ok(m) => Ok(Value::Bool(m.is_dir())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(RuntimeError::InvalidOperation(format!(
            "is_dir '{}': {}",
            path, e
        ))),
    }
}

/// Built-in function: lines_iter(path) -> file_lines iterator
/// Returns a lazy iterator that reads one line at a time without loading the whole file
pub fn builtin_lines_iter(args: &[Value]) -> Result<Value, RuntimeError> {
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
    Value::file_lines(&path)
}

/// Built-in function: read_bytes(path) -> array of ints (0–255)
pub fn builtin_read_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
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
    let bytes = std::fs::read(&path)
        .map_err(|e| RuntimeError::InvalidOperation(format!("read_bytes '{}': {}", path, e)))?;
    let arr: Vec<Value> = bytes.iter().map(|&b| Value::Int(b as i64)).collect();
    Ok(Value::array(arr))
}

/// Built-in function: write_bytes(path, bytes_array)
/// Writes an array of ints (0–255) as raw bytes to a file
pub fn builtin_write_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
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
    let elements = match &args[1] {
        Value::Array(a) => a.clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let mut bytes = Vec::with_capacity(elements.len());
    for (i, v) in elements.iter().enumerate() {
        match v {
            Value::Int(n) if *n >= 0 && *n <= 255 => bytes.push(*n as u8),
            Value::Int(n) => {
                return Err(RuntimeError::InvalidOperation(format!(
                    "write_bytes: byte at index {} is {} (must be 0–255)",
                    i, n
                )))
            }
            other => {
                return Err(RuntimeError::TypeError {
                    expected: "int (0–255)".to_string(),
                    got: other.type_name().to_string(),
                })
            }
        }
    }
    std::fs::write(&path, &bytes)
        .map(|_| Value::Null)
        .map_err(|e| RuntimeError::InvalidOperation(format!("write_bytes '{}': {}", path, e)))
}

/// Built-in function: list_dir(path) -> array of filenames (not full paths)
pub fn builtin_list_dir(args: &[Value]) -> Result<Value, RuntimeError> {
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
    let entries = std::fs::read_dir(&path)
        .map_err(|e| RuntimeError::InvalidOperation(format!("list_dir '{}': {}", path, e)))?;
    let mut names: Vec<Value> = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| RuntimeError::InvalidOperation(format!("list_dir '{}': {}", path, e)))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        names.push(Value::string(name));
    }
    names.sort_by(|a, b| format!("{}", a).cmp(&format!("{}", b)));
    Ok(Value::array(names))
}

/// Built-in function: path_join(a, b) -> joined path string
pub fn builtin_path_join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::ArityMismatch {
            expected: 2,
            got: args.len(),
        });
    }
    let mut path = std::path::PathBuf::new();
    for arg in args {
        match arg {
            Value::String(s) => path.push(s.as_ref().as_str()),
            other => {
                return Err(RuntimeError::TypeError {
                    expected: "string".to_string(),
                    got: other.type_name().to_string(),
                })
            }
        }
    }
    Ok(Value::string(path.to_string_lossy().into_owned()))
}

/// Built-in function: rename(src, dst) — move or rename a file/directory
pub fn builtin_rename(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            expected: 2,
            got: args.len(),
        });
    }
    let src = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let dst = match &args[1] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    std::fs::rename(&src, &dst)
        .map(|_| Value::Null)
        .map_err(|e| {
            RuntimeError::InvalidOperation(format!("rename '{}' -> '{}': {}", src, dst, e))
        })
}

/// Built-in function: rm(path) — remove a file (use rmdir for directories)
pub fn builtin_rm(args: &[Value]) -> Result<Value, RuntimeError> {
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
    std::fs::remove_file(&path)
        .map(|_| Value::Null)
        .map_err(|e| RuntimeError::InvalidOperation(format!("rm '{}': {}", path, e)))
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
    Ok(Value::string(
        line.trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string(),
    ))
}

/// Built-in function: json_parse(string) -> value
pub fn builtin_json_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    let s = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let json: JsonValue = serde_json::from_str(&s)
        .map_err(|e| RuntimeError::InvalidOperation(format!("json_parse error: {}", e)))?;
    json_to_value(json)
}

fn json_to_value(json: JsonValue) -> Result<Value, RuntimeError> {
    match json {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(b) => Ok(Value::Bool(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else {
                Ok(Value::Float(n.as_f64().unwrap_or(0.0)))
            }
        }
        JsonValue::String(s) => Ok(Value::string(s)),
        JsonValue::Array(arr) => {
            let values: Result<Vec<Value>, RuntimeError> =
                arr.into_iter().map(json_to_value).collect();
            Ok(Value::array(values?))
        }
        JsonValue::Object(map) => {
            let pairs: Result<Vec<(Value, Value)>, RuntimeError> = map
                .into_iter()
                .map(|(k, v)| Ok((Value::string(k), json_to_value(v)?)))
                .collect();
            Ok(Value::Dict(Rc::new(pairs?)))
        }
    }
}

/// Built-in function: json_stringify(value) -> string
pub fn builtin_json_stringify(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    let json = value_to_json(&args[0])?;
    Ok(Value::string(json.to_string()))
}

fn value_to_json(value: &Value) -> Result<JsonValue, RuntimeError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Int(n) => Ok(JsonValue::Number((*n).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .ok_or_else(|| {
                RuntimeError::InvalidOperation(
                    "cannot serialize non-finite float to JSON".to_string(),
                )
            }),
        Value::String(s) => Ok(JsonValue::String(s.as_ref().clone())),
        Value::Array(arr) => {
            let values: Result<Vec<JsonValue>, RuntimeError> =
                arr.iter().map(value_to_json).collect();
            Ok(JsonValue::Array(values?))
        }
        Value::Dict(pairs) => {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs.iter() {
                let key = match k {
                    Value::String(s) => s.as_ref().clone(),
                    other => {
                        return Err(RuntimeError::InvalidOperation(format!(
                            "json_stringify: dict key must be string, got {}",
                            other.type_name()
                        )))
                    }
                };
                map.insert(key, value_to_json(v)?);
            }
            Ok(JsonValue::Object(map))
        }
        other => Err(RuntimeError::InvalidOperation(format!(
            "json_stringify: cannot serialize {}",
            other.type_name()
        ))),
    }
}

/// Built-in function: clock() -> float
/// Returns seconds since Unix epoch as a float.
pub fn builtin_clock(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            expected: 0,
            got: args.len(),
        });
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs_f64();
    Ok(Value::Float(secs))
}

/// Parse an optional config dict into `HttpOptions`.
/// Accepted keys: `timeout` (int seconds), `user_agent` (string).
pub fn parse_http_opts(val: &Value) -> Result<HttpOptions, RuntimeError> {
    let pairs = match val {
        Value::Dict(p) => p.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "dict".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };

    fn get<'a>(pairs: &'a [(Value, Value)], key: &str) -> Option<&'a Value> {
        pairs.iter().find_map(|(k, v)| match k {
            Value::String(s) if s.as_ref() == key => Some(v),
            _ => None,
        })
    }

    let timeout_secs = match get(&pairs, "timeout") {
        Some(Value::Int(n)) => Some(*n as u64),
        Some(Value::Float(f)) => Some(*f as u64),
        Some(other) => {
            return Err(RuntimeError::TypeError {
                expected: "int".to_string(),
                got: other.type_name().to_string(),
            })
        }
        None => None,
    };
    let user_agent = match get(&pairs, "user_agent") {
        Some(Value::String(s)) => Some(s.as_ref().clone()),
        Some(other) => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
        None => None,
    };
    Ok(HttpOptions {
        timeout_secs,
        user_agent,
    })
}

/// Built-in function: http_get(url [, opts]) -> string
pub fn builtin_http_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    let url = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let opts = if args.len() == 2 {
        parse_http_opts(&args[1])?
    } else {
        HttpOptions::default()
    };
    let body = build_http_client_with_opts(&opts)
        .get(&url)
        .send()
        .map_err(|e| RuntimeError::InvalidOperation(format!("http_get failed: {}", e)))?
        .text()
        .map_err(|e| RuntimeError::InvalidOperation(format!("http_get read failed: {}", e)))?;
    Ok(Value::string(body))
}

/// Built-in function: http_post(url, body [, opts]) -> string
pub fn builtin_http_post(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(RuntimeError::ArityMismatch {
            expected: 2,
            got: args.len(),
        });
    }
    let url = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        other => {
            return Err(RuntimeError::TypeError {
                expected: "string".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    let body = format!("{}", args[1]);
    let opts = if args.len() == 3 {
        parse_http_opts(&args[2])?
    } else {
        HttpOptions::default()
    };
    let response_body = build_http_client_with_opts(&opts)
        .post(&url)
        .body(body)
        .send()
        .map_err(|e| RuntimeError::InvalidOperation(format!("http_post failed: {}", e)))?
        .text()
        .map_err(|e| RuntimeError::InvalidOperation(format!("http_post read failed: {}", e)))?;
    Ok(Value::string(response_body))
}

/// Built-in function: sleep(seconds)
/// Pauses execution for the given number of seconds (int or float).
pub fn builtin_sleep(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }
    let secs = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        other => {
            return Err(RuntimeError::TypeError {
                expected: "int or float".to_string(),
                got: other.type_name().to_string(),
            })
        }
    };
    if secs > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(secs));
    }
    Ok(Value::Null)
}

/// Built-in function: set(array)
/// Creates a set from an array (removes duplicates, only hashable values allowed)
pub fn builtin_set(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::Array(arr) => {
            // False positive: our Hash impl only hashes immutable data
            #[allow(clippy::mutable_key_type)]
            let mut set = HashSet::new();
            for value in arr.iter() {
                if !value.is_hashable() {
                    return Err(RuntimeError::TypeError {
                        expected: "hashable type (int, float, string, bool, null)".to_string(),
                        got: format!("{} (not hashable)", value.type_name()),
                    });
                }
                set.insert(value.clone());
            }
            Ok(Value::set(set))
        }
        other => Err(RuntimeError::TypeError {
            expected: "array".to_string(),
            got: other.type_name().to_string(),
        }),
    }
}
