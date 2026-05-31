//! Tests extracted from hudhudscript-modules/src/gguf.rs

use hudhudscript_modules::{
    parse_gguf_header, GgufError, GgufMetadata, GgufQuantization, GgufValue, GGUF_MAGIC,
    GGUF_TYPE_ARRAY, GGUF_TYPE_BOOL, GGUF_TYPE_FLOAT32, GGUF_TYPE_FLOAT64, GGUF_TYPE_INT16,
    GGUF_TYPE_INT32, GGUF_TYPE_INT64, GGUF_TYPE_INT8, GGUF_TYPE_STRING, GGUF_TYPE_UINT16,
    GGUF_TYPE_UINT32, GGUF_TYPE_UINT64, GGUF_TYPE_UINT8,
};

/// Build a minimal valid GGUF v3 header with 0 tensors and 0 KV pairs.
fn minimal_header() -> Vec<u8> {
    let mut buf = Vec::new();
    // magic
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    // version 3
    buf.extend_from_slice(&3u32.to_le_bytes());
    // tensor count
    buf.extend_from_slice(&0u64.to_le_bytes());
    // kv count
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf
}

#[test]
fn test_parse_empty_header() {
    let data = minimal_header();
    let meta = parse_gguf_header(&data).unwrap();
    assert_eq!(meta.architecture, "");
    assert_eq!(meta.context_length, 0);
}

#[test]
fn test_invalid_magic() {
    let mut data = minimal_header();
    data[0] = 0xFF;
    assert!(matches!(
        parse_gguf_header(&data),
        Err(GgufError::InvalidMagic(_))
    ));
}

#[test]
fn test_too_short() {
    assert!(matches!(
        parse_gguf_header(&[0u8; 4]),
        Err(GgufError::TooShort)
    ));
}

#[test]
fn test_unsupported_version() {
    let mut data = minimal_header();
    // overwrite version to 99
    data[4..8].copy_from_slice(&99u32.to_le_bytes());
    assert!(matches!(
        parse_gguf_header(&data),
        Err(GgufError::UnsupportedVersion(99))
    ));
}

#[test]
fn test_quantization_from() {
    assert_eq!(GgufQuantization::from(2), GgufQuantization::Q4_0);
    assert_eq!(GgufQuantization::from(1), GgufQuantization::F16);
    assert_eq!(GgufQuantization::from(0), GgufQuantization::F32);
    assert_eq!(GgufQuantization::from(999), GgufQuantization::Unknown(999));
}

#[test]
fn test_quantization_display() {
    assert_eq!(GgufQuantization::Q4_0.to_string(), "Q4_0");
    assert_eq!(GgufQuantization::F16.to_string(), "F16");
    assert_eq!(GgufQuantization::Unknown(42).to_string(), "Unknown(42)");
}

/// Build a header with one KV pair: general.architecture = "llama"
fn header_with_architecture() -> Vec<u8> {
    let mut buf = Vec::new();
    // magic
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    // version 3
    buf.extend_from_slice(&3u32.to_le_bytes());
    // tensor count = 0
    buf.extend_from_slice(&0u64.to_le_bytes());
    // kv count = 1
    buf.extend_from_slice(&1u64.to_le_bytes());

    // KV: key
    let key = b"general.architecture";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    // type = STRING (8)
    buf.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
    // value
    let val = b"llama";
    buf.extend_from_slice(&(val.len() as u64).to_le_bytes());
    buf.extend_from_slice(val);

    buf
}

#[test]
fn test_parse_architecture_kv() {
    let data = header_with_architecture();
    let meta = parse_gguf_header(&data).unwrap();
    assert_eq!(meta.architecture, "llama");
}

// ---- Quantization: remaining variants ----

#[test]
fn test_quantization_all_known_variants() {
    assert_eq!(GgufQuantization::from(3), GgufQuantization::Q4_1);
    assert_eq!(GgufQuantization::from(6), GgufQuantization::Q5_0);
    assert_eq!(GgufQuantization::from(7), GgufQuantization::Q5_1);
    assert_eq!(GgufQuantization::from(8), GgufQuantization::Q8_0);
}

#[test]
fn test_quantization_display_all() {
    assert_eq!(GgufQuantization::Q4_1.to_string(), "Q4_1");
    assert_eq!(GgufQuantization::Q5_0.to_string(), "Q5_0");
    assert_eq!(GgufQuantization::Q5_1.to_string(), "Q5_1");
    assert_eq!(GgufQuantization::Q8_0.to_string(), "Q8_0");
    assert_eq!(GgufQuantization::F32.to_string(), "F32");
}

// ---- Quantization serialization roundtrip ----

#[test]
fn test_quantization_serialization() {
    let q = GgufQuantization::Q4_0;
    let json = serde_json::to_string(&q).unwrap();
    let deserialized: GgufQuantization = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, q);
}

// ---- GgufMetadata serialization roundtrip ----

#[test]
fn test_gguf_metadata_serialization() {
    let meta = GgufMetadata {
        architecture: "llama".to_string(),
        context_length: 4096,
        quantization: GgufQuantization::Q4_0,
        embedding_length: 4096,
        vocab_size: 32000,
        file_size: 1_000_000,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: GgufMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.architecture, "llama");
    assert_eq!(deserialized.context_length, 4096);
    assert_eq!(deserialized.quantization, GgufQuantization::Q4_0);
    assert_eq!(deserialized.embedding_length, 4096);
    assert_eq!(deserialized.vocab_size, 32000);
}

// ---- GgufError display ----

#[test]
fn test_gguf_error_display() {
    let err = GgufError::TooShort;
    assert!(err.to_string().contains("too short"));

    let err = GgufError::InvalidMagic(0xDEADBEEF);
    assert!(err.to_string().contains("Invalid GGUF magic"));

    let err = GgufError::UnsupportedVersion(99);
    assert!(err.to_string().contains("99"));

    let err = GgufError::UnexpectedEof;
    assert!(err.to_string().contains("Unexpected end"));

    let err = GgufError::InvalidUtf8;
    assert!(err.to_string().contains("Invalid UTF-8"));
}

// ---- Version 2 is supported ----

#[test]
fn test_parse_version_2() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&2u32.to_le_bytes()); // version 2
    buf.extend_from_slice(&0u64.to_le_bytes()); // tensor count
    buf.extend_from_slice(&0u64.to_le_bytes()); // kv count
    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

// ---- Header with u32 KV (file_type / quantization) ----

#[test]
fn test_parse_with_file_type_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes()); // tensor count
    buf.extend_from_slice(&1u64.to_le_bytes()); // kv count = 1

    // KV: general.file_type = 2 (Q4_0)
    let key = b"general.file_type";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_UINT32.to_le_bytes());
    buf.extend_from_slice(&2u32.to_le_bytes()); // Q4_0

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.quantization, GgufQuantization::Q4_0);
}

// ---- Truncated header (no kv count) ----

#[test]
fn test_parse_truncated_after_version() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    // Missing tensor_count and kv_count
    // But we have 16 bytes minimum for the TooShort check
    buf.extend_from_slice(&0u64.to_le_bytes()); // tensor_count
                                                // No kv_count => UnexpectedEof when trying to read it
    let result = parse_gguf_header(&buf);
    assert!(result.is_err());
}

// ---- GgufValue::as_u64 and as_str coverage ----

#[test]
fn test_gguf_value_as_u64() {
    assert_eq!(GgufValue::U32(42).as_u64(), Some(42));
    assert_eq!(GgufValue::I32(10).as_u64(), Some(10));
    assert_eq!(GgufValue::U64(100).as_u64(), Some(100));
    assert_eq!(GgufValue::I64(200).as_u64(), Some(200));
    assert_eq!(GgufValue::F32(3.5).as_u64(), Some(3));
    assert_eq!(GgufValue::F64(7.9).as_u64(), Some(7));
    assert!(GgufValue::Bool(()).as_u64().is_none());
    assert!(GgufValue::Str("test".into()).as_u64().is_none());
    assert!(GgufValue::Array(()).as_u64().is_none());
    assert!(GgufValue::Other.as_u64().is_none());
}

#[test]
fn test_gguf_value_as_str() {
    assert_eq!(GgufValue::Str("hello".into()).as_str(), Some("hello"));
    assert!(GgufValue::U32(0).as_str().is_none());
    assert!(GgufValue::Other.as_str().is_none());
}

/// Build a header with multiple KV pairs for context_length, embedding_length, vocab_size
fn header_with_multiple_kvs() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes()); // tensor count
    buf.extend_from_slice(&4u64.to_le_bytes()); // kv count = 4

    // KV 1: general.architecture = "gpt2" (STRING)
    let key1 = b"general.architecture";
    buf.extend_from_slice(&(key1.len() as u64).to_le_bytes());
    buf.extend_from_slice(key1);
    buf.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
    let val1 = b"gpt2";
    buf.extend_from_slice(&(val1.len() as u64).to_le_bytes());
    buf.extend_from_slice(val1);

    // KV 2: gpt2.context_length = 1024 (UINT32)
    let key2 = b"gpt2.context_length";
    buf.extend_from_slice(&(key2.len() as u64).to_le_bytes());
    buf.extend_from_slice(key2);
    buf.extend_from_slice(&GGUF_TYPE_UINT32.to_le_bytes());
    buf.extend_from_slice(&1024u32.to_le_bytes());

    // KV 3: gpt2.embedding_length = 768 (UINT64)
    let key3 = b"gpt2.embedding_length";
    buf.extend_from_slice(&(key3.len() as u64).to_le_bytes());
    buf.extend_from_slice(key3);
    buf.extend_from_slice(&GGUF_TYPE_UINT64.to_le_bytes());
    buf.extend_from_slice(&768u64.to_le_bytes());

    // KV 4: tokenizer.ggml.vocab_size = 50257 (INT32)
    let key4 = b"tokenizer.ggml.vocab_size";
    buf.extend_from_slice(&(key4.len() as u64).to_le_bytes());
    buf.extend_from_slice(key4);
    buf.extend_from_slice(&GGUF_TYPE_INT32.to_le_bytes());
    buf.extend_from_slice(&50257i32.to_le_bytes());

    buf
}

#[test]
fn test_parse_multiple_kv_pairs() {
    let data = header_with_multiple_kvs();
    let meta = parse_gguf_header(&data).unwrap();
    assert_eq!(meta.architecture, "gpt2");
    assert_eq!(meta.context_length, 1024);
    assert_eq!(meta.embedding_length, 768);
    assert_eq!(meta.vocab_size, 50257);
}

#[test]
fn test_parse_header_with_bool_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes()); // 1 kv

    // KV: some.bool_key = true (BOOL)
    let key = b"some.bool_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_BOOL.to_le_bytes());
    buf.push(1u8); // true

    let meta = parse_gguf_header(&buf).unwrap();
    // Bool values are not extracted, so defaults remain
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_float32_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.float_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_FLOAT32.to_le_bytes());
    buf.extend_from_slice(&3.14f32.to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_float64_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.f64_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_FLOAT64.to_le_bytes());
    buf.extend_from_slice(&2.718f64.to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_i64_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.i64_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_INT64.to_le_bytes());
    buf.extend_from_slice(&(-42i64).to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_uint8_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.u8_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_UINT8.to_le_bytes());
    buf.push(42u8);

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_int8_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.i8_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_INT8.to_le_bytes());
    buf.push(0xFFu8);

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_uint16_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.u16_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_UINT16.to_le_bytes());
    buf.extend_from_slice(&256u16.to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_int16_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    let key = b"some.i16_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_INT16.to_le_bytes());
    buf.extend_from_slice(&(-1i16).to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_with_array_kv() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());
    buf.extend_from_slice(&0u64.to_le_bytes());
    buf.extend_from_slice(&1u64.to_le_bytes());

    // KV: some.array = [1u32, 2u32, 3u32]
    let key = b"some.array_key";
    buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&GGUF_TYPE_ARRAY.to_le_bytes());
    buf.extend_from_slice(&GGUF_TYPE_UINT32.to_le_bytes()); // elem type
    buf.extend_from_slice(&3u64.to_le_bytes()); // count
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&2u32.to_le_bytes());
    buf.extend_from_slice(&3u32.to_le_bytes());

    let meta = parse_gguf_header(&buf).unwrap();
    assert_eq!(meta.architecture, "");
}

#[test]
fn test_parse_header_file_size() {
    let data = minimal_header();
    let meta = parse_gguf_header(&data).unwrap();
    assert_eq!(meta.file_size, data.len() as u64);
}

#[test]
fn test_parse_header_no_quantization_defaults_to_unknown() {
    let data = minimal_header();
    let meta = parse_gguf_header(&data).unwrap();
    assert_eq!(meta.quantization, GgufQuantization::Unknown(0));
}

#[test]
fn test_gguf_value_clone() {
    let v = GgufValue::Str("test".to_string());
    let cloned = v.clone();
    assert_eq!(cloned.as_str(), Some("test"));
}

#[test]
fn test_quantization_unknown_display() {
    assert_eq!(GgufQuantization::Unknown(100).to_string(), "Unknown(100)");
    assert_eq!(GgufQuantization::Unknown(0).to_string(), "Unknown(0)");
}
