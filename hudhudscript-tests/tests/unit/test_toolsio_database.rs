//! Tests for hudhudscript-tools-io database internals — metadata decoding,
//! migration checksum decoding, runtime block_on, and config validation
//! (moved from src/database/{metadata,migrations,runtime,config}.rs).

use hudhudscript_tools_io::database::metadata::{metadata_bool, metadata_text, table_name};
use hudhudscript_tools_io::database::migrations::decoded_i64;
use hudhudscript_tools_io::database::runtime::block_on;
use hudhudscript_tools_io::database::Row;
use hudhudscript_tools_io::DatabaseConfig;
use serde_json::json;

// ── metadata ──────────────────────────────────────────────────────

#[test]
fn table_name_accepts_driver_specific_column_case() {
    for key in ["hudhud_table_name", "HUDHUD_TABLE_NAME"] {
        let row = Row::from([(key.into(), json!("users"))]);
        assert_eq!(table_name(row).as_deref(), Some("users"));
    }
}

#[test]
fn metadata_text_accepts_mysql_information_schema_bytes() {
    let value = json!({"$type": "bytes", "base64": "dXNlcnM="});
    assert_eq!(metadata_text(&value).as_deref(), Some("users"));
    assert_eq!(metadata_bool(&json!(1)), Some(true));
    assert_eq!(metadata_bool(&json!(0)), Some(false));
}

// ── migrations ────────────────────────────────────────────────────

#[test]
fn migration_versions_accept_plain_and_lossless_i64_json() {
    assert_eq!(decoded_i64(&json!(42)), Some(42));
    assert_eq!(
        decoded_i64(&json!({
            "$type": "i64",
            "value": "9007199254740992"
        })),
        Some(9_007_199_254_740_992)
    );
}

#[test]
fn migration_versions_reject_other_tagged_values() {
    assert_eq!(decoded_i64(&json!({"$type": "u64", "value": "42"})), None);
    assert_eq!(
        decoded_i64(&json!({"$type": "i64", "value": "invalid"})),
        None
    );
}

// ── runtime ───────────────────────────────────────────────────────

#[test]
fn works_without_a_caller_runtime() {
    assert_eq!(block_on(async { Ok(7) }).unwrap(), 7);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn works_inside_a_multithread_runtime() {
    assert_eq!(block_on(async { Ok(8) }).unwrap(), 8);
}

#[test]
fn works_inside_a_current_thread_runtime() {
    let caller = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let value = caller.block_on(async { block_on(async { Ok(9) }) });
    assert_eq!(value.unwrap(), 9);
}

// ── config ────────────────────────────────────────────────────────

#[test]
fn tls_required_rejects_plain_remote_url() {
    let mut config = DatabaseConfig::postgres("postgres://user:pass@db.example/app");
    config.tls_required = true;
    assert!(config.validate().is_err());
    config.connection_string.push_str("?sslmode=verify-full");
    assert!(config.validate().is_ok());
}

#[test]
fn backend_and_url_scheme_must_match() {
    let config = DatabaseConfig::mysql("postgres://localhost/app");
    assert!(config.validate().is_err());
}

#[test]
fn debug_output_redacts_credentials() {
    let config = DatabaseConfig::postgres("postgres://admin:secret@localhost/app");
    let output = format!("{config:?}");
    assert!(output.contains("<redacted>"));
    assert!(!output.contains("secret"));
    assert!(!output.contains("admin"));
}
