//! AES-256-GCM AEAD encryption / decryption.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::types::{MAX_CIPHER_B64_BYTES, MAX_CIPHER_DATA_BYTES, MAX_CRYPTO_KEY_BYTES};
use super::utils::{ensure_max_bytes, ensure_non_empty, require_str, runtime_error, sha256_bytes};

pub fn crypto_encrypt(args: &[Value16]) -> HudHudResult<Value16> {
    use aes_gcm::{
        aead::{Aead, OsRng},
        AeadCore, Aes256Gcm, Key, KeyInit,
    };

    let data = require_str(args, 0, "crypto.encrypt")?;
    let key_str = require_str(args, 1, "crypto.encrypt")?;
    ensure_max_bytes("crypto.encrypt", "data", data, MAX_CIPHER_DATA_BYTES)?;
    ensure_non_empty("crypto.encrypt", "key", key_str)?;
    ensure_max_bytes("crypto.encrypt", "key", key_str, MAX_CRYPTO_KEY_BYTES)?;

    let key_bytes = sha256_bytes(key_str.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data.as_bytes().as_ref())
        .map_err(|e| runtime_error(format!("crypto.encrypt: AES-GCM error: {}", e)))?;

    let mut output = Vec::with_capacity(nonce.len() + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&output);
    Ok(Value16::string(encoded))
}

pub fn crypto_decrypt(args: &[Value16]) -> HudHudResult<Value16> {
    use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};

    let ciphertext_b64 = require_str(args, 0, "crypto.decrypt")?;
    let key_str = require_str(args, 1, "crypto.decrypt")?;
    ensure_max_bytes(
        "crypto.decrypt",
        "ciphertext",
        ciphertext_b64,
        MAX_CIPHER_B64_BYTES,
    )?;
    ensure_non_empty("crypto.decrypt", "key", key_str)?;
    ensure_max_bytes("crypto.decrypt", "key", key_str, MAX_CRYPTO_KEY_BYTES)?;

    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64.as_bytes())
        .map_err(|e| runtime_error(format!("crypto.decrypt: invalid base64: {}", e)))?;

    if raw.len() < 12 {
        return Err(runtime_error(
            "crypto.decrypt: ciphertext too short (need at least 12 bytes for nonce)",
        ));
    }

    let nonce = Nonce::from_slice(&raw[..12]);
    let ciphertext = &raw[12..];

    let key_bytes = sha256_bytes(key_str.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        runtime_error("crypto.decrypt: authentication failed — invalid key or corrupted data")
    })?;

    let text = String::from_utf8(plaintext)
        .map_err(|e| runtime_error(format!("crypto.decrypt: invalid UTF-8: {}", e)))?;
    Ok(Value16::string(text))
}
