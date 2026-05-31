//! crypto.hmac implementation (RFC 2104 over sha2).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use sha2::{Digest, Sha256, Sha512};

use super::types::{MAX_CRYPTO_DATA_BYTES, MAX_CRYPTO_KEY_BYTES};
use super::utils::{ensure_max_bytes, ensure_non_empty, require_str, runtime_error};

pub fn crypto_hmac(args: &[Value16]) -> HudHudResult<Value16> {
    let algorithm = require_str(args, 0, "crypto.hmac")?;
    let key = require_str(args, 1, "crypto.hmac")?;
    let data = require_str(args, 2, "crypto.hmac")?;
    ensure_non_empty("crypto.hmac", "key", key)?;
    ensure_max_bytes("crypto.hmac", "key", key, MAX_CRYPTO_KEY_BYTES)?;
    ensure_max_bytes("crypto.hmac", "data", data, MAX_CRYPTO_DATA_BYTES)?;

    let result = match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" | "hmac-sha256" => {
            hmac_sha2::<Sha256>(key.as_bytes(), data.as_bytes(), 64)
        }
        "sha512" | "sha-512" | "hmac-sha512" => {
            hmac_sha2::<Sha512>(key.as_bytes(), data.as_bytes(), 128)
        }
        other => {
            return Err(runtime_error(format!(
                "crypto.hmac: unsupported algorithm '{}'. Supported: sha256, sha512",
                other
            )));
        }
    };
    Ok(Value16::string(result))
}

fn hmac_sha2<D: Digest + Default>(key: &[u8], data: &[u8], block_size: usize) -> String {
    let mut key_padded = if key.len() > block_size {
        let mut hasher = D::new();
        hasher.update(key);
        hasher.finalize().to_vec()
    } else {
        key.to_vec()
    };
    key_padded.resize(block_size, 0);

    let ipad: Vec<u8> = key_padded.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = key_padded.iter().map(|b| b ^ 0x5c).collect();

    let mut inner = D::new();
    inner.update(&ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = D::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    hex::encode(outer.finalize())
}
