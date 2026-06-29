//! Tests for hudhud-crypto — public API happy-path + error cases.
//! Moved from inline #[cfg(test)] per Kural.

use hudhud_crypto::crypto_ops::hash::crypto_hash;
use hudhud_crypto::crypto_ops::random::crypto_random_bytes;
use hudhud_crypto::crypto_ops::utils::{constant_time_eq, secure_random_bytes, sha256_bytes};
use hudhudscript_bytecode::Value16;

#[test]
fn test_sha256_known_vector() {
    let hash = sha256_bytes(b"abc");
    // SHA-256("abc") = ba7816bf...
    assert_eq!(hash.len(), 32);
    assert_eq!(
        hex::encode(&hash),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn test_sha256_empty() {
    let hash = sha256_bytes(b"");
    assert_eq!(hash.len(), 32);
    assert_eq!(
        hex::encode(&hash),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_secure_random_bytes_length() {
    let r = secure_random_bytes(16);
    assert_eq!(r.len(), 16);
    // Should not be all zeros
    assert!(
        r.iter().any(|&b| b != 0),
        "random bytes should not be all zeros"
    );
}

#[test]
fn test_constant_time_eq_same() {
    let a = b"hello";
    let b = b"hello";
    assert!(constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_different() {
    assert!(!constant_time_eq(b"hello", b"world"));
}

#[test]
fn test_crypto_hash_sha256() {
    let result = crypto_hash(&[
        Value16::string("sha256".to_string()),
        Value16::string("abc".to_string()),
    ]);
    assert!(
        result.is_ok(),
        "crypto_hash(sha256, abc) should succeed: {:?}",
        result.err()
    );
    let val = result.unwrap();
    assert!(val.as_string().is_some(), "hash should be a hex string");
}

#[test]
fn test_crypto_hash_missing_args() {
    let result = crypto_hash(&[Value16::string("sha256".to_string())]);
    assert!(result.is_err(), "should fail with missing input");
}

#[test]
fn test_crypto_random_bytes_ok() {
    let result = crypto_random_bytes(&[Value16::number(16.0)]);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.as_string().is_some(),
        "random bytes should return hex string"
    );
}

#[test]
fn test_crypto_random_bytes_zero_length() {
    let result = crypto_random_bytes(&[Value16::number(0.0)]);
    assert!(result.is_ok());
}
