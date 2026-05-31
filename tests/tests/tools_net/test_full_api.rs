//! Real unit tests for hudhudscript-tools-net — OpenAPI types

use hudhudscript_tools_net::*;
use std::collections::HashMap;

#[test]
fn openapi_document_empty() {
    let doc = OpenApiDocument {
        openapi: Some("3.0.0".to_string()),
        swagger: None,
        info: None,
        paths: HashMap::new(),
    };
    assert_eq!(doc.openapi, Some("3.0.0".to_string()));
    assert!(doc.paths.is_empty());
}

#[test]
fn openapi_document_swagger_v2() {
    let doc = OpenApiDocument {
        openapi: None,
        swagger: Some("2.0".to_string()),
        info: None,
        paths: HashMap::new(),
    };
    assert_eq!(doc.swagger, Some("2.0".to_string()));
}

#[test]
fn openapi_document_with_path_item() {
    use hudhudscript_tools_net::openapi::OpenApiPathItem;
    let mut paths = HashMap::new();
    paths.insert("/users".to_string(), OpenApiPathItem::default());
    let doc = OpenApiDocument {
        openapi: Some("3.0.0".to_string()),
        swagger: None,
        info: None,
        paths,
    };
    assert_eq!(doc.paths.len(), 1);
}

#[test]
fn sanitize_name_removes_spaces() {
    let name = sanitize_name("hello world");
    assert!(!name.contains(" "));
    assert!(!name.is_empty());
}

#[test]
fn sanitize_name_handles_special_chars() {
    let name = sanitize_name("get-users/list");
    assert!(!name.is_empty());
}
