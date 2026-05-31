//! AES-256-GCM-style encrypt/decrypt using keystream + HMAC.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use crate::stdlib::e2e_ops::helpers::{
    constant_time_eq, generate_keystream, hex_decode, hmac_sha256, require_str,
    secure_random_bytes, sha256_bytes,
};

pub fn e2e_encrypt_message(args: &[Value16]) -> SharedResult<Value16> {
    let plaintext = require_str(args, 0, "e2e.encrypt_message")?;
    let shared_secret_hex = require_str(args, 1, "e2e.encrypt_message")?;

    let shared_secret = hex_decode(shared_secret_hex, "e2e.encrypt_message", "shared_secret")?;
    let key = sha256_bytes(&shared_secret);
    let nonce = secure_random_bytes(12)?;

    let pt_bytes = plaintext.as_bytes();
    let keystream = generate_keystream(&key, &nonce, pt_bytes.len());
    let ciphertext: Vec<u8> = pt_bytes
        .iter()
        .zip(keystream.iter())
        .map(|(p, k)| p ^ k)
        .collect();

    let mut tag_input = Vec::with_capacity(nonce.len() + ciphertext.len());
    tag_input.extend_from_slice(&nonce);
    tag_input.extend_from_slice(&ciphertext);
    let tag = hmac_sha256(&key, &tag_input);

    let mut ct_with_tag = ciphertext;
    ct_with_tag.extend_from_slice(&tag);

    let mut result = HashMap::new();
    result.insert(
        "ciphertext".to_string(),
        Value16::string(hex::encode(&ct_with_tag)),
    );
    result.insert("nonce".to_string(), Value16::string(hex::encode(&nonce)));
    Ok(Value16::object(result))
}

pub fn e2e_decrypt_message(args: &[Value16]) -> SharedResult<Value16> {
    let ciphertext_hex = require_str(args, 0, "e2e.decrypt_message")?;
    let nonce_hex = require_str(args, 1, "e2e.decrypt_message")?;
    let shared_secret_hex = require_str(args, 2, "e2e.decrypt_message")?;

    let ct_with_tag = hex_decode(ciphertext_hex, "e2e.decrypt_message", "ciphertext")?;
    let nonce = hex_decode(nonce_hex, "e2e.decrypt_message", "nonce")?;
    let shared_secret = hex_decode(shared_secret_hex, "e2e.decrypt_message", "shared_secret")?;

    if nonce.len() != 12 {
        return Err(runtime_error(
            "e2e.decrypt_message: nonce must be 12 bytes (24 hex chars)",
        ));
    }
    if ct_with_tag.len() < 32 {
        return Err(runtime_error("e2e.decrypt_message: ciphertext too short"));
    }

    let ciphertext = &ct_with_tag[..ct_with_tag.len() - 32];
    let tag = &ct_with_tag[ct_with_tag.len() - 32..];

    let key = sha256_bytes(&shared_secret);

    let mut tag_input = Vec::with_capacity(nonce.len() + ciphertext.len());
    tag_input.extend_from_slice(&nonce);
    tag_input.extend_from_slice(ciphertext);
    let expected_tag = hmac_sha256(&key, &tag_input);

    if !constant_time_eq(tag, &expected_tag) {
        return Err(runtime_error(
            "e2e.decrypt_message: authentication failed — invalid key or corrupted data",
        ));
    }

    let keystream = generate_keystream(&key, &nonce, ciphertext.len());
    let plaintext: Vec<u8> = ciphertext
        .iter()
        .zip(keystream.iter())
        .map(|(c, k)| c ^ k)
        .collect();

    let text = String::from_utf8(plaintext)
        .map_err(|e| runtime_error(format!("e2e.decrypt_message: invalid UTF-8 output: {}", e)))?;

    Ok(Value16::string(text))
}
