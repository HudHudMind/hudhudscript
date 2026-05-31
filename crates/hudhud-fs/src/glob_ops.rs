//! Shared Glob pattern matching builtin — used by both VM and interpreter.

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

/// Execute a Glob method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "find" => {
            let pattern = args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error("Glob.find() expects a string argument"))?;

            let paths = glob::glob(pattern)
                .map_err(|e| runtime_error(format!("Glob.find error: {}", e)))?;

            let mut results = Vec::new();
            for entry in paths {
                match entry {
                    Ok(path) => {
                        results.push(Value16::string(path.to_string_lossy().to_string()));
                    }
                    Err(e) => {
                        return Err(runtime_error(format!("Glob.find error: {}", e)));
                    }
                }
            }
            Ok(Value16::array(results))
        }
        "match" => {
            if args.len() < 2 {
                return Err(runtime_error("Glob.match() requires 2 arguments"));
            }
            let input = args[0]
                .as_str()
                .ok_or_else(|| runtime_error("Glob.match: first argument must be a string"))?;
            let pattern = args[1]
                .as_str()
                .ok_or_else(|| runtime_error("Glob.match: second argument must be a string"))?;

            let compiled = glob::Pattern::new(pattern)
                .map_err(|e| runtime_error(format!("Glob.match error: {}", e)))?;

            Ok(Value16::bool_(compiled.matches(input)))
        }
        _ => Err(runtime_error(format!("Unknown Glob method: {}", method))),
    }
}
