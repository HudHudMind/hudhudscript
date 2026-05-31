//! Crypto operation identifiers and zero-cost dispatch.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

pub const MAX_RANDOM_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_CRYPTO_DATA_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_CIPHER_DATA_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_CIPHER_B64_BYTES: usize = 15 * 1024 * 1024;
pub const MAX_CRYPTO_KEY_BYTES: usize = 1024 * 1024;
pub const MAX_PASSWORD_BYTES: usize = 1024 * 1024;
pub const MAX_HASH_STRING_BYTES: usize = 1024;

/// Enum identifying each crypto operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoMethodId {
    Hash,
    Hmac,
    Encrypt,
    Decrypt,
    RandomBytes,
    HashPassword,
    VerifyPassword,
}

impl std::str::FromStr for CryptoMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hash" => Ok(Self::Hash),
            "hmac" => Ok(Self::Hmac),
            "encrypt" => Ok(Self::Encrypt),
            "decrypt" => Ok(Self::Decrypt),
            "random_bytes" => Ok(Self::RandomBytes),
            "hash_password" => Ok(Self::HashPassword),
            "verify_password" => Ok(Self::VerifyPassword),
            _ => Err(super::utils::runtime_error(format!(
                "Unknown crypto method: {}",
                s
            ))),
        }
    }
}

/// Zero-cost enum dispatch for crypto operations.
pub fn dispatch(method: CryptoMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        CryptoMethodId::Hash => super::hash::crypto_hash(args),
        CryptoMethodId::Hmac => super::hmac::crypto_hmac(args),
        CryptoMethodId::Encrypt => super::cipher::crypto_encrypt(args),
        CryptoMethodId::Decrypt => super::cipher::crypto_decrypt(args),
        CryptoMethodId::RandomBytes => super::random::crypto_random_bytes(args),
        CryptoMethodId::HashPassword => super::password::crypto_hash_password(args),
        CryptoMethodId::VerifyPassword => super::password::crypto_verify_password(args),
    }
}
