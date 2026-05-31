use hudhudscript_tools_net::*;
use serde_json::json;
use std::collections::HashMap;

// =========================================================================
// HttpMethod
// =========================================================================

#[test]
fn http_method_display_get() {
    assert_eq!(format!("{}", HttpMethod::Get), "GET");
}

#[test]
fn http_method_display_post() {
    assert_eq!(format!("{}", HttpMethod::Post), "POST");
}

#[test]
fn http_method_display_put() {
    assert_eq!(format!("{}", HttpMethod::Put), "PUT");
}

#[test]
fn http_method_display_delete() {
    assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
}

#[test]
fn http_method_display_patch() {
    assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
}

#[test]
fn http_method_display_head() {
    assert_eq!(format!("{}", HttpMethod::Head), "HEAD");
}

#[test]
fn http_method_eq() {
    assert_eq!(HttpMethod::Get, HttpMethod::Get);
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
}

#[test]
fn http_method_clone() {
    let m = HttpMethod::Patch;
    let m2 = m.clone();
    assert_eq!(m, m2);
}

#[test]
fn http_method_serialize_deserialize() {
    let m = HttpMethod::Get;
    let json = serde_json::to_string(&m).unwrap();
    let back: HttpMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(back, HttpMethod::Get);
}

// =========================================================================
// HttpAuth
// =========================================================================

#[test]
fn http_auth_bearer() {
    let auth = HttpAuth::Bearer {
        token: "tok123".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("bearer"));
    assert!(json.contains("tok123"));
}

#[test]
fn http_auth_api_key_default_header() {
    let auth = HttpAuth::ApiKey {
        key: "key123".to_string(),
        header: None,
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("api_key"));
}

#[test]
fn http_auth_api_key_custom_header() {
    let auth = HttpAuth::ApiKey {
        key: "key456".to_string(),
        header: Some("X-Custom-Key".to_string()),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("X-Custom-Key"));
}

#[test]
fn http_auth_basic() {
    let auth = HttpAuth::Basic {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("basic"));
    assert!(json.contains("user"));
}

#[test]
fn http_auth_deserialize_bearer() {
    let json = r#"{"type":"bearer","token":"abc"}"#;
    let auth: HttpAuth = serde_json::from_str(json).unwrap();
    if let HttpAuth::Bearer { token } = auth {
        assert_eq!(token, "abc");
    } else {
        panic!("expected Bearer");
    }
}

// =========================================================================
// HttpRequest
// =========================================================================

#[test]
fn http_request_construction() {
    let req = HttpRequest {
        method: HttpMethod::Post,
        url: "https://example.com/api".to_string(),
        headers: HashMap::new(),
        auth: None,
        body: Some(json!({"key": "value"})),
        timeout_secs: Some(60),
        retries: Some(3),
    };
    assert_eq!(req.method, HttpMethod::Post);
    assert_eq!(req.url, "https://example.com/api");
    assert_eq!(req.timeout_secs, Some(60));
}

#[test]
fn http_request_serialize_deserialize() {
    let req = HttpRequest {
        method: HttpMethod::Get,
        url: "https://api.test.com".to_string(),
        headers: HashMap::new(),
        auth: None,
        body: None,
        timeout_secs: None,
        retries: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: HttpRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.url, "https://api.test.com");
}

#[test]
fn http_request_with_headers() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());
    let req = HttpRequest {
        method: HttpMethod::Get,
        url: "https://example.com".to_string(),
        headers,
        auth: None,
        body: None,
        timeout_secs: None,
        retries: None,
    };
    assert_eq!(req.headers.len(), 2);
}

// =========================================================================
// HttpResponse
// =========================================================================

#[test]
fn http_response_ok_true_for_200() {
    let resp = HttpResponse {
        status: 200,
        headers: HashMap::new(),
        body: json!({"result": "ok"}),
        ok: true,
    };
    assert!(resp.ok);
    assert_eq!(resp.status, 200);
}

#[test]
fn http_response_ok_false_for_404() {
    let resp = HttpResponse {
        status: 404,
        headers: HashMap::new(),
        body: json!("not found"),
        ok: false,
    };
    assert!(!resp.ok);
    assert_eq!(resp.status, 404);
}

#[test]
fn http_response_serialize_deserialize() {
    let resp = HttpResponse {
        status: 201,
        headers: HashMap::new(),
        body: json!({"id": 42}),
        ok: true,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: HttpResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, 201);
}

// =========================================================================
// HttpToolError
// =========================================================================

#[test]
fn http_tool_error_display_request_failed() {
    let err = HttpToolError::RequestFailed("connection refused".to_string());
    assert!(format!("{err}").contains("connection refused"));
}

#[test]
fn http_tool_error_display_parse_error() {
    let err = HttpToolError::ParseError("invalid json".to_string());
    assert!(format!("{err}").contains("invalid json"));
}

#[test]
fn http_tool_error_display_invalid_url() {
    let err = HttpToolError::InvalidUrl("not-a-url".to_string());
    assert!(format!("{err}").contains("not-a-url"));
}

#[test]
fn http_tool_error_display_timeout() {
    let err = HttpToolError::Timeout(30);
    assert!(format!("{err}").contains("30"));
}

// =========================================================================
// HttpTool
// =========================================================================

#[test]
fn http_tool_new() {
    let _tool = HttpTool::new();
}

#[test]
fn http_tool_default() {
    let _tool = HttpTool::default();
}

// =========================================================================
// RestResource
// =========================================================================

#[test]
fn rest_resource_new() {
    let r = RestResource::new("https://api.example.com");
    assert_eq!(r.base_url, "https://api.example.com");
    assert!(r.auth.is_none());
    assert!(r.headers.is_empty());
}

#[test]
fn rest_resource_request_get() {
    let r = RestResource::new("https://api.example.com");
    let req = r.request(HttpMethod::Get, "/users");
    assert_eq!(req.url, "https://api.example.com/users");
    assert_eq!(req.method, HttpMethod::Get);
}

#[test]
fn rest_resource_request_post() {
    let r = RestResource::new("https://api.example.com/");
    let req = r.request(HttpMethod::Post, "/items");
    assert_eq!(req.url, "https://api.example.com/items");
    assert_eq!(req.method, HttpMethod::Post);
}

#[test]
fn rest_resource_inherits_auth() {
    let mut r = RestResource::new("https://api.example.com");
    r.auth = Some(HttpAuth::Bearer {
        token: "secret".to_string(),
    });
    let req = r.request(HttpMethod::Get, "/data");
    assert!(req.auth.is_some());
}

#[test]
fn rest_resource_inherits_headers() {
    let mut r = RestResource::new("https://api.example.com");
    r.headers
        .insert("X-Custom".to_string(), "value".to_string());
    let req = r.request(HttpMethod::Get, "/data");
    assert_eq!(req.headers.get("X-Custom").unwrap(), "value");
}

#[test]
fn rest_resource_request_defaults() {
    let r = RestResource::new("https://api.example.com");
    let req = r.request(HttpMethod::Delete, "/item/1");
    assert!(req.body.is_none());
    assert!(req.timeout_secs.is_none());
    assert!(req.retries.is_none());
}

// =========================================================================
// OpenAPI — discover_tools_from_openapi
// =========================================================================

fn sample_openapi_json() -> String {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/users": {
                "get": {
                    "operationId": "listUsers",
                    "summary": "List all users",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer" }
                        }
                    ],
                    "tags": ["users"]
                },
                "post": {
                    "operationId": "createUser",
                    "summary": "Create a user",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "name": { "type": "string", "description": "User name" },
                                        "email": { "type": "string" }
                                    },
                                    "required": ["name"]
                                }
                            }
                        }
                    },
                    "tags": ["users"]
                }
            },
            "/users/{id}": {
                "get": {
                    "operationId": "getUser",
                    "summary": "Get user by ID",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ]
                },
                "delete": {
                    "summary": "Delete a user",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ]
                }
            }
        }
    })
    .to_string()
}

#[test]
fn discover_tools_basic() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    assert!(tools.len() >= 4);
}

#[test]
fn discover_tools_operation_id_used_as_name() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"listusers"));
    assert!(names.contains(&"createuser"));
    assert!(names.contains(&"getuser"));
}

#[test]
fn discover_tools_fallback_name_from_path() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    // The DELETE on /users/{id} has no operationId
    let delete_tool = tools.iter().find(|t| t.method == "DELETE").unwrap();
    assert!(!delete_tool.name.is_empty());
    assert!(delete_tool.name.contains("delete"));
}

#[test]
fn discover_tools_description() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let list = tools.iter().find(|t| t.name == "listusers").unwrap();
    assert_eq!(list.description, Some("List all users".to_string()));
}

#[test]
fn discover_tools_parameters_from_query() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let list = tools.iter().find(|t| t.name == "listusers").unwrap();
    let props = list.parameters.properties.as_ref().unwrap();
    assert!(props.contains_key("limit"));
}

#[test]
fn discover_tools_parameters_from_body() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let create = tools.iter().find(|t| t.name == "createuser").unwrap();
    let props = create.parameters.properties.as_ref().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("email"));
    // "name" should be required
    let required = create.parameters.required.as_ref().unwrap();
    assert!(required.contains(&"name".to_string()));
}

#[test]
fn discover_tools_tags() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let list = tools.iter().find(|t| t.name == "listusers").unwrap();
    assert!(list.tags.contains(&"users".to_string()));
}

#[test]
fn discover_tools_method_populated() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let get_tool = tools.iter().find(|t| t.name == "getuser").unwrap();
    assert_eq!(get_tool.method, "GET");
}

#[test]
fn discover_tools_path_populated() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let get_tool = tools.iter().find(|t| t.name == "getuser").unwrap();
    assert_eq!(get_tool.path, "/users/{id}");
}

#[test]
fn discover_tools_invalid_json() {
    let result = discover_tools_from_openapi("not valid json");
    assert!(result.is_err());
}

#[test]
fn discover_tools_empty_paths() {
    let json = json!({
        "openapi": "3.1.0",
        "info": { "title": "Empty", "version": "1.0" },
        "paths": {}
    })
    .to_string();
    let tools = discover_tools_from_openapi(&json).unwrap();
    assert!(tools.is_empty());
}

#[test]
fn discover_tools_required_path_param() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let get_user = tools.iter().find(|t| t.name == "getuser").unwrap();
    let required = get_user.parameters.required.as_ref().unwrap();
    assert!(required.contains(&"id".to_string()));
}

// =========================================================================
// OpenApiDocument deserialization
// =========================================================================

#[test]
fn openapi_document_with_openapi_version() {
    let json = json!({
        "openapi": "3.1.0",
        "info": { "title": "My API", "version": "2.0" },
        "paths": {}
    })
    .to_string();
    let doc: OpenApiDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.openapi, Some("3.1.0".to_string()));
}

#[test]
fn openapi_document_with_swagger_version() {
    let json = json!({
        "swagger": "2.0",
        "info": { "title": "Old API" },
        "paths": {}
    })
    .to_string();
    let doc: OpenApiDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.swagger, Some("2.0".to_string()));
}

#[test]
fn openapi_document_info_fields() {
    let json = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Cool API",
            "description": "A cool API",
            "version": "1.2.3"
        },
        "paths": {}
    })
    .to_string();
    let doc: OpenApiDocument = serde_json::from_str(&json).unwrap();
    let info = doc.info.unwrap();
    assert_eq!(info.title, Some("Cool API".to_string()));
    assert_eq!(info.description, Some("A cool API".to_string()));
    assert_eq!(info.version, Some("1.2.3".to_string()));
}

// =========================================================================
// DiscoveredTool serialization
// =========================================================================

#[test]
fn discovered_tool_serialize_deserialize() {
    let tools = discover_tools_from_openapi(&sample_openapi_json()).unwrap();
    let tool = &tools[0];
    let json = serde_json::to_string(tool).unwrap();
    let back: DiscoveredTool = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, tool.name);
}
