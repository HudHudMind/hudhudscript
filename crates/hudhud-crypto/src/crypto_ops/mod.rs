//! Shared crypto builtins — SHA-2, BLAKE3, HMAC, AES-256-GCM, Argon2id, secure RNG (Kural 7).
//!
//! Single source of truth for both the VM and interpreter runtimes.
//! All algorithms use real, audited implementations:
//!   - Hashing: SHA-256, SHA-512 (`sha2`), BLAKE3 (`blake3`)
//!   - HMAC: HMAC-SHA-256, HMAC-SHA-512 (RFC 2104 over `sha2`)
//!   - Encryption: AES-256-GCM (`aes-gcm`) — real AEAD
//!   - Password hashing: Argon2id (`argon2`) — memory-hard KDF
//!   - Random: OS RNG (`aes-gcm` → `getrandom` on all platforms)

pub mod cipher;
pub mod hash;
pub mod hmac;
pub mod password;
pub mod random;
pub mod types;
pub mod utils;

pub use types::{dispatch, CryptoMethodId, MAX_HASH_STRING_BYTES};
