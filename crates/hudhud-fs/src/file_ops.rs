//! Shared File I/O builtin — used by both VM and interpreter.
//!
//! Provides: file.read, file.write, file.append, file.delete, file.exists, file.list

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCode::RuntimeTypeError, message.into())
}

fn resource_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCode::RuntimeResourceError, message.into())
}

/// Execute a file method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    let get_path = |args: &[Value16]| -> HudHudResult<String> {
        match args.first().and_then(|v| v.as_str()) {
            Some(s) => Ok(s.to_string()),
            None => Err(type_error(format!(
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
                .map_err(|e| resource_error(format!("file.read('{}') failed: {}", path, e)))
        }
        "write" => {
            let path = get_path(args)?;
            let content = match args.get(1).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => return Err(type_error("file.write() requires string content")),
            };
            std::fs::write(&path, &content)
                .map(|_| Value16::bool_(true))
                .map_err(|e| resource_error(format!("file.write('{}') failed: {}", path, e)))
        }
        "append" => {
            use std::io::Write;
            let path = get_path(args)?;
            let content = match args.get(1).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => return Err(type_error("file.append() requires string content")),
            };
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(|e| resource_error(format!("file.append('{}') failed: {}", path, e)))?;
            f.write_all(content.as_bytes())
                .map(|_| Value16::bool_(true))
                .map_err(|e| resource_error(format!("file.append write failed: {}", e)))
        }
        "delete" => {
            let path = get_path(args)?;
            std::fs::remove_file(&path)
                .map(|_| Value16::bool_(true))
                .map_err(|e| resource_error(format!("file.delete('{}') failed: {}", path, e)))
        }
        "exists" => {
            let path = get_path(args)?;
            Ok(Value16::bool_(std::path::Path::new(&path).exists()))
        }
        "list" => {
            let path = get_path(args)?;
            let entries = std::fs::read_dir(&path)
                .map_err(|e| resource_error(format!("file.list('{}') failed: {}", path, e)))?;
            let names: Vec<Value16> = entries
                .filter_map(|e| e.ok())
                .map(|e| Value16::string(e.file_name().to_string_lossy().to_string()))
                .collect();
            Ok(Value16::array(names))
        }
        _ => Err(runtime_error(format!("Unknown file method: {}", method))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("hudhud-g10-{}-{}", name, std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn file_read_null_is_runtime_type_error() {
        let error = dispatch("read", &[Value16::null()]).unwrap_err();
        assert_eq!(error.code, ErrorCode::RuntimeTypeError);
    }

    #[test]
    fn file_read_number_is_runtime_type_error() {
        let error = dispatch("read", &[Value16::number(12.0)]).unwrap_err();
        assert_eq!(error.code, ErrorCode::RuntimeTypeError);
    }

    #[test]
    fn file_read_missing_is_runtime_resource_error() {
        let error = dispatch("read", &[Value16::string(temp_path("missing"))]).unwrap_err();
        assert_eq!(error.code, ErrorCode::RuntimeResourceError);
    }

    #[test]
    fn file_read_valid_fixture_is_unchanged() {
        let path = temp_path("fixture");
        std::fs::write(&path, "g10-content").unwrap();
        let value = dispatch("read", &[Value16::string(path)]).unwrap();
        assert_eq!(value.as_string(), Some("g10-content".to_string()));
    }

    #[test]
    fn file_write_non_string_content_is_runtime_type_error() {
        let error = dispatch(
            "write",
            &[Value16::string(temp_path("typed")), Value16::number(12.0)],
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::RuntimeTypeError);
    }
}
