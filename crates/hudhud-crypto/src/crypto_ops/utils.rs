//! Internal helpers for crypto builtins.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use sha2::{Digest, Sha256};

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn secure_random_bytes(count: usize) -> Vec<u8> {
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;
    let mut buf = vec![0u8; count];
    OsRng.fill_bytes(&mut buf);
    buf
}

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}

pub fn ensure_non_empty(method: &str, field: &str, value: &str) -> HudHudResult<()> {
    if value.is_empty() {
        return Err(runtime_error(format!(
            "{}: {} cannot be empty",
            method, field
        )));
    }
    Ok(())
}

pub fn ensure_max_bytes(
    method: &str,
    field: &str,
    value: &str,
    max_bytes: usize,
) -> HudHudResult<()> {
    if value.len() > max_bytes {
        return Err(runtime_error(format!(
            "{}: {} too large (max {} bytes)",
            method, field, max_bytes
        )));
    }
    Ok(())
}
