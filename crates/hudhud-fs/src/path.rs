//! Shared Path builtins — used by both VM and interpreter.

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

/// Execute a Path method.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "join" => {
            let mut result = std::path::PathBuf::new();
            for arg in args {
                let s = arg
                    .as_str()
                    .ok_or_else(|| runtime_error("Path.join() requires string arguments"))?;
                result.push(s);
            }
            Ok(Value16::string(result.to_string_lossy().to_string()))
        }
        "parent" | "dirname" => {
            let s = get_str_arg(args, 0, method)?;
            match std::path::Path::new(s).parent() {
                Some(p) => Ok(Value16::string(p.to_string_lossy().to_string())),
                None => Ok(Value16::null()),
            }
        }
        "filename" | "basename" => {
            let s = get_str_arg(args, 0, method)?;
            match std::path::Path::new(s).file_name() {
                Some(n) => Ok(Value16::string(n.to_string_lossy().to_string())),
                None => Ok(Value16::null()),
            }
        }
        "extension" | "extname" => {
            let s = get_str_arg(args, 0, method)?;
            match std::path::Path::new(s).extension() {
                Some(e) => Ok(Value16::string(e.to_string_lossy().to_string())),
                None => Ok(Value16::null()),
            }
        }
        "resolve" | "normalize" => {
            let s = get_str_arg(args, 0, method)?;
            let p = std::path::Path::new(s);
            match std::fs::canonicalize(p) {
                Ok(abs) => Ok(Value16::string(abs.to_string_lossy().to_string())),
                Err(canon_err) => {
                    if p.is_absolute() {
                        Ok(Value16::string(s.to_string()))
                    } else {
                        let cwd = std::env::current_dir().map_err(|e| {
                            runtime_error(format!(
                                "Path.resolve: cannot canonicalize '{}' ({}) and cannot get current directory ({})",
                                s, canon_err, e
                            ))
                        })?;
                        Ok(Value16::string(cwd.join(p).to_string_lossy().to_string()))
                    }
                }
            }
        }
        "isAbsolute" | "is_absolute" => {
            let s = get_str_arg(args, 0, method)?;
            Ok(Value16::bool_(std::path::Path::new(s).is_absolute()))
        }
        "exists" => {
            let s = get_str_arg(args, 0, method)?;
            Ok(Value16::bool_(std::path::Path::new(s).exists()))
        }
        _ => Err(runtime_error(format!("Unknown Path method: {}", method))),
    }
}

fn get_str_arg<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    args.get(idx)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error(format!("Path.{}() requires a string argument", method)))
}
