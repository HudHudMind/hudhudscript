use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::crypto_ops::cipher::crypto_encrypt;
use hudhudscript_shared_builtins::crypto_ops::random::crypto_random_bytes;
use hudhudscript_shared_builtins::crypto_ops::password::crypto_verify_password;
use hudhudscript_shared_builtins::crypto_ops::MAX_HASH_STRING_BYTES;

#[test]
fn random_bytes_rejects_negative_and_fractional() {
    let negative = crypto_random_bytes(&[Value16::number(-1.0)]);
    assert!(negative.is_err());

    let fractional = crypto_random_bytes(&[Value16::number(3.14)]);
    assert!(fractional.is_err());
}

#[test]
fn encrypt_rejects_empty_key() {
    let result = crypto_encrypt(&[Value16::string("hello"), Value16::string(String::new())]);
    assert!(result.is_err());
}

#[test]
fn verify_password_rejects_oversized_hash_input() {
    let oversized_hash = "x".repeat(MAX_HASH_STRING_BYTES + 1);
    let result =
        crypto_verify_password(&[Value16::string("password"), Value16::string(oversized_hash)]);
    assert!(result.is_err());
}
// TEST MARKER
