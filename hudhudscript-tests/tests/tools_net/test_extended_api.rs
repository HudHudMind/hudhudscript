//! Extended unit tests for hudhudscript-tools-net
//! Covers: OpenApiDocument edge cases, OpenApiInfo, sanitize_name, discover_tools

use hudhudscript_tools_net::openapi::*;
use hudhudscript_tools_net::*;
use hudhudscript_tools_schema::registry::RegistryError;
use hudhudscript_tools_schema::schema::JsonSchema;
use std::collections::HashMap;

// ── OpenApiDocument edge cases ──────────────────────────────────────

#[test]
fn openapi_document_both_openapi_and_swagger_none() {
    let doc = OpenApiDocument {
        openapi: None,
        swagger: None,
        info: None,
        paths: HashMap::new(),
    };
    assert!(doc.openapi.is_none());
    assert!(doc.swagger.is_none());
}

#[test]
fn openapi_document_with_info() {
    let info = OpenApiInfo {
        title: Some("Test API".to_string()),
        version: Some("1.0.0".to_string()),
        description: Some("A test API".to_string()),
    };
    let doc = OpenApiDocument {
        openapi: Some("3.0.0".to_string()),
        swagger: None,
        info: Some(info),
        paths: HashMap::new(),
    };
    assert_eq!(
        doc.info.as_ref().unwrap().title.as_deref(),
        Some("Test API")
    );
    assert_eq!(doc.info.as_ref().unwrap().version.as_deref(), Some("1.0.0"));
}

#[test]
fn openapi_document_multiple_paths() {
    let mut paths = HashMap::new();
    for i in 0..5 {
        paths.insert(format!("/path/{}", i), OpenApiPathItem::default());
    }
    let doc = OpenApiDocument {
        openapi: Some("3.0.0".to_string()),
        swagger: None,
        info: None,
        paths,
    };
    assert_eq!(doc.paths.len(), 5);
}

// ── OpenApiInfo ─────────────────────────────────────────────────────

#[test]
fn openapi_info_all_none() {
    let info = OpenApiInfo {
        title: None,
        version: None,
        description: None,
    };
    assert!(info.title.is_none());
    assert!(info.version.is_none());
    assert!(info.description.is_none());
}

#[test]
fn openapi_info_full_fields() {
    let info = OpenApiInfo {
        title: Some("My API".to_string()),
        version: Some("2.0.0".to_string()),
        description: Some("Description here".to_string()),
    };
    assert_eq!(info.title.as_deref(), Some("My API"));
    assert_eq!(info.version.as_deref(), Some("2.0.0"));
    assert_eq!(info.description.as_deref(), Some("Description here"));
}

// ── OpenApiPathItem ─────────────────────────────────────────────────

#[test]
fn openapi_path_item_default_all_none() {
    let path = OpenApiPathItem::default();
    assert!(path.get.is_none());
    assert!(path.post.is_none());
    assert!(path.put.is_none());
    assert!(path.delete.is_none());
    assert!(path.patch.is_none());
}

#[test]
fn openapi_path_item_with_get_operation() {
    let operation = OpenApiOperation {
        operation_id: Some("listUsers".to_string()),
        summary: Some("List all users".to_string()),
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let path = OpenApiPathItem {
        get: Some(operation),
        ..Default::default()
    };
    assert!(path.get.is_some());
    assert_eq!(
        path.get.as_ref().unwrap().operation_id.as_deref(),
        Some("listUsers")
    );
    assert!(path.post.is_none());
}

// ── OpenApiOperation parameters ─────────────────────────────────────

#[test]
fn openapi_operation_with_parameters() {
    let params = vec![OpenApiParameter {
        name: "limit".to_string(),
        location: "query".to_string(),
        description: None,
        required: false,
        schema: None,
    }];
    let operation = OpenApiOperation {
        operation_id: Some("search".to_string()),
        parameters: params,
        summary: None,
        description: None,
        request_body: None,
        tags: vec![],
    };
    assert_eq!(operation.parameters.len(), 1);
    assert_eq!(operation.parameters[0].name, "limit");
}

#[test]
fn openapi_operation_empty_vec_defaults() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    assert!(op.parameters.is_empty());
    assert!(op.tags.is_empty());
    assert!(op.operation_id.is_none());
}

#[test]
fn openapi_operation_with_tags() {
    let op = OpenApiOperation {
        operation_id: Some("getPets".to_string()),
        tags: vec!["pets".to_string(), "public".to_string()],
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
    };
    assert_eq!(op.tags.len(), 2);
    assert!(op.tags.contains(&"pets".to_string()));
}

// ── OpenApiParameter ────────────────────────────────────────────────

#[test]
fn openapi_parameter_path_required() {
    let param = OpenApiParameter {
        name: "userId".to_string(),
        location: "path".to_string(),
        required: true,
        description: Some("User identifier".to_string()),
        schema: None,
    };
    assert_eq!(param.name, "userId");
    assert_eq!(param.location, "path");
    assert!(param.required);
}

#[test]
fn openapi_parameter_header_optional() {
    let param = OpenApiParameter {
        name: "Authorization".to_string(),
        location: "header".to_string(),
        required: false,
        description: None,
        schema: None,
    };
    assert_eq!(param.location, "header");
    assert!(!param.required);
}

// ── OpenApiRequestBody ──────────────────────────────────────────────

#[test]
fn openapi_request_body_json() {
    let mut content = HashMap::new();
    content.insert(
        "application/json".to_string(),
        OpenApiMediaType { schema: None },
    );
    let body = OpenApiRequestBody {
        required: true,
        description: Some("Payload".to_string()),
        content,
    };
    assert!(body.required);
    assert!(body.content.contains_key("application/json"));
}

// ── DiscoveredTool ──────────────────────────────────────────────────

#[test]
fn discovered_tool_fields() {
    let tool = DiscoveredTool {
        name: "listUsers".to_string(),
        description: Some("List all users".to_string()),
        parameters: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        method: "GET".to_string(),
        path: "/users".to_string(),
        tags: vec!["users".to_string()],
    };
    assert_eq!(tool.name, "listUsers");
    assert_eq!(tool.method, "GET");
    assert_eq!(tool.path, "/users");
    assert_eq!(tool.tags.len(), 1);
}

// ── sanitize_name edge cases ────────────────────────────────────────

#[test]
fn sanitize_name_empty_string() {
    let name = sanitize_name("");
    // Should not panic on empty string
    let _ = name;
}

#[test]
fn sanitize_name_already_clean() {
    let name = sanitize_name("listUsers");
    // sanitize_name lowercases as part of its normalization
    assert_eq!(name, "listusers");
}

#[test]
fn sanitize_name_with_curly_braces() {
    let name = sanitize_name("/users/{id}/posts");
    // Curly braces should be handled
    assert!(!name.is_empty());
}

#[test]
fn sanitize_name_with_dots() {
    let name = sanitize_name("get.user.profile");
    assert!(!name.is_empty());
}

#[test]
fn sanitize_name_with_hyphens() {
    let name = sanitize_name("get-users-list");
    assert!(!name.is_empty());
}

#[test]
fn sanitize_name_is_deterministic() {
    let input = "get-users/{id}/profile";
    let result1 = sanitize_name(input);
    let result2 = sanitize_name(input);
    assert_eq!(result1, result2);
}

// ── derive_tool_name ────────────────────────────────────────────────

#[test]
fn derive_tool_name_uses_operation_id() {
    let op = OpenApiOperation {
        operation_id: Some("myCustomName".to_string()),
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/users");
    assert!(!name.is_empty());
}

#[test]
fn derive_tool_name_falls_back_to_path() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/users");
    assert!(!name.is_empty());
}

#[test]
fn derive_tool_name_post_method() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "POST", "/items");
    assert!(!name.is_empty());
}

// ── discover_tools_from_openapi ─────────────────────────────────────

#[test]
fn discover_tools_empty_spec() {
    let json = r#"{"openapi":"3.0.0","info":{"title":"Test"},"paths":{}}"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert!(tools.is_empty());
}

#[test]
fn discover_tools_single_get_endpoint() {
    let json = r#"{
        "openapi":"3.0.0",
        "info":{"title":"TestServer"},
        "paths":{
            "/users":{
                "get":{
                    "operationId":"listUsers",
                    "summary":"List users",
                    "parameters":[],
                    "tags":["users"]
                }
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "listusers");
    assert_eq!(tools[0].method, "GET");
    assert_eq!(tools[0].path, "/users");
}

#[test]
fn discover_tools_multiple_endpoints() {
    let json = r#"{
        "openapi":"3.0.0",
        "info":{"title":"TestServer"},
        "paths":{
            "/users":{
                "get":{"operationId":"listUsers","parameters":[]},
                "post":{"operationId":"createUser","parameters":[]}
            },
            "/items":{
                "get":{"operationId":"listItems","parameters":[]}
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 3);
}

#[test]
fn discover_tools_swagger_v2() {
    let json = r#"{
        "swagger":"2.0",
        "info":{"title":"LegacyAPI"},
        "paths":{
            "/v1/data":{
                "get":{"operationId":"getData","parameters":[]}
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "getdata");
}

#[test]
fn discover_tools_invalid_json() {
    let json = "not json at all";
    let result = discover_tools_from_openapi(json);
    assert!(result.is_err());
}

#[test]
fn discover_tools_without_info_uses_default() {
    let json = r#"{
        "openapi":"3.0.0",
        "paths":{
            "/health":{
                "get":{"operationId":"healthCheck","parameters":[]}
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 1);
}

#[test]
fn discover_tools_with_query_parameters() {
    let json = r#"{
        "openapi":"3.0.0",
        "info":{"title":"SearchAPI"},
        "paths":{
            "/search":{
                "get":{
                    "operationId":"search",
                    "parameters":[
                        {"name":"q","in":"query","required":true},
                        {"name":"limit","in":"query","required":false}
                    ]
                }
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "search");
}

#[test]
fn discover_tools_skips_path_item_without_operations() {
    let json = r#"{
        "openapi":"3.0.0",
        "info":{"title":"Test"},
        "paths":{
            "/empty":{}
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert!(tools.is_empty());
}

// ── HTTP types ──────────────────────────────────────────────────────

#[test]
fn http_request_get_method() {
    let request = HttpRequest {
        method: HttpMethod::Get,
        url: "https://api.example.com/data".to_string(),
        headers: HashMap::new(),
        auth: None,
        body: None,
        timeout_secs: None,
        retries: None,
    };
    assert_eq!(request.method, HttpMethod::Get);
    assert!(request.body.is_none());
}

#[test]
fn http_request_post_with_body() {
    let request = HttpRequest {
        method: HttpMethod::Post,
        url: "https://api.example.com/data".to_string(),
        headers: HashMap::new(),
        auth: None,
        body: Some(serde_json::json!({"key": "value"})),
        timeout_secs: None,
        retries: None,
    };
    assert_eq!(request.method, HttpMethod::Post);
    assert!(request.body.is_some());
}

#[test]
fn http_response_ok() {
    let response = HttpResponse {
        status: 200,
        headers: HashMap::new(),
        body: serde_json::json!("OK"),
        ok: true,
    };
    assert_eq!(response.status, 200);
    assert!(response.ok);
}

#[test]
fn http_response_error() {
    let response = HttpResponse {
        status: 404,
        headers: HashMap::new(),
        body: serde_json::json!({"error": "Not Found"}),
        ok: false,
    };
    assert_eq!(response.status, 404);
    assert!(!response.ok);
}

#[test]
fn http_method_display() {
    assert_eq!(format!("{}", HttpMethod::Get), "GET");
    assert_eq!(format!("{}", HttpMethod::Post), "POST");
    assert_eq!(format!("{}", HttpMethod::Put), "PUT");
    assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
    assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
    assert_eq!(format!("{}", HttpMethod::Head), "HEAD");
}

// ── OpenApiError Display ────────────────────────────────────────────

#[test]
fn openapi_error_parse_error_display() {
    let err = OpenApiError::ParseError("bad spec".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("bad spec"));
}

#[test]
fn openapi_error_from_registry_error() {
    // Construct via From trait
    let err: OpenApiError = RegistryError::ToolNotFound("myTool".to_string()).into();
    assert!(matches!(err, OpenApiError::RegistryError(_)));
}
