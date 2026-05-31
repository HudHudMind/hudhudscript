use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(ErrorCode::RuntimeTypeError, format!("{}: expected {}, got {}", context, expected, got))
}

pub fn require_str<'a>(
    args: &'a [Value16],
    idx: usize,
    method: &str,
) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(type_error("string", "missing", method)),
    }
}
