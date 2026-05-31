//! External tests for hudhudscript-package::signature —
//! PackageSignature, SignatureAlgorithm, SignaturePolicy, verify_signature, enforce_policy, compute_signature.

use chrono::Utc;
use hudhudscript_package::{
    compute_signature, enforce_policy, generate_keypair, verify_signature, PackageError,
    PackageSignature, SignatureAlgorithm, SignaturePolicy,
};

/// Generate a test keypair (signing_key, verifying_key) — both 32 bytes for Ed25519.
fn test_keypair() -> (Vec<u8>, Vec<u8>) {
    generate_keypair()
}

fn make_signature(data: &[u8], signing_key: &[u8]) -> PackageSignature {
    PackageSignature {
        signer: "test-signer".to_string(),
        signature_bytes: compute_signature(data, signing_key),
        algorithm: SignatureAlgorithm::Ed25519,
        timestamp: Utc::now(),
    }
}

#[test]
fn test_verify_valid_signature() {
    let (sk, vk) = test_keypair();
    let data = b"package content";
    let sig = make_signature(data, &sk);
    let result = verify_signature(data, &sig, &vk).unwrap();
    assert!(result);
}

#[test]
fn test_verify_invalid_signature() {
    let (sk, _vk) = test_keypair();
    let (_sk2, vk2) = test_keypair(); // different keypair
    let data = b"package content";
    let sig = make_signature(data, &sk);
    let result = verify_signature(data, &sig, &vk2).unwrap();
    assert!(!result);
}

#[test]
fn test_verify_tampered_data() {
    let (sk, vk) = test_keypair();
    let data = b"package content";
    let sig = make_signature(data, &sk);
    let tampered = b"tampered content";
    let result = verify_signature(tampered, &sig, &vk).unwrap();
    assert!(!result);
}

#[test]
fn test_verify_empty_key_error() {
    let (sk, _vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    let result = verify_signature(b"data", &sig, b"");
    assert!(result.is_err());
}

#[test]
fn test_verify_empty_signature_error() {
    let sig = PackageSignature {
        signer: "test".to_string(),
        signature_bytes: vec![],
        algorithm: SignatureAlgorithm::Ed25519,
        timestamp: Utc::now(),
    };
    let (_sk, vk) = test_keypair();
    let result = verify_signature(b"data", &sig, &vk);
    assert!(result.is_err());
}

#[test]
fn test_enforce_policy_none() {
    let result = enforce_policy(SignaturePolicy::None, b"data", None, None);
    assert!(result.is_ok());
}

#[test]
fn test_enforce_policy_optional_no_sig() {
    let result = enforce_policy(SignaturePolicy::Optional, b"data", None, None);
    assert!(result.is_ok());
}

#[test]
fn test_enforce_policy_optional_valid_sig() {
    let (sk, vk) = test_keypair();
    let data = b"package data";
    let sig = make_signature(data, &sk);
    let result = enforce_policy(SignaturePolicy::Optional, data, Some(&sig), Some(&vk));
    assert!(result.is_ok());
}

#[test]
fn test_enforce_policy_optional_invalid_sig() {
    let (sk, _vk) = test_keypair();
    let (_sk2, wrong_vk) = test_keypair();
    let data = b"package data";
    let sig = make_signature(data, &sk);
    let result = enforce_policy(SignaturePolicy::Optional, data, Some(&sig), Some(&wrong_vk));
    assert!(result.is_err());
}

#[test]
fn test_enforce_policy_required_no_sig() {
    let (_sk, vk) = test_keypair();
    let result = enforce_policy(SignaturePolicy::Required, b"data", None, Some(&vk));
    assert!(result.is_err());
}

#[test]
fn test_enforce_policy_required_no_key() {
    let (sk, _vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    let result = enforce_policy(SignaturePolicy::Required, b"data", Some(&sig), None);
    assert!(result.is_err());
}

#[test]
fn test_enforce_policy_required_valid() {
    let (sk, vk) = test_keypair();
    let data = b"important data";
    let sig = make_signature(data, &sk);
    let result = enforce_policy(SignaturePolicy::Required, data, Some(&sig), Some(&vk));
    assert!(result.is_ok());
}

#[test]
fn test_signature_serialization() {
    let (sk, _vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    let json = serde_json::to_string(&sig).unwrap();
    let deserialized: PackageSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.signer, "test-signer");
    assert_eq!(deserialized.signature_bytes, sig.signature_bytes);
    assert_eq!(deserialized.algorithm, SignatureAlgorithm::Ed25519);
}

#[test]
fn test_signature_policy_default() {
    assert_eq!(SignaturePolicy::default(), SignaturePolicy::Optional);
}

#[test]
fn test_compute_signature_deterministic() {
    let (sk, _vk) = test_keypair();
    let sig1 = compute_signature(b"data", &sk);
    let sig2 = compute_signature(b"data", &sk);
    assert_eq!(sig1, sig2);
    assert_eq!(sig1.len(), 64); // Ed25519 produces 64-byte signatures
}

#[test]
fn test_compute_signature_different_keys() {
    let (sk1, _vk1) = test_keypair();
    let (sk2, _vk2) = test_keypair();
    let sig1 = compute_signature(b"data", &sk1);
    let sig2 = compute_signature(b"data", &sk2);
    assert_ne!(sig1, sig2);
}

#[test]
fn test_compute_signature_different_data() {
    let (sk, _vk) = test_keypair();
    let sig1 = compute_signature(b"data1", &sk);
    let sig2 = compute_signature(b"data2", &sk);
    assert_ne!(sig1, sig2);
}

#[test]
fn test_signature_algorithm_serde() {
    let algos = [
        (SignatureAlgorithm::Ed25519, "\"ed25519\""),
        (SignatureAlgorithm::RsaSha256, "\"rsa-sha256\""),
        (SignatureAlgorithm::EcdsaP256, "\"ecdsa-p256\""),
    ];
    for (algo, expected) in &algos {
        let json = serde_json::to_string(algo).unwrap();
        assert_eq!(&json, expected);
        let deserialized: SignatureAlgorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, algo);
    }
}

#[test]
fn test_signature_policy_serde() {
    let policies = [
        (SignaturePolicy::Required, "\"required\""),
        (SignaturePolicy::Optional, "\"optional\""),
        (SignaturePolicy::None, "\"none\""),
    ];
    for (policy, expected) in &policies {
        let json = serde_json::to_string(policy).unwrap();
        assert_eq!(&json, expected);
        let deserialized: SignaturePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, policy);
    }
}

#[test]
fn test_enforce_policy_required_invalid_sig() {
    let (sk, _vk) = test_keypair();
    let (_sk2, wrong_vk) = test_keypair();
    let data = b"data";
    let sig = make_signature(data, &sk);
    let result = enforce_policy(SignaturePolicy::Required, data, Some(&sig), Some(&wrong_vk));
    assert!(result.is_err());
}

#[test]
fn test_enforce_policy_optional_with_sig_no_key() {
    let (sk, _vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    // sig present, key is None => Optional policy should still pass (no verification)
    let result = enforce_policy(SignaturePolicy::Optional, b"data", Some(&sig), None);
    assert!(result.is_ok());
}

#[test]
fn test_enforce_policy_none_with_everything() {
    let (sk, vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    let result = enforce_policy(SignaturePolicy::None, b"data", Some(&sig), Some(&vk));
    assert!(result.is_ok());
}

#[test]
fn test_enforce_policy_none_with_invalid_sig() {
    let (sk, _vk) = test_keypair();
    let (_sk2, wrong_vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    // None policy ignores everything
    let result = enforce_policy(SignaturePolicy::None, b"data", Some(&sig), Some(&wrong_vk));
    assert!(result.is_ok());
}

#[test]
fn test_signature_with_rsa_algorithm() {
    let (sk, _vk) = test_keypair();
    let sig = PackageSignature {
        signer: "rsa-signer".to_string(),
        signature_bytes: compute_signature(b"data", &sk),
        algorithm: SignatureAlgorithm::RsaSha256,
        timestamp: Utc::now(),
    };
    let json = serde_json::to_string(&sig).unwrap();
    let deserialized: PackageSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.algorithm, SignatureAlgorithm::RsaSha256);
}

#[test]
fn test_signature_with_ecdsa_algorithm() {
    let (sk, _vk) = test_keypair();
    let sig = PackageSignature {
        signer: "ecdsa-signer".to_string(),
        signature_bytes: compute_signature(b"data", &sk),
        algorithm: SignatureAlgorithm::EcdsaP256,
        timestamp: Utc::now(),
    };
    let json = serde_json::to_string(&sig).unwrap();
    let deserialized: PackageSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.algorithm, SignatureAlgorithm::EcdsaP256);
}

#[test]
fn test_compute_signature_empty_data() {
    let (sk, _vk) = test_keypair();
    let sig = compute_signature(b"", &sk);
    assert_eq!(sig.len(), 64); // Ed25519 always produces 64-byte signatures
}

#[test]
fn test_enforce_policy_required_errors_are_descriptive() {
    // No signature
    let (_sk, vk) = test_keypair();
    let result = enforce_policy(SignaturePolicy::Required, b"data", None, Some(&vk));
    match result {
        Err(PackageError::SecurityVulnerability(msg)) => {
            assert!(msg.contains("required but not present"));
        }
        _ => panic!("Expected SecurityVulnerability error"),
    }

    // No key
    let (sk, _vk) = test_keypair();
    let sig = make_signature(b"data", &sk);
    let result = enforce_policy(SignaturePolicy::Required, b"data", Some(&sig), None);
    match result {
        Err(PackageError::SecurityVulnerability(msg)) => {
            assert!(msg.contains("Public key is required"));
        }
        _ => panic!("Expected SecurityVulnerability error"),
    }
}
