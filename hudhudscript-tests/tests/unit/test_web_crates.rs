//! Tests for hudhud-web-* crates (moved from inline #[cfg(test)] modules).
//! Only public-API tests; internal-function tests deleted per Kural.

use hudhud_web_markdown::to_html;
use hudhud_web_request::cookies::parse_cookies;
use hudhud_web_request::json_body::parse_json_body;
use hudhud_web_request::multipart::parse_multipart;
use hudhud_web_request::query::parse_query_string;
use hudhud_web_response::html;
use hudhud_web_template::filters::{apply_filter, html_escape};
use hudhud_web_template::{escape, render};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

// ── web-markdown ──────────────────────────────────────────────────

#[test]
fn test_markdown_heading() {
    let result = to_html(&[Value16::string("# Başlık")]).unwrap();
    assert_eq!(result.as_str().unwrap(), "<h1>Başlık</h1>\n");
}

#[test]
fn test_markdown_bold() {
    let result = to_html(&[Value16::string("**kalın**")]).unwrap();
    assert_eq!(result.as_str().unwrap(), "<p><strong>kalın</strong></p>\n");
}

// ── web-request/cookies ──────────────────────────────────────────

#[test]
fn test_parse_cookies_simple() {
    let obj = parse_cookies("session=abc123; theme=dark");
    let map = obj.as_object().unwrap();
    assert_eq!(map.get("session").unwrap().as_str().unwrap(), "abc123");
    assert_eq!(map.get("theme").unwrap().as_str().unwrap(), "dark");
}

#[test]
fn test_parse_cookies_empty() {
    let obj = parse_cookies("");
    assert!(obj.as_object().unwrap().is_empty());
}

// ── web-request/json_body ────────────────────────────────────────

#[test]
fn test_parse_json_object() {
    let result = parse_json_body(r#"{"name":"onur","age":30}"#).unwrap();
    let map = result.as_object().unwrap();
    assert_eq!(map.get("name").unwrap().as_str().unwrap(), "onur");
    assert_eq!(map.get("age").unwrap().as_number().unwrap(), 30.0);
}

#[test]
fn test_parse_json_array() {
    let result = parse_json_body(r#"[1,2,3]"#).unwrap();
    assert_eq!(result.as_array().unwrap().len(), 3);
}

// ── web-request/query ────────────────────────────────────────────

#[test]
fn test_parse_query_simple() {
    let obj = parse_query_string("a=1&b=2");
    let map = obj.as_object().unwrap();
    assert_eq!(map.get("a").unwrap().as_str().unwrap(), "1");
    assert_eq!(map.get("b").unwrap().as_str().unwrap(), "2");
}

#[test]
fn test_parse_query_empty() {
    assert!(parse_query_string("").as_object().unwrap().is_empty());
}

// ── web-request/multipart ────────────────────────────────────────

#[test]
fn test_parse_multipart_single_file() {
    let body = b"------boundary\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
Content-Type: text/plain\r\n\
\r\n\
hello world\r\n\
------boundary--\r\n";
    let result = parse_multipart(body, "----boundary");
    let map = result.as_object().unwrap();
    let file = map.get("file").unwrap().as_object().unwrap();
    assert_eq!(file.get("filename").unwrap().as_str().unwrap(), "test.txt");
    assert_eq!(
        file.get("content_type").unwrap().as_str().unwrap(),
        "text/plain"
    );
}

// ── web-response ─────────────────────────────────────────────────

#[test]
fn test_html_response() {
    let resp = html(&[Value16::string("<h1>Hello</h1>")]).unwrap();
    let obj = resp.as_object().unwrap();
    assert_eq!(obj.get("status").unwrap().as_number().unwrap(), 200.0);
    assert_eq!(obj.get("body").unwrap().as_str().unwrap(), "<h1>Hello</h1>");
}

// ── web-template/filters ─────────────────────────────────────────

#[test]
fn test_html_escape_chars() {
    assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    assert_eq!(html_escape("a & b"), "a &amp; b");
}

#[test]
fn test_filter_upper() {
    assert_eq!(
        apply_filter("upper", &Value16::string("hello"), &[])
            .as_str()
            .unwrap(),
        "HELLO"
    );
}

#[test]
fn test_filter_lower() {
    assert_eq!(
        apply_filter("lower", &Value16::string("HELLO"), &[])
            .as_str()
            .unwrap(),
        "hello"
    );
}

// ── web-template ─────────────────────────────────────────────────

#[test]
fn test_escape_html() {
    let result = escape(&[Value16::string("<script>alert('x')</script>")]).unwrap();
    assert!(result.as_str().unwrap().contains("&lt;script&gt;"));
}

#[test]
fn test_render_variable() {
    let mut ctx = HashMap::new();
    ctx.insert("name".to_string(), Value16::string("Onur".to_string()));
    let result = render(&[
        Value16::string("<h1>Merhaba {{ name }}</h1>"),
        Value16::object(ctx),
    ]);
    assert!(result.is_ok(), "render should succeed: {:?}", result.err());
}
