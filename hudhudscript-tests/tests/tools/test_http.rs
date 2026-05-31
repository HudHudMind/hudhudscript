use hudhudscript_tools::http::{
    HttpAuth, HttpMethod, HttpRequest, HttpResponse, HttpTool, HttpToolError, RestResource,
};
use std::collections::HashMap;

#[test]
fn test_http_method_display() {
    assert_eq!(HttpMethod::Get.to_string(), "GET");
    assert_eq!(HttpMethod::Post.to_string(), "POST");
    assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
}

#[test]
fn test_rest_resource_request_building() {
    let resource = RestResource::new("https://api.example.com");
    let req = resource.request(HttpMethod::Get, "/users");
    assert_eq!(req.url, "https://api.example.com/users");
    assert_eq!(req.method, HttpMethod::Get);
}

#[test]
fn test_rest_resource_trailing_slash() {
    let resource = RestResource::new("https://api.example.com/");
    let req = resource.request(HttpMethod::Post, "/items");
    assert_eq!(req.url, "https://api.example.com/items");
}

#[test]
fn test_http_auth_bearer() {
    let auth = HttpAuth::Bearer {
        token: "my-token".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("bearer"));
}

#[test]
fn test_http_method_display_all_variants() {
    assert_eq!(HttpMethod::Put.to_string(), "PUT");
    assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    assert_eq!(HttpMethod::Head.to_string(), "HEAD");
}

#[test]
fn test_http_auth_api_key_serialization() {
    let auth = HttpAuth::ApiKey {
        key: "secret-key".to_string(),
        header: Some("X-Custom-Key".to_string()),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("api_key"));
    assert!(json.contains("secret-key"));
    assert!(json.contains("X-Custom-Key"));
}

#[test]
fn test_http_auth_basic_serialization() {
    let auth = HttpAuth::Basic {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("basic"));
    assert!(json.contains("user"));
}

#[test]
fn test_rest_resource_new() {
    let resource = RestResource::new("https://api.example.com");
    assert_eq!(resource.base_url, "https://api.example.com");
    assert!(resource.auth.is_none());
    assert!(resource.headers.is_empty());
}

#[test]
fn test_rest_resource_request_inherits_headers() {
    let mut resource = RestResource::new("https://api.example.com");
    resource
        .headers
        .insert("Accept".to_string(), "application/json".to_string());

    let req = resource.request(HttpMethod::Get, "/data");
    assert_eq!(req.headers["Accept"], "application/json");
}

#[test]
fn test_rest_resource_request_inherits_auth() {
    let mut resource = RestResource::new("https://api.example.com");
    resource.auth = Some(HttpAuth::Bearer {
        token: "test-token".to_string(),
    });

    let req = resource.request(HttpMethod::Post, "/submit");
    assert!(req.auth.is_some());
}

#[test]
fn test_rest_resource_request_defaults() {
    let resource = RestResource::new("https://api.example.com");
    let req = resource.request(HttpMethod::Delete, "/resource/1");
    assert_eq!(req.url, "https://api.example.com/resource/1");
    assert_eq!(req.method, HttpMethod::Delete);
    assert!(req.body.is_none());
    assert!(req.timeout_secs.is_none());
    assert!(req.retries.is_none());
}

#[test]
fn test_http_tool_default() {
    let tool = HttpTool::default();
    // Just verify it can be created without panicking
    let _ = tool;
}

#[test]
fn test_http_request_serialization() {
    let req = HttpRequest {
        method: HttpMethod::Get,
        url: "https://example.com".to_string(),
        headers: HashMap::new(),
        auth: None,
        body: None,
        timeout_secs: Some(10),
        retries: Some(3),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: HttpRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.url, "https://example.com");
    assert_eq!(deserialized.timeout_secs, Some(10));
    assert_eq!(deserialized.retries, Some(3));
}

#[test]
fn test_http_response_ok_field() {
    let resp = HttpResponse {
        status: 200,
        headers: HashMap::new(),
        body: serde_json::json!(null),
        ok: true,
    };
    assert!(resp.ok);
    assert_eq!(resp.status, 200);

    let resp_err = HttpResponse {
        status: 404,
        headers: HashMap::new(),
        body: serde_json::json!(null),
        ok: false,
    };
    assert!(!resp_err.ok);
    assert_eq!(resp_err.status, 404);
}

#[test]
fn test_http_tool_error_display() {
    let err = HttpToolError::RequestFailed("connection refused".to_string());
    assert!(err
        .to_string()
        .contains("HTTP request failed: connection refused"));

    let err = HttpToolError::InvalidUrl("bad://url".to_string());
    assert!(err.to_string().contains("Invalid URL: bad://url"));

    let err = HttpToolError::Timeout(30);
    assert!(err.to_string().contains("Timeout after 30s"));

    let err = HttpToolError::ParseError("unexpected EOF".to_string());
    assert!(err
        .to_string()
        .contains("Response parse error: unexpected EOF"));
}

#[test]
fn test_http_method_serde_roundtrip() {
    let methods = vec![
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
    ];
    for method in methods {
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: HttpMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, method);
    }
}

#[test]
fn test_http_auth_api_key_default_header() {
    let auth = HttpAuth::ApiKey {
        key: "my-key".to_string(),
        header: None,
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("api_key"));
    // When header is None, default is X-API-Key (tested in execute path)
    let deserialized: HttpAuth = serde_json::from_str(&json).unwrap();
    match deserialized {
        HttpAuth::ApiKey { key, header } => {
            assert_eq!(key, "my-key");
            assert!(header.is_none());
        }
        _ => panic!("Expected ApiKey variant"),
    }
}

#[test]
fn test_rest_resource_with_auth_and_headers() {
    let mut resource = RestResource::new("https://api.example.com");
    resource.auth = Some(HttpAuth::Basic {
        username: "user".to_string(),
        password: "pass".to_string(),
    });
    resource
        .headers
        .insert("X-Custom".to_string(), "value".to_string());

    let req = resource.request(HttpMethod::Put, "/update");
    assert_eq!(req.url, "https://api.example.com/update");
    assert_eq!(req.method, HttpMethod::Put);
    assert!(req.auth.is_some());
    assert_eq!(req.headers.get("X-Custom").unwrap(), "value");
}

#[test]
fn test_http_response_serialization_roundtrip() {
    let resp = HttpResponse {
        status: 201,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h
        },
        body: serde_json::json!({"created": true}),
        ok: true,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: HttpResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.status, 201);
    assert!(deserialized.ok);
    assert_eq!(deserialized.headers["content-type"], "application/json");
}

#[test]
fn test_http_request_with_body() {
    let req = HttpRequest {
        method: HttpMethod::Post,
        url: "https://example.com/api".to_string(),
        headers: HashMap::new(),
        auth: Some(HttpAuth::Bearer {
            token: "tok123".to_string(),
        }),
        body: Some(serde_json::json!({"key": "value"})),
        timeout_secs: Some(60),
        retries: Some(2),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: HttpRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.method, HttpMethod::Post);
    assert!(deserialized.body.is_some());
    assert_eq!(deserialized.timeout_secs, Some(60));
    assert_eq!(deserialized.retries, Some(2));
}

#[test]
fn test_http_tool_new() {
    let tool = HttpTool::new();
    // Should construct without panicking
    let _ = tool;
}
