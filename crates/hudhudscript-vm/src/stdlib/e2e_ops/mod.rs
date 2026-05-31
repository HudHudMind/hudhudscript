//! Shared e2e encryption builtins — X25519 ECDH, AES-like keystream encryption,
//! HMAC signatures. Matrix/Olm-style session primitives.
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! Uses audited `x25519-dalek` for Curve25519 scalar multiplication and
//! `sha2` for hashing + HMAC.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

mod encrypt;
mod helpers;
mod key_exchange;
mod session;
mod sign;

pub fn call_e2e_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "generate_keypair" => key_exchange::e2e_generate_keypair(args),
        "derive_shared_secret" => key_exchange::e2e_derive_shared_secret(args),
        "encrypt_message" => encrypt::e2e_encrypt_message(args),
        "decrypt_message" => encrypt::e2e_decrypt_message(args),
        "create_session" => session::e2e_create_session(args),
        "sign_message" => sign::e2e_sign_message(args),
        "verify_signature" => sign::e2e_verify_signature(args),
        _ => Err(runtime_error(format!("Unknown e2e method: {}", method))),
    }
}
