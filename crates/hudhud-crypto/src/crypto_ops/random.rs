//! Secure random bytes generation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::types::MAX_RANDOM_BYTES;
use super::utils::{runtime_error, secure_random_bytes, type_error};

pub fn crypto_random_bytes(args: &[Value16]) -> HudHudResult<Value16> {
    let count = match args.first() {
        Some(v) => match v.as_number() {
            Some(n) => {
                if !n.is_finite() || n < 0.0 {
                    return Err(runtime_error(
                        "crypto.random_bytes: count must be a non-negative finite integer",
                    ));
                }
                if n.fract() != 0.0 {
                    return Err(runtime_error(
                        "crypto.random_bytes: count must be an integer",
                    ));
                }
                n as usize
            }
            None => {
                return Err(type_error(
                    "number",
                    v.type_name_str(),
                    "crypto.random_bytes",
                ));
            }
        },
        None => {
            return Err(runtime_error(
                "crypto.random_bytes requires a count argument",
            ));
        }
    };

    if count > MAX_RANDOM_BYTES {
        return Err(runtime_error(format!(
            "crypto.random_bytes: count too large (max {} bytes)",
            MAX_RANDOM_BYTES
        )));
    }

    let bytes = secure_random_bytes(count);
    Ok(Value16::string(hex::encode(bytes)))
}
