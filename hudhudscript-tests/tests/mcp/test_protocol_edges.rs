//! Edge-case tests for MCP protocol types: deserialization from raw JSON
//! strings, skip_serializing_if behavior, Content enum variant coverage,
//! and error code constants.

use hudhudscript_mcp::{
    ClientCapabilities, ClientInfo, Content, InitializeRequest, InitializeResponse, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse, PromptsCapability, RequestId, Resource, ResourceContent,
    ResourceReadRequest, ResourceReadResponse, ResourcesCapability, ResourcesListRequest,
    ResourcesListResponse, ServerCapabilities, ServerInfo, Tool, ToolCallRequest, ToolCallResponse,
    ToolsCapability, ToolsListRequest, ToolsListResponse,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Content enum — full variant deserialization from raw JSON
// ---------------------------------------------------------------------------

#[test]
fn content_text_from_json() {
    let raw = r#"{"type":"text","text":"hello world"}"#;
    let c: Content = serde_json::from_str(raw).expect("should deserialize text content");
    match c {
        Content::Text { text } => assert_eq!(text, "hello world"),
        _ => panic!("expected Text variant"),
    }
}

#[test]
fn content_image_from_json() {
    let raw = r#"{"type":"image","data":"aGVsbG8=","mime_type":"image/png"}"#;
    let c: Content = serde_json::from_str(raw).expect("should deserialize image content");
    match c {
        Content::Image { data, mime_type } => {
            assert_eq!(data, "aGVsbG8=");
            assert_eq!(mime_type, "image/png");
        }
        _ => panic!("expected Image variant"),
    }
}

#[test]
fn content_resource_from_json() {
    let raw = r#"{
        "type": "resource",
        "resource": {
            "uri": "file:///data.csv",
            "mimeType": "text/csv",
            "text": "a,b,c"
        }
    }"#;
    let c: Content = serde_json::from_str(raw).expect("should deserialize resource content");
    match c {
        Content::Resource { resource } => {
            assert_eq!(resource.uri, "file:///data.csv");
            assert_eq!(resource.mime_type.as_deref(), Some("text/csv"));
            assert_eq!(resource.text.as_deref(), Some("a,b,c"));
            assert!(resource.blob.is_none());
        }
        _ => panic!("expected Resource variant"),
    }
}

#[test]
fn content_image_roundtrip() {
    let original = Content::Image {
        data: "base64data".to_string(),
        mime_type: "image/jpeg".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let back: Content = serde_json::from_str(&json).unwrap();
    match back {
        Content::Image { data, mime_type } => {
            assert_eq!(data, "base64data");
            assert_eq!(mime_type, "image/jpeg");
        }
        _ => panic!("expected Image"),
    }
}

#[test]
fn content_resource_roundtrip() {
    let original = Content::Resource {
        resource: ResourceContent {
            uri: "s3://bucket/key".to_string(),
            mime_type: Some("application/octet-stream".to_string()),
            text: None,
            blob: Some("blobdata".to_string()),
        },
    };
    let json = serde_json::to_string(&original).unwrap();
    let back: Content = serde_json::from_str(&json).unwrap();
    match back {
        Content::Resource { resource } => {
            assert_eq!(resource.uri, "s3://bucket/key");
            assert_eq!(resource.blob.as_deref(), Some("blobdata"));
            assert!(resource.text.is_none());
        }
        _ => panic!("expected Resource"),
    }
}

// ---------------------------------------------------------------------------
// ToolCallResponse with is_error flag
// ---------------------------------------------------------------------------

#[test]
fn tool_call_response_with_error_flag() {
    let raw = r#"{
        "content": [{"type":"text","text":"something went wrong"}],
        "isError": true
    }"#;
    let resp: ToolCallResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.is_error, Some(true));
    assert_eq!(resp.content.len(), 1);
}

#[test]
fn tool_call_response_is_error_skipped_when_none() {
    let resp = ToolCallResponse {
        content: vec![Content::Text { text: "ok".into() }],
        is_error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(
        !json.contains("isError"),
        "isError should be skipped when None, got: {}",
        json
    );
}

// ---------------------------------------------------------------------------
// InitializeRequest full round-trip
// ---------------------------------------------------------------------------

#[test]
fn initialize_request_full_roundtrip() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: Some(json!({"feature_x": true})),
            sampling: Some(json!({})),
        },
        client_info: ClientInfo {
            name: "TestClient".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let json_str = serde_json::to_string(&req).unwrap();
    let back: InitializeRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.protocol_version, "2024-11-05");
    assert_eq!(back.client_info.name, "TestClient");
    assert!(back.capabilities.experimental.is_some());
    assert!(back.capabilities.sampling.is_some());
}

#[test]
fn initialize_request_skip_none_capabilities() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "C".to_string(),
            version: "1".to_string(),
        },
    };
    let json_str = serde_json::to_string(&req).unwrap();
    assert!(
        !json_str.contains("experimental"),
        "experimental should be skipped: {}",
        json_str
    );
    assert!(
        !json_str.contains("sampling"),
        "sampling should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// InitializeResponse full round-trip with all capability fields
// ---------------------------------------------------------------------------

#[test]
fn initialize_response_full_capabilities_roundtrip() {
    let resp = InitializeResponse {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            experimental: Some(json!({"beta": true})),
            logging: Some(json!({})),
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(false),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        },
        server_info: ServerInfo {
            name: "full-server".to_string(),
            version: "2.0.0".to_string(),
        },
    };

    let json_str = serde_json::to_string(&resp).unwrap();
    let back: InitializeResponse = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.protocol_version, "2024-11-05");
    assert_eq!(back.server_info.name, "full-server");

    let caps = back.capabilities;
    assert!(caps.experimental.is_some());
    assert!(caps.logging.is_some());

    let prompts = caps.prompts.unwrap();
    assert_eq!(prompts.list_changed, Some(true));

    let resources = caps.resources.unwrap();
    assert_eq!(resources.subscribe, Some(true));
    assert_eq!(resources.list_changed, Some(false));

    let tools = caps.tools.unwrap();
    assert_eq!(tools.list_changed, Some(true));
}

#[test]
fn server_capabilities_skip_none_fields() {
    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: None,
        resources: None,
        tools: None,
    };
    let json_str = serde_json::to_string(&caps).unwrap();
    // All fields are None and marked skip_serializing_if, so JSON should be "{}"
    assert_eq!(
        json_str, "{}",
        "All-None capabilities should serialize to empty object"
    );
}

// ---------------------------------------------------------------------------
// ResourceContent with blob field
// ---------------------------------------------------------------------------

#[test]
fn resource_content_blob_roundtrip() {
    let rc = ResourceContent {
        uri: "custom://id/123".to_string(),
        mime_type: None,
        text: None,
        blob: Some("binarydata".to_string()),
    };
    let json_str = serde_json::to_string(&rc).unwrap();
    assert!(json_str.contains("blob"));
    assert!(
        !json_str.contains("mimeType"),
        "mimeType should be skipped when None"
    );
    assert!(
        !json_str.contains("\"text\""),
        "text should be skipped when None"
    );

    let back: ResourceContent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.blob.as_deref(), Some("binarydata"));
    assert!(back.mime_type.is_none());
    assert!(back.text.is_none());
}

// ---------------------------------------------------------------------------
// JsonRpcError with data field
// ---------------------------------------------------------------------------

#[test]
fn json_rpc_error_with_data() {
    let err = JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(json!({"details": "missing field 'name'"})),
    };
    let json_str = serde_json::to_string(&err).unwrap();
    assert!(json_str.contains("details"));

    let back: JsonRpcError = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.code, -32602);
    assert!(back.data.is_some());
}

#[test]
fn json_rpc_error_data_skipped_when_none() {
    let err = JsonRpcError {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    };
    let json_str = serde_json::to_string(&err).unwrap();
    assert!(
        !json_str.contains("data"),
        "data should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// JsonRpcResponse: both result and error present (unusual but valid JSON)
// ---------------------------------------------------------------------------

#[test]
fn json_rpc_response_skip_serializing_if_fields() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: None,
        error: None,
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    assert!(
        !json_str.contains("result"),
        "result should be skipped: {}",
        json_str
    );
    assert!(
        !json_str.contains("error"),
        "error should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// JsonRpcRequest: params skip
// ---------------------------------------------------------------------------

#[test]
fn json_rpc_request_params_skipped_when_none() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(99),
        method: "notifications/initialized".to_string(),
        params: None,
    };
    let json_str = serde_json::to_string(&req).unwrap();
    assert!(
        !json_str.contains("\"params\""),
        "params key should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// Deserialization from external JSON (simulating real server payloads)
// ---------------------------------------------------------------------------

#[test]
fn deserialize_tools_list_response_from_json() {
    let raw = r#"{
        "tools": [
            {
                "name": "execute_sql",
                "description": "Run an SQL query",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }
        ],
        "nextCursor": "abc123"
    }"#;
    let resp: ToolsListResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.tools.len(), 1);
    assert_eq!(resp.tools[0].name, "execute_sql");
    assert_eq!(resp.next_cursor.as_deref(), Some("abc123"));
    // Verify inputSchema is preserved
    let schema = &resp.tools[0].input_schema;
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["query"].is_object());
}

#[test]
fn deserialize_resources_list_response_from_json() {
    let raw = r#"{
        "resources": [
            { "uri": "db://main/users", "name": "users table" },
            { "uri": "db://main/orders", "name": "orders table", "description": "Order records", "mimeType": "application/json" }
        ]
    }"#;
    let resp: ResourcesListResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.resources.len(), 2);
    assert!(resp.resources[0].description.is_none());
    assert!(resp.resources[0].mime_type.is_none());
    assert_eq!(
        resp.resources[1].description.as_deref(),
        Some("Order records")
    );
    assert_eq!(
        resp.resources[1].mime_type.as_deref(),
        Some("application/json")
    );
    assert!(resp.next_cursor.is_none());
}

// ---------------------------------------------------------------------------
// ToolCallRequest serialization
// ---------------------------------------------------------------------------

#[test]
fn tool_call_request_arguments_skipped_when_none() {
    let req = ToolCallRequest {
        name: "ping".to_string(),
        arguments: None,
    };
    let json_str = serde_json::to_string(&req).unwrap();
    assert!(
        !json_str.contains("arguments"),
        "arguments should be skipped: {}",
        json_str
    );
}

#[test]
fn tool_call_request_with_arguments_roundtrip() {
    let req = ToolCallRequest {
        name: "search".to_string(),
        arguments: Some(json!({"query": "hello", "limit": 10})),
    };
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ToolCallRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.name, "search");
    let args = back.arguments.unwrap();
    assert_eq!(args["query"], "hello");
    assert_eq!(args["limit"], 10);
}

// ---------------------------------------------------------------------------
// ResourcesListRequest / ResourceReadRequest
// ---------------------------------------------------------------------------

#[test]
fn resources_list_request_cursor_skipped_when_none() {
    let req = ResourcesListRequest { cursor: None };
    let json_str = serde_json::to_string(&req).unwrap();
    assert!(
        !json_str.contains("cursor"),
        "cursor should be skipped: {}",
        json_str
    );
}

#[test]
fn resource_read_request_roundtrip() {
    let req = ResourceReadRequest {
        uri: "file:///etc/config.yaml".to_string(),
    };
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ResourceReadRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.uri, "file:///etc/config.yaml");
}

// ---------------------------------------------------------------------------
// ResourceReadResponse with multiple contents
// ---------------------------------------------------------------------------

#[test]
fn resource_read_response_multiple_contents() {
    let raw = r#"{
        "contents": [
            { "uri": "file:///a.txt", "text": "aaa" },
            { "uri": "file:///b.txt", "blob": "YmJi", "mimeType": "application/octet-stream" }
        ]
    }"#;
    let resp: ResourceReadResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.contents.len(), 2);
    assert_eq!(resp.contents[0].text.as_deref(), Some("aaa"));
    assert!(resp.contents[0].blob.is_none());
    assert_eq!(resp.contents[1].blob.as_deref(), Some("YmJi"));
    assert!(resp.contents[1].text.is_none());
}

// ---------------------------------------------------------------------------
// PromptsCapability and ResourcesCapability skip behavior
// ---------------------------------------------------------------------------

#[test]
fn prompts_capability_list_changed_skip() {
    let cap = PromptsCapability { list_changed: None };
    let json_str = serde_json::to_string(&cap).unwrap();
    assert!(
        !json_str.contains("listChanged"),
        "listChanged should be skipped: {}",
        json_str
    );
}

#[test]
fn resources_capability_skip_none_fields() {
    let cap = ResourcesCapability {
        subscribe: None,
        list_changed: None,
    };
    let json_str = serde_json::to_string(&cap).unwrap();
    assert_eq!(json_str, "{}");
}

#[test]
fn resources_capability_full_roundtrip() {
    let cap = ResourcesCapability {
        subscribe: Some(true),
        list_changed: Some(true),
    };
    let json_str = serde_json::to_string(&cap).unwrap();
    let back: ResourcesCapability = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.subscribe, Some(true));
    assert_eq!(back.list_changed, Some(true));
}

// ---------------------------------------------------------------------------
// ToolsListRequest cursor skip
// ---------------------------------------------------------------------------

#[test]
fn tools_list_request_cursor_skip() {
    let req = ToolsListRequest { cursor: None };
    let json_str = serde_json::to_string(&req).unwrap();
    assert!(
        !json_str.contains("cursor"),
        "cursor should be skipped: {}",
        json_str
    );
}

#[test]
fn tools_list_request_with_cursor_roundtrip() {
    let req = ToolsListRequest {
        cursor: Some("next-page".to_string()),
    };
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ToolsListRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.cursor.as_deref(), Some("next-page"));
}

// ---------------------------------------------------------------------------
// Tool without description
// ---------------------------------------------------------------------------

#[test]
fn tool_description_skipped_when_none() {
    let tool = Tool {
        name: "bare_tool".to_string(),
        description: None,
        input_schema: json!({}),
    };
    let json_str = serde_json::to_string(&tool).unwrap();
    assert!(
        !json_str.contains("description"),
        "description should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// Resource minimal fields (no optional fields)
// ---------------------------------------------------------------------------

#[test]
fn resource_skip_optional_fields() {
    let r = Resource {
        uri: "mem://x".to_string(),
        name: "x".to_string(),
        description: None,
        mime_type: None,
    };
    let json_str = serde_json::to_string(&r).unwrap();
    assert!(!json_str.contains("description"));
    assert!(!json_str.contains("mimeType"));
}

// ---------------------------------------------------------------------------
// ToolsListResponse: nextCursor skip
// ---------------------------------------------------------------------------

#[test]
fn tools_list_response_next_cursor_skip() {
    let resp = ToolsListResponse {
        tools: vec![],
        next_cursor: None,
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    assert!(
        !json_str.contains("nextCursor"),
        "nextCursor should be skipped: {}",
        json_str
    );
}

// ---------------------------------------------------------------------------
// ResourcesListResponse: nextCursor skip
// ---------------------------------------------------------------------------

#[test]
fn resources_list_response_next_cursor_skip() {
    let resp = ResourcesListResponse {
        resources: vec![],
        next_cursor: None,
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    assert!(!json_str.contains("nextCursor"));
}

// ---------------------------------------------------------------------------
// RequestId used as HashMap key
// ---------------------------------------------------------------------------

#[test]
fn request_id_hash_map_key() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(RequestId::new_number(1), "first");
    map.insert(RequestId::new_string("abc"), "second");
    map.insert(RequestId::new_number(1), "replaced");

    assert_eq!(map.len(), 2);
    assert_eq!(map[&RequestId::new_number(1)], "replaced");
    assert_eq!(map[&RequestId::new_string("abc")], "second");
}
