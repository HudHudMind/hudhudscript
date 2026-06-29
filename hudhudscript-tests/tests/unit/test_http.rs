//! Tests for hudhud-http — public API (non-network).
use hudhud_http::http_ops;
use hudhud_http::http_ops::parse_url_host;
use hudhudscript_bytecode::Value16;

#[test]
fn test_parse_url_host_standard() {
    assert_eq!(
        parse_url_host("https://example.com/path"),
        Some("example.com".into())
    );
}

#[test]
fn test_parse_url_host_with_port() {
    assert_eq!(
        parse_url_host("http://localhost:8080/api"),
        Some("localhost".into())
    );
}

#[test]
fn test_parse_url_host_no_scheme() {
    assert_eq!(parse_url_host("example.com"), None);
}

#[test]
fn test_parse_url_host_empty() {
    assert_eq!(parse_url_host(""), None);
}

#[test]
fn test_http_get_missing_url() {
    let result = http_ops::get(&[]);
    assert!(result.is_err(), "get() with no args should fail");
}

#[test]
fn test_http_post_missing_args() {
    let result = http_ops::post(&[]);
    assert!(result.is_err(), "post() with no args should fail");
}
