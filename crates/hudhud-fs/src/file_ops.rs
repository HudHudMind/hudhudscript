//! Shared File I/O builtin — used by both VM and interpreter.
//!
//! Provides: file.read, file.write, file.append, file.delete, file.exists, file.list

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Execute a file method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    let get_path = |args: &[Value16]| -> HudHudResult<String> {
        match args.first().and_then(|v| v.as_str()) {
            Some(s) => Ok(s.to_string()),
            None => Err(runtime_error(format!(
                "file.{}() requires a string path",
                method
            ))),
        }
    };

    match method {
        "read" => {
            let path = get_path(args)?;
            std::fs::read_to_string(&path)
                .map(Value16::string)
                .map_err(|e| runtime_error(format!("file.read('{}') failed: {}", path, e)))
        }
        "write" => {
            let path = get_path(args)?;
            let content = match args.get(1).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    return Err(runtime_error(
                        "file.write() requires string content".to_string(),
                    ))
                }
            };
            std::fs::write(&path, &content)
                .map(|_| Value16::bool_(true))
                .map_err(|e| runtime_error(format!("file.write('{}') failed: {}", path, e)))
        }
        "append" => {
            use std::io::Write;
            let path = get_path(args)?;
            let content = match args.get(1).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    return Err(runtime_error(
                        "file.append() requires string content".to_string(),
                    ))
                }
            };
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(|e| runtime_error(format!("file.append('{}') failed: {}", path, e)))?;
            f.write_all(content.as_bytes())
                .map(|_| Value16::bool_(true))
                .map_err(|e| runtime_error(format!("file.append write failed: {}", e)))
        }
        "delete" => {
            let path = get_path(args)?;
            std::fs::remove_file(&path)
                .map(|_| Value16::bool_(true))
                .map_err(|e| runtime_error(format!("file.delete('{}') failed: {}", path, e)))
        }
        "exists" => {
            let path = get_path(args)?;
            Ok(Value16::bool_(std::path::Path::new(&path).exists()))
        }
        "list" => {
            let path = get_path(args)?;
            let entries = std::fs::read_dir(&path)
                .map_err(|e| runtime_error(format!("file.list('{}') failed: {}", path, e)))?;
            let names: Vec<Value16> = entries
                .filter_map(|e| e.ok())
                .map(|e| Value16::string(e.file_name().to_string_lossy().to_string()))
                .collect();
            Ok(Value16::array(names))
        }
        _ => Err(runtime_error(format!("Unknown file method: {}", method))),
    }
}
