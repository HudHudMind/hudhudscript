//! Ed25519 cryptographic signature for package verification.
//!
//! PRODUCTION: Real Ed25519 via ed25519-dalek. No HMAC/SHA256 mock implementations.

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{PackageError, Result};

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// Cryptographic signature attached to a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSignature {
    /// Identity of the signer (e.g. key fingerprint or email).
    pub signer: String,
    /// Raw Ed25519 signature bytes (64 bytes, hex-encoded for JSON).
    #[serde(with = "hex_bytes")]
    pub signature_bytes: Vec<u8>,
    /// Algorithm used.
    pub algorithm: SignatureAlgorithm,
    /// When the signature was created.
    pub timestamp: DateTime<Utc>,
}

/// Supported signature algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureAlgorithm {
    Ed25519,
    RsaSha256,
    EcdsaP256,
}

/// Policy governing whether signatures are required, optional, or ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SignaturePolicy {
    Required,
    #[default]
    Optional,
    None,
}

mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Real Ed25519 verification
// ─────────────────────────────────────────────────────────────────────────────

/// Verify a package signature using Ed25519.
///
/// `public_key` must be 32 bytes (Ed25519 verifying key).
/// `signature.signature_bytes` must be 64 bytes (Ed25519 signature).
pub fn verify_signature(
    package_data: &[u8],
    signature: &PackageSignature,
    public_key: &[u8],
) -> Result<bool> {
    if public_key.len() != 32 {
        return Err(PackageError::Other(anyhow::anyhow!(
            "Ed25519 public key must be 32 bytes, got {}",
            public_key.len()
        )));
    }

    if signature.signature_bytes.len() != 64 {
        return Err(PackageError::Other(anyhow::anyhow!(
            "Ed25519 signature must be 64 bytes, got {}",
            signature.signature_bytes.len()
        )));
    }

    let verifying_key = VerifyingKey::from_bytes(
        public_key.try_into().unwrap(), // safe: we checked len == 32
    )
    .map_err(|e| PackageError::Other(anyhow::anyhow!("Invalid Ed25519 public key: {}", e)))?;

    let sig = Signature::from_bytes(
        signature.signature_bytes.as_slice().try_into().unwrap(), // safe: len == 64
    );

    Ok(verifying_key.verify(package_data, &sig).is_ok())
}

/// Sign package data with an Ed25519 signing key.
///
/// `signing_key_bytes` must be 32 bytes (Ed25519 secret key).
/// Returns 64-byte signature.
pub fn compute_signature(data: &[u8], signing_key_bytes: &[u8]) -> Vec<u8> {
    let signing_key = SigningKey::from_bytes(
        signing_key_bytes
            .try_into()
            .expect("signing key must be 32 bytes"),
    );
    let signature = signing_key.sign(data);
    signature.to_bytes().to_vec()
}

/// Generate a new Ed25519 keypair.
/// Returns (signing_key_32_bytes, verifying_key_32_bytes).
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    use rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Apply the signature policy.
pub fn enforce_policy(
    policy: SignaturePolicy,
    package_data: &[u8],
    signature: Option<&PackageSignature>,
    public_key: Option<&[u8]>,
) -> Result<()> {
    match policy {
        SignaturePolicy::None => Ok(()),
        SignaturePolicy::Optional => {
            if let (Some(sig), Some(key)) = (signature, public_key) {
                let valid = verify_signature(package_data, sig, key)?;
                if !valid {
                    return Err(PackageError::ChecksumMismatch(
                        "Package Ed25519 signature verification failed".to_string(),
                    ));
                }
            }
            Ok(())
        }
        SignaturePolicy::Required => {
            let sig = signature.ok_or_else(|| {
                PackageError::SecurityVulnerability(
                    "Signature is required but not present".to_string(),
                )
            })?;
            let key = public_key.ok_or_else(|| {
                PackageError::SecurityVulnerability(
                    "Public key is required for signature verification".to_string(),
                )
            })?;
            let valid = verify_signature(package_data, sig, key)?;
            if !valid {
                return Err(PackageError::ChecksumMismatch(
                    "Package Ed25519 signature verification failed".to_string(),
                ));
            }
            Ok(())
        }
    }
}
