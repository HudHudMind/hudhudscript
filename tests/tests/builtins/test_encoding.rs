use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::hudhud_encoding::{Base64MethodId, HexMethodId, UrlMethodId};

// Thin wrappers so the test bodies stay identical to the interpreter-era
// originals (Kural 1).
fn base64_encode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    Base64MethodId::Encode.dispatch(args)
}
fn base64_decode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    Base64MethodId::Decode.dispatch(args)
}
fn hex_encode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    HexMethodId::Encode.dispatch(args)
}
fn hex_decode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    HexMethodId::Decode.dispatch(args)
}
fn url_encode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    UrlMethodId::Encode.dispatch(args)
}
fn url_decode(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    UrlMethodId::Decode.dispatch(args)
}

#[test]
fn test_base64_roundtrip() {
    let input = Value16::string("Hello World".to_string());
    let encoded = base64_encode(&[input]).unwrap();
    assert_eq!(encoded, Value16::string("SGVsbG8gV29ybGQ=".to_string()));

    let decoded = base64_decode(&[encoded]).unwrap();
    assert_eq!(decoded, Value16::string("Hello World".to_string()));
}

#[test]
fn test_hex_roundtrip() {
    let input = Value16::string("Hello".to_string());
    let encoded = hex_encode(&[input]).unwrap();
    assert_eq!(encoded, Value16::string("48656c6c6f".to_string()));

    let decoded = hex_decode(&[encoded]).unwrap();
    assert_eq!(decoded, Value16::string("Hello".to_string()));
}

#[test]
fn test_url_roundtrip() {
    let input = Value16::string("hello world&foo=bar".to_string());
    let encoded = url_encode(&[input]).unwrap();
    if let Some(s) = encoded.as_str() {
        assert!(s.contains("%20") || s.contains("+"));
    }

    let decoded = url_decode(&[encoded]).unwrap();
    assert_eq!(decoded, Value16::string("hello world&foo=bar".to_string()));
}

#[test]
fn test_base64_decode_error() {
    let result = base64_decode(&[Value16::string("!!!invalid!!!".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_hex_decode_error() {
    let result = hex_decode(&[Value16::string("ZZZZ".to_string())]);
    assert!(result.is_err());
}
