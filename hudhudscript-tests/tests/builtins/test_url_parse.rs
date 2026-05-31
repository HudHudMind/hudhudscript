use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::url_parser::UrlParserMethodId;
use std::collections::HashMap;

#[test]
fn test_url_parse_full() {
    let result = hudhudscript_shared_builtins::url_parser::dispatch(
        UrlParserMethodId::Parse,
        &[Value16::string(
            "https://user:pass@example.com:8080/path?q=1#frag".to_string(),
        )],
    )
    .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("scheme"),
            Some(&Value16::string("https".to_string()))
        );
        assert_eq!(
            obj.get("host"),
            Some(&Value16::string("example.com".to_string()))
        );
        assert_eq!(obj.get("port"), Some(&Value16::number(8080.0)));
        assert_eq!(obj.get("path"), Some(&Value16::string("/path".to_string())));
        assert_eq!(obj.get("query"), Some(&Value16::string("q=1".to_string())));
        assert_eq!(
            obj.get("fragment"),
            Some(&Value16::string("frag".to_string()))
        );
        assert_eq!(
            obj.get("username"),
            Some(&Value16::string("user".to_string()))
        );
        assert_eq!(
            obj.get("password"),
            Some(&Value16::string("pass".to_string()))
        );
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_url_parse_simple() {
    let result = hudhudscript_shared_builtins::url_parser::dispatch(
        UrlParserMethodId::Parse,
        &[Value16::string("https://example.com/page".to_string())],
    )
    .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("scheme"),
            Some(&Value16::string("https".to_string()))
        );
        assert_eq!(
            obj.get("host"),
            Some(&Value16::string("example.com".to_string()))
        );
        assert_eq!(obj.get("port"), Some(&Value16::null()));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_url_parse_invalid() {
    let result = hudhudscript_shared_builtins::url_parser::dispatch(
        UrlParserMethodId::Parse,
        &[Value16::string("not a url".to_string())],
    );
    assert!(result.is_err());
}

#[test]
fn test_url_format() {
    let mut obj = HashMap::new();
    obj.insert("scheme".to_string(), Value16::string("https".to_string()));
    obj.insert(
        "host".to_string(),
        Value16::string("example.com".to_string()),
    );
    obj.insert("port".to_string(), Value16::number(443.0));
    obj.insert("path".to_string(), Value16::string("/api".to_string()));
    let result = hudhudscript_shared_builtins::url_parser::dispatch(
        UrlParserMethodId::Format,
        &[Value16::object(obj)],
    )
    .unwrap();
    if let Some(s) = result.as_str() {
        assert!(s.contains("https://"));
        assert!(s.contains("example.com"));
        assert!(s.contains("/api"));
    } else {
        panic!("Expected string");
    }
}
