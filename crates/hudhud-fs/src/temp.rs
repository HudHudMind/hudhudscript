//! Shared Temp file/directory builtin — used by both VM and interpreter.

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
use std::collections::HashMap;

/// Execute a Temp method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "file" => {
            let prefix = args.first().and_then(|v| v.as_str().map(|s| s.to_string()));

            let file = if let Some(pfx) = prefix {
                tempfile::Builder::new()
                    .prefix(&pfx)
                    .tempfile()
                    .map_err(|e| runtime_error(format!("Temp.file error: {}", e)))?
            } else {
                tempfile::NamedTempFile::new()
                    .map_err(|e| runtime_error(format!("Temp.file error: {}", e)))?
            };

            let path = file.path().to_string_lossy().to_string();
            let _ = file.into_temp_path();

            let mut result = hudhudscript_bytecode::ObjMap::default();
            result.insert("path".to_string(), Value16::string(path));
            Ok(Value16::object(result))
        }
        "dir" => {
            let prefix = args.first().and_then(|v| v.as_str().map(|s| s.to_string()));

            let dir = if let Some(pfx) = prefix {
                tempfile::Builder::new()
                    .prefix(&pfx)
                    .tempdir()
                    .map_err(|e| runtime_error(format!("Temp.dir error: {}", e)))?
            } else {
                tempfile::tempdir().map_err(|e| runtime_error(format!("Temp.dir error: {}", e)))?
            };

            let path = dir.path().to_string_lossy().to_string();
            let _ = dir.keep();

            let mut result = hudhudscript_bytecode::ObjMap::default();
            result.insert("path".to_string(), Value16::string(path));
            Ok(Value16::object(result))
        }
        "path" => {
            let prefix = args
                .first()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "hudhud".to_string());

            let dir = std::env::temp_dir();
            let unique = format!(
                "{}{:x}",
                prefix,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            Ok(Value16::string(
                dir.join(unique).to_string_lossy().to_string(),
            ))
        }
        _ => Err(runtime_error(format!("Unknown Temp method: {}", method))),
    }
}
