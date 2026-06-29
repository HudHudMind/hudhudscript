use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::crypto_ops::cipher::{crypto_decrypt, crypto_encrypt};
use hudhudscript_shared_builtins::crypto_ops::hash::crypto_hash;
use hudhudscript_shared_builtins::crypto_ops::hmac::crypto_hmac;
use hudhudscript_shared_builtins::crypto_ops::password::{
    crypto_hash_password, crypto_verify_password,
};
use hudhudscript_shared_builtins::crypto_ops::random::crypto_random_bytes;

#[test]
fn test_sha256_hash() {
    let result = crypto_hash(&[
        Value16::string("sha256".to_string()),
        Value16::string("hello".to_string()),
    ])
    .unwrap();
    if let Some(h) = result.as_str() {
        assert_eq!(
            h,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_sha512_hash() {
    let result = crypto_hash(&[
        Value16::string("sha512".to_string()),
        Value16::string("hello".to_string()),
    ])
    .unwrap();
    if let Some(h) = result.as_str() {
        assert_eq!(h.len(), 128); // SHA-512 produces 64 bytes = 128 hex chars
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_hmac_sha256() {
    let result = crypto_hmac(&[
        Value16::string("sha256".to_string()),
        Value16::string("secret".to_string()),
        Value16::string("hello".to_string()),
    ])
    .unwrap();
    if let Some(h) = result.as_str() {
        assert_eq!(h.len(), 64); // 32 bytes = 64 hex chars
                                 // Known HMAC-SHA256("secret", "hello")
        assert_eq!(
            h,
            "88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b"
        );
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let plaintext = "Hello, World! This is a secret message.";
    let key = "my-super-secret-key-256";

    let encrypted = crypto_encrypt(&[
        Value16::string(plaintext.to_string()),
        Value16::string(key.to_string()),
    ])
    .unwrap();

    if let Some(ciphertext) = encrypted.as_str() {
        assert!(!ciphertext.is_empty());
        assert_ne!(ciphertext, plaintext);

        let decrypted =
            crypto_decrypt(&[encrypted.clone(), Value16::string(key.to_string())]).unwrap();

        assert_eq!(decrypted, Value16::string(plaintext.to_string()));
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_decrypt_wrong_key() {
    let encrypted = crypto_encrypt(&[
        Value16::string("secret data".to_string()),
        Value16::string("correct-key".to_string()),
    ])
    .unwrap();

    let result = crypto_decrypt(&[encrypted, Value16::string("wrong-key".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_random_bytes() {
    let result = crypto_random_bytes(&[Value16::number(16.0)]).unwrap();
    if let Some(h) = result.as_str() {
        assert_eq!(h.len(), 32); // 16 bytes = 32 hex chars
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_password_hash_and_verify() {
    let password = "my-password-123";

    let hashed = crypto_hash_password(&[Value16::string(password.to_string())]).unwrap();

    if let Some(hash_str) = hashed.as_str() {
        // Argon2id produces $argon2id$ prefix (production-grade KDF)
        assert!(
            hash_str.starts_with("$argon2"),
            "Expected Argon2 hash, got: {}",
            &hash_str[..hash_str.len().min(30)]
        );

        let verified =
            crypto_verify_password(&[Value16::string(password.to_string()), hashed.clone()])
                .unwrap();
        assert_eq!(verified, Value16::boolean(true));

        let wrong = crypto_verify_password(&[
            Value16::string("wrong-password".to_string()),
            hashed.clone(),
        ])
        .unwrap();
        assert_eq!(wrong, Value16::boolean(false));
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_unsupported_hash_algorithm() {
    let result = crypto_hash(&[
        Value16::string("md5".to_string()),
        Value16::string("data".to_string()),
    ]);
    assert!(result.is_err());
}
