//! X25519 key exchange primitives.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use crate::stdlib::e2e_ops::helpers::{
    compute_shared_secret, derive_public_key, hex_decode, secure_random_bytes,
};

pub fn e2e_generate_keypair(args: &[Value16]) -> SharedResult<Value16> {
    let _ = args;
    let private_bytes = secure_random_bytes(32)?;

    let mut clamped = private_bytes.clone();
    clamped[0] &= 248;
    clamped[31] &= 127;
    clamped[31] |= 64;

    let public_bytes = derive_public_key(&clamped);

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert(
        "public_key".to_string(),
        Value16::string(hex::encode(&public_bytes)),
    );
    result.insert(
        "private_key".to_string(),
        Value16::string(hex::encode(&clamped)),
    );
    Ok(Value16::object(result))
}

pub fn e2e_derive_shared_secret(args: &[Value16]) -> SharedResult<Value16> {
    let my_private_hex =
        crate::stdlib::e2e_ops::helpers::require_str(args, 0, "e2e.derive_shared_secret")?;
    let their_public_hex =
        crate::stdlib::e2e_ops::helpers::require_str(args, 1, "e2e.derive_shared_secret")?;

    let my_private = hex_decode(my_private_hex, "e2e.derive_shared_secret", "my_private")?;
    let their_public = hex_decode(their_public_hex, "e2e.derive_shared_secret", "their_public")?;

    if my_private.len() != 32 {
        return Err(runtime_error(
            "e2e.derive_shared_secret: private key must be 32 bytes (64 hex chars)",
        ));
    }
    if their_public.len() != 32 {
        return Err(runtime_error(
            "e2e.derive_shared_secret: public key must be 32 bytes (64 hex chars)",
        ));
    }

    let shared = compute_shared_secret(&my_private, &their_public);
    Ok(Value16::string(hex::encode(&shared)))
}
