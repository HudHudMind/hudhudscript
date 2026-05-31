//! HMAC-based message signatures.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

use crate::stdlib::e2e_ops::helpers::{
    constant_time_eq, derive_public_key, hex_decode, hmac_sha512, require_str, sha256_bytes,
};

pub fn e2e_sign_message(args: &[Value16]) -> SharedResult<Value16> {
    let message = require_str(args, 0, "e2e.sign_message")?;
    let private_key_hex = require_str(args, 1, "e2e.sign_message")?;

    let private_key = hex_decode(private_key_hex, "e2e.sign_message", "private_key")?;

    if private_key.len() != 32 {
        return Err(runtime_error(
            "e2e.sign_message: private key must be 32 bytes (64 hex chars)",
        ));
    }

    let public_key = derive_public_key(&private_key);
    let sign_key = sha256_bytes(&public_key);
    let signature = hmac_sha512(&sign_key, message.as_bytes());
    Ok(Value16::string(hex::encode(&signature)))
}

pub fn e2e_verify_signature(args: &[Value16]) -> SharedResult<Value16> {
    let message = require_str(args, 0, "e2e.verify_signature")?;
    let signature_hex = require_str(args, 1, "e2e.verify_signature")?;
    let public_key_hex = require_str(args, 2, "e2e.verify_signature")?;

    let signature = hex_decode(signature_hex, "e2e.verify_signature", "signature")?;
    let public_key = hex_decode(public_key_hex, "e2e.verify_signature", "public_key")?;

    if public_key.len() != 32 {
        return Err(runtime_error(
            "e2e.verify_signature: public key must be 32 bytes (64 hex chars)",
        ));
    }
    if signature.len() != 64 {
        return Err(runtime_error(
            "e2e.verify_signature: signature must be 64 bytes (128 hex chars)",
        ));
    }

    let verify_key = sha256_bytes(&public_key);
    let expected = hmac_sha512(&verify_key, message.as_bytes());

    Ok(Value16::boolean(constant_time_eq(&signature, &expected)))
}
