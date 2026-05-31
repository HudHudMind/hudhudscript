//! Tests for public helper functions in hudhudscript-tools-net:
//! sanitize_name, derive_tool_name, extract_param_type, build_parameters_schema,
//! and additional edge cases not covered by tools_net_test_lib.rs.

use hudhudscript_tools_net::http::*;
use hudhudscript_tools_net::openapi::*;
use serde_json::json;
use std::collections::HashMap;

// =========================================================================
// sanitize_name
// =========================================================================

#[test]
fn sanitize_name_lowercases_and_replaces_non_alnum() {
    assert_eq!(sanitize_name("GetUser"), "getuser");
    assert_eq!(sanitize_name("list-users"), "list_users");
    assert_eq!(sanitize_name("create.item"), "create_item");
    assert_eq!(sanitize_name("foo/bar/baz"), "foo_bar_baz");
}

#[test]
fn sanitize_name_preserves_underscores() {
    assert_eq!(sanitize_name("already_snake_case"), "already_snake_case");
}

#[test]
fn sanitize_name_empty_string() {
    assert_eq!(sanitize_name(""), "");
}

#[test]
fn sanitize_name_all_special_chars() {
    let result = sanitize_name("!@#$%^&*()");
    // Every character should become '_'
    assert_eq!(result.len(), 10);
    assert!(result.chars().all(|c| c == '_'));
}

// =========================================================================
// derive_tool_name
// =========================================================================

#[test]
fn derive_tool_name_uses_operation_id_when_present() {
    let op = OpenApiOperation {
        operation_id: Some("listPets".to_string()),
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/pets");
    assert_eq!(name, "listpets");
}

#[test]
fn derive_tool_name_falls_back_to_method_and_path() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "DELETE", "/users/{id}");
    // Should contain "delete" and path segments, with braces stripped
    assert!(name.starts_with("delete_"), "got: {name}");
    assert!(name.contains("users"), "got: {name}");
    assert!(!name.contains('{'), "braces should be stripped: {name}");
    assert!(!name.contains('}'), "braces should be stripped: {name}");
}

#[test]
fn derive_tool_name_nested_path_fallback() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/api/v1/items");
    assert_eq!(name, "get_api_v1_items");
}

// =========================================================================
// extract_param_type
// =========================================================================

#[test]
fn extract_param_type_returns_type_from_schema() {
    let schema = json!({"type": "integer"});
    assert_eq!(extract_param_type(Some(&schema)), "integer");
}

#[test]
fn extract_param_type_defaults_to_string_when_none() {
    assert_eq!(extract_param_type(None), "string");
}

#[test]
fn extract_param_type_defaults_to_string_when_no_type_field() {
    let schema = json!({"format": "date"});
    assert_eq!(extract_param_type(Some(&schema)), "string");
}

// =========================================================================
// build_parameters_schema
// =========================================================================

#[test]
fn build_parameters_schema_from_query_and_path_params() {
    let op = OpenApiOperation {
        operation_id: Some("test".to_string()),
        summary: None,
        description: None,
        parameters: vec![
            OpenApiParameter {
                name: "id".to_string(),
                location: "path".to_string(),
                description: Some("Resource ID".to_string()),
                required: true,
                schema: Some(json!({"type": "string"})),
            },
            OpenApiParameter {
                name: "verbose".to_string(),
                location: "query".to_string(),
                description: None,
                required: false,
                schema: Some(json!({"type": "boolean"})),
            },
        ],
        request_body: None,
        tags: vec![],
    };

    let schema = build_parameters_schema(&op);
    assert_eq!(schema.schema_type, "object");

    let props = schema.properties.as_ref().expect("should have properties");
    assert_eq!(props.len(), 2);
    assert_eq!(props["id"].property_type, "string");
    assert_eq!(props["id"].description, Some("Resource ID".to_string()));
    assert_eq!(props["verbose"].property_type, "boolean");

    let required = schema.required.as_ref().expect("should have required");
    assert!(required.contains(&"id".to_string()));
    assert!(!required.contains(&"verbose".to_string()));
}

#[test]
fn build_parameters_schema_empty_operation() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };

    let schema = build_parameters_schema(&op);
    assert_eq!(schema.schema_type, "object");
    assert!(schema.properties.is_none());
    assert!(schema.required.is_none());
}

#[test]
fn build_parameters_schema_merges_body_and_params() {
    let op = OpenApiOperation {
        operation_id: Some("create".to_string()),
        summary: None,
        description: None,
        parameters: vec![OpenApiParameter {
            name: "org_id".to_string(),
            location: "path".to_string(),
            description: None,
            required: true,
            schema: Some(json!({"type": "string"})),
        }],
        request_body: Some(OpenApiRequestBody {
            description: None,
            required: true,
            content: {
                let mut m = HashMap::new();
                m.insert(
                    "application/json".to_string(),
                    OpenApiMediaType {
                        schema: Some(json!({
                            "type": "object",
                            "properties": {
                                "title": {"type": "string", "description": "Item title"},
                                "count": {"type": "integer"}
                            },
                            "required": ["title"]
                        })),
                    },
                );
                m
            },
        }),
        tags: vec![],
    };

    let schema = build_parameters_schema(&op);
    let props = schema.properties.as_ref().unwrap();
    // Should have path param + body params
    assert_eq!(props.len(), 3);
    assert!(props.contains_key("org_id"));
    assert!(props.contains_key("title"));
    assert!(props.contains_key("count"));
    assert_eq!(props["title"].description, Some("Item title".to_string()));

    let required = schema.required.as_ref().unwrap();
    assert!(required.contains(&"org_id".to_string()));
    assert!(required.contains(&"title".to_string()));
    assert!(!required.contains(&"count".to_string()));
}

// =========================================================================
// HttpMethod serde — verify UPPERCASE rename
// =========================================================================

#[test]
fn http_method_serde_all_variants_uppercase() {
    let variants = vec![
        (HttpMethod::Get, "\"GET\""),
        (HttpMethod::Post, "\"POST\""),
        (HttpMethod::Put, "\"PUT\""),
        (HttpMethod::Delete, "\"DELETE\""),
        (HttpMethod::Patch, "\"PATCH\""),
        (HttpMethod::Head, "\"HEAD\""),
    ];
    for (variant, expected_json) in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            serialized, expected_json,
            "serialize mismatch for {:?}",
            variant
        );
        let deserialized: HttpMethod = serde_json::from_str(expected_json).unwrap();
        assert_eq!(
            deserialized, variant,
            "deserialize mismatch for {expected_json}"
        );
    }
}

// =========================================================================
// RestResource — URL joining edge cases
// =========================================================================

#[test]
fn rest_resource_trims_trailing_slash_before_joining() {
    let r = RestResource::new("https://api.example.com///");
    // trim_end_matches('/') strips all trailing slashes
    let req = r.request(HttpMethod::Get, "/items");
    assert_eq!(req.url, "https://api.example.com/items");
}

#[test]
fn rest_resource_serialization_round_trip() {
    let mut r = RestResource::new("https://api.test.io");
    r.auth = Some(HttpAuth::ApiKey {
        key: "secret".to_string(),
        header: Some("X-Token".to_string()),
    });
    r.headers
        .insert("Accept".to_string(), "text/xml".to_string());

    let json = serde_json::to_string(&r).unwrap();
    let back: RestResource = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base_url, "https://api.test.io");
    assert!(back.auth.is_some());
    assert_eq!(back.headers.get("Accept").unwrap(), "text/xml");
}

// =========================================================================
// OpenAPI — Swagger 2.x document parsing
// =========================================================================

#[test]
fn discover_tools_from_swagger2_document() {
    let json = json!({
        "swagger": "2.0",
        "info": {"title": "Legacy API", "version": "0.1"},
        "paths": {
            "/health": {
                "get": {
                    "operationId": "healthCheck",
                    "summary": "Health endpoint"
                }
            }
        }
    })
    .to_string();

    let tools = discover_tools_from_openapi(&json).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "healthcheck");
    assert_eq!(tools[0].method, "GET");
    assert_eq!(tools[0].path, "/health");
}
