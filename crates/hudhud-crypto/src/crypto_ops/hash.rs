//! crypto.hash implementation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use sha2::{Digest, Sha256, Sha512};

use super::types::MAX_CRYPTO_DATA_BYTES;
use super::utils::{ensure_max_bytes, require_str, runtime_error};

pub fn crypto_hash(args: &[Value16]) -> HudHudResult<Value16> {
    let algorithm = require_str(args, 0, "crypto.hash")?;
    let data = require_str(args, 1, "crypto.hash")?;
    ensure_max_bytes("crypto.hash", "data", data, MAX_CRYPTO_DATA_BYTES)?;

    let hex_result = match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" => {
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha512" | "sha-512" => {
            let mut hasher = Sha512::new();
            hasher.update(data.as_bytes());
            hex::encode(hasher.finalize())
        }
        "blake3" => blake3::hash(data.as_bytes()).to_hex().to_string(),
        other => {
            return Err(runtime_error(format!(
                "crypto.hash: unsupported algorithm '{}'. Supported: sha256, sha512, blake3",
                other
            )));
        }
    };
    Ok(Value16::string(hex_result))
}
