//! Argon2id password hashing and verification.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::types::{MAX_HASH_STRING_BYTES, MAX_PASSWORD_BYTES};
use super::utils::{
    constant_time_eq, ensure_max_bytes, ensure_non_empty, require_str, runtime_error, sha256_bytes,
};

pub fn crypto_hash_password(args: &[Value16]) -> HudHudResult<Value16> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let password = require_str(args, 0, "crypto.hash_password")?;
    ensure_non_empty("crypto.hash_password", "password", password)?;
    ensure_max_bytes(
        "crypto.hash_password",
        "password",
        password,
        MAX_PASSWORD_BYTES,
    )?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| runtime_error(format!("crypto.hash_password: Argon2 error: {}", e)))?;

    Ok(Value16::string(hash.to_string()))
}

pub fn crypto_verify_password(args: &[Value16]) -> HudHudResult<Value16> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let password = require_str(args, 0, "crypto.verify_password")?;
    let stored = require_str(args, 1, "crypto.verify_password")?;
    ensure_non_empty("crypto.verify_password", "password", password)?;
    ensure_max_bytes(
        "crypto.verify_password",
        "password",
        password,
        MAX_PASSWORD_BYTES,
    )?;
    ensure_non_empty("crypto.verify_password", "hash", stored)?;
    ensure_max_bytes(
        "crypto.verify_password",
        "hash",
        stored,
        MAX_HASH_STRING_BYTES,
    )?;

    if stored.starts_with("$argon2") {
        let parsed_hash = PasswordHash::new(stored)
            .map_err(|e| runtime_error(format!("crypto.verify_password: invalid hash: {}", e)))?;
        let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
        Ok(Value16::bool_(result.is_ok()))
    } else if stored.starts_with("$hud$") {
        // Legacy: $hud$salt_hex$hash_hex (iterated SHA-256)
        let parts: Vec<&str> = stored.split('$').collect();
        if parts.len() != 4 {
            return Ok(Value16::bool_(false));
        }
        let salt = hex::decode(parts[2]).unwrap_or_default();
        let expected_hash = parts[3];

        let mut hash = sha256_bytes(&[salt.as_slice(), password.as_bytes()].concat());
        for _ in 0..99_999 {
            hash = sha256_bytes(&hash);
        }
        let computed = hex::encode(&hash);
        Ok(Value16::bool_(constant_time_eq(
            computed.as_bytes(),
            expected_hash.as_bytes(),
        )))
    } else {
        Err(runtime_error(
            "crypto.verify_password: unrecognized hash format (expected $argon2id$ or $hud$)",
        ))
    }
}
