//! Public API tests for hudhudscript-mcp protocol types

use hudhudscript_mcp::{
    error_codes, methods, ClientCapabilities, ClientInfo, Content, InitializeRequest,
    InitializeResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse, PromptsCapability,
    RequestId, Resource, ResourceContent, ResourceReadRequest, ResourceReadResponse,
    ResourcesCapability, ResourcesListRequest, ResourcesListResponse, ServerCapabilities,
    ServerInfo, Tool, ToolCallRequest, ToolCallResponse, ToolsCapability, ToolsListRequest,
    ToolsListResponse, TransportConfig, TransportType,
};
use serde_json::json;

// ===============================================================================
// RequestId
// ===============================================================================

#[test]
fn request_id_string() {
    let id = RequestId::new_string("req-1");
    assert_eq!(id, RequestId::String("req-1".to_string()));
}

#[test]
fn request_id_number() {
    let id = RequestId::new_number(42);
    assert_eq!(id, RequestId::Number(42));
}

#[test]
fn request_id_string_eq() {
    let a = RequestId::new_string("x");
    let b = RequestId::new_string("x");
    assert_eq!(a, b);
}

#[test]
fn request_id_number_eq() {
    let a = RequestId::new_number(1);
    let b = RequestId::new_number(1);
    assert_eq!(a, b);
}

#[test]
fn request_id_ne_different_types() {
    let a = RequestId::new_string("1");
    let b = RequestId::new_number(1);
    assert_ne!(a, b);
}

#[test]
fn request_id_serde_string() {
    let id = RequestId::new_string("abc");
    let json = serde_json::to_string(&id).unwrap();
    let back: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[test]
fn request_id_serde_number() {
    let id = RequestId::new_number(99);
    let json = serde_json::to_string(&id).unwrap();
    let back: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[test]
fn request_id_clone() {
    let id = RequestId::new_string("clone-test");
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

#[test]
fn request_id_debug() {
    let id = RequestId::new_number(7);
    let dbg = format!("{:?}", id);
    assert!(dbg.contains("7"));
}

// ===============================================================================
// JsonRpcRequest
// ===============================================================================

#[test]
fn json_rpc_request_fields() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        method: "tools/list".to_string(),
        params: None,
    };
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "tools/list");
    assert!(req.params.is_none());
}

#[test]
fn json_rpc_request_with_params() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_string("r1"),
        method: "tools/call".to_string(),
        params: Some(json!({"name": "search", "arguments": {"query": "test"}})),
    };
    assert!(req.params.is_some());
}

#[test]
fn json_rpc_request_serde_roundtrip() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(42),
        method: "initialize".to_string(),
        params: Some(json!({"key": "value"})),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.method, "initialize");
    assert_eq!(back.id, RequestId::new_number(42));
}

#[test]
fn json_rpc_request_params_none_skipped() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        method: "test".to_string(),
        params: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("params"));
}

// ===============================================================================
// JsonRpcResponse
// ===============================================================================

#[test]
fn json_rpc_response_success() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: Some(json!({"status": "ok"})),
        error: None,
    };
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn json_rpc_response_error() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
}

#[test]
fn json_rpc_response_serde_roundtrip() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_string("r1"),
        result: Some(json!(42)),
        error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.result, Some(json!(42)));
}

// ===============================================================================
// JsonRpcError
// ===============================================================================

#[test]
fn json_rpc_error_fields() {
    let err = JsonRpcError {
        code: -32700,
        message: "Parse error".to_string(),
        data: Some(json!({"detail": "unexpected token"})),
    };
    assert_eq!(err.code, -32700);
    assert_eq!(err.message, "Parse error");
    assert!(err.data.is_some());
}

#[test]
fn json_rpc_error_without_data() {
    let err = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };
    assert!(err.data.is_none());
}

#[test]
fn json_rpc_error_serde_roundtrip() {
    let err = JsonRpcError {
        code: -32603,
        message: "Internal error".to_string(),
        data: None,
    };
    let json = serde_json::to_string(&err).unwrap();
    let back: JsonRpcError = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code, -32603);
}

// ===============================================================================
// InitializeRequest & Response
// ===============================================================================

#[test]
fn initialize_request_fields() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "hudhudscript".to_string(),
            version: "0.4.0".to_string(),
        },
    };
    assert_eq!(req.protocol_version, "2024-11-05");
    assert_eq!(req.client_info.name, "hudhudscript");
}

#[test]
fn initialize_response_fields() {
    let resp = InitializeResponse {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        },
        server_info: ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    assert_eq!(resp.server_info.name, "test-server");
    assert!(resp.capabilities.tools.is_some());
}

// ===============================================================================
// ClientCapabilities & ServerCapabilities
// ===============================================================================

#[test]
fn client_capabilities_empty() {
    let caps = ClientCapabilities {
        experimental: None,
        sampling: None,
    };
    assert!(caps.experimental.is_none());
    assert!(caps.sampling.is_none());
}

#[test]
fn server_capabilities_all_none() {
    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: None,
        resources: None,
        tools: None,
    };
    assert!(caps.tools.is_none());
    assert!(caps.resources.is_none());
}

#[test]
fn server_capabilities_with_all() {
    let caps = ServerCapabilities {
        experimental: Some(json!({})),
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
    };
    assert!(caps.prompts.is_some());
    assert!(caps.resources.as_ref().unwrap().subscribe == Some(true));
}

// ===============================================================================
// Tool, ToolsListRequest, ToolsListResponse
// ===============================================================================

#[test]
fn tool_fields() {
    let tool = Tool {
        name: "get_weather".to_string(),
        description: Some("Get weather info".to_string()),
        input_schema: json!({"type": "object"}),
    };
    assert_eq!(tool.name, "get_weather");
    assert!(tool.description.is_some());
}

#[test]
fn tool_no_description() {
    let tool = Tool {
        name: "ping".to_string(),
        description: None,
        input_schema: json!({"type": "object"}),
    };
    assert!(tool.description.is_none());
}

#[test]
fn tools_list_request_no_cursor() {
    let req = ToolsListRequest { cursor: None };
    assert!(req.cursor.is_none());
}

#[test]
fn tools_list_request_with_cursor() {
    let req = ToolsListRequest {
        cursor: Some("page2".to_string()),
    };
    assert_eq!(req.cursor.as_deref(), Some("page2"));
}

#[test]
fn tools_list_response_fields() {
    let resp = ToolsListResponse {
        tools: vec![
            Tool {
                name: "t1".to_string(),
                description: None,
                input_schema: json!({}),
            },
            Tool {
                name: "t2".to_string(),
                description: None,
                input_schema: json!({}),
            },
        ],
        next_cursor: None,
    };
    assert_eq!(resp.tools.len(), 2);
    assert!(resp.next_cursor.is_none());
}

#[test]
fn tools_list_response_with_cursor() {
    let resp = ToolsListResponse {
        tools: vec![],
        next_cursor: Some("next".to_string()),
    };
    assert_eq!(resp.next_cursor.as_deref(), Some("next"));
}

// ===============================================================================
// ToolCallRequest & ToolCallResponse
// ===============================================================================

#[test]
fn tool_call_request_fields() {
    let req = ToolCallRequest {
        name: "search".to_string(),
        arguments: Some(json!({"query": "rust"})),
    };
    assert_eq!(req.name, "search");
    assert!(req.arguments.is_some());
}

#[test]
fn tool_call_request_no_arguments() {
    let req = ToolCallRequest {
        name: "ping".to_string(),
        arguments: None,
    };
    assert!(req.arguments.is_none());
}

#[test]
fn tool_call_response_success() {
    let resp = ToolCallResponse {
        content: vec![Content::Text {
            text: "Hello".to_string(),
        }],
        is_error: None,
    };
    assert_eq!(resp.content.len(), 1);
    assert!(resp.is_error.is_none());
}

#[test]
fn tool_call_response_error() {
    let resp = ToolCallResponse {
        content: vec![Content::Text {
            text: "Error occurred".to_string(),
        }],
        is_error: Some(true),
    };
    assert_eq!(resp.is_error, Some(true));
}

// ===============================================================================
// Content
// ===============================================================================

#[test]
fn content_text() {
    let c = Content::Text {
        text: "hello".to_string(),
    };
    if let Content::Text { text } = &c {
        assert_eq!(text, "hello");
    } else {
        panic!("Expected Text content");
    }
}

#[test]
fn content_image() {
    let c = Content::Image {
        data: "base64data".to_string(),
        mime_type: "image/png".to_string(),
    };
    if let Content::Image { data, mime_type } = &c {
        assert_eq!(data, "base64data");
        assert_eq!(mime_type, "image/png");
    } else {
        panic!("Expected Image content");
    }
}

#[test]
fn content_resource() {
    let c = Content::Resource {
        resource: ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("content".to_string()),
            blob: None,
        },
    };
    if let Content::Resource { resource } = &c {
        assert_eq!(resource.uri, "file:///test.txt");
    } else {
        panic!("Expected Resource content");
    }
}

#[test]
fn content_serde_text_roundtrip() {
    let c = Content::Text {
        text: "test".to_string(),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: Content = serde_json::from_str(&json).unwrap();
    if let Content::Text { text } = back {
        assert_eq!(text, "test");
    } else {
        panic!("Expected Text");
    }
}

// ===============================================================================
// Resource & ResourceContent
// ===============================================================================

#[test]
fn resource_fields() {
    let r = Resource {
        uri: "file:///data.json".to_string(),
        name: "data".to_string(),
        description: Some("Data file".to_string()),
        mime_type: Some("application/json".to_string()),
    };
    assert_eq!(r.uri, "file:///data.json");
    assert_eq!(r.name, "data");
}

#[test]
fn resource_minimal() {
    let r = Resource {
        uri: "file:///x".to_string(),
        name: "x".to_string(),
        description: None,
        mime_type: None,
    };
    assert!(r.description.is_none());
    assert!(r.mime_type.is_none());
}

#[test]
fn resource_content_with_text() {
    let rc = ResourceContent {
        uri: "file:///a.txt".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: Some("Hello world".to_string()),
        blob: None,
    };
    assert_eq!(rc.text.as_deref(), Some("Hello world"));
    assert!(rc.blob.is_none());
}

#[test]
fn resource_content_with_blob() {
    let rc = ResourceContent {
        uri: "file:///img.png".to_string(),
        mime_type: Some("image/png".to_string()),
        text: None,
        blob: Some("base64encodeddata".to_string()),
    };
    assert!(rc.text.is_none());
    assert!(rc.blob.is_some());
}

// ===============================================================================
// ResourcesListRequest & Response, ResourceReadRequest & Response
// ===============================================================================

#[test]
fn resources_list_request_no_cursor() {
    let req = ResourcesListRequest { cursor: None };
    assert!(req.cursor.is_none());
}

#[test]
fn resources_list_response_fields() {
    let resp = ResourcesListResponse {
        resources: vec![Resource {
            uri: "file:///a".to_string(),
            name: "a".to_string(),
            description: None,
            mime_type: None,
        }],
        next_cursor: None,
    };
    assert_eq!(resp.resources.len(), 1);
}

#[test]
fn resource_read_request() {
    let req = ResourceReadRequest {
        uri: "file:///test".to_string(),
    };
    assert_eq!(req.uri, "file:///test");
}

#[test]
fn resource_read_response() {
    let resp = ResourceReadResponse {
        contents: vec![ResourceContent {
            uri: "file:///test".to_string(),
            mime_type: None,
            text: Some("content".to_string()),
            blob: None,
        }],
    };
    assert_eq!(resp.contents.len(), 1);
}

// ===============================================================================
// methods constants
// ===============================================================================

#[test]
fn method_initialize() {
    assert_eq!(methods::INITIALIZE, "initialize");
}

#[test]
fn method_tools_list() {
    assert_eq!(methods::TOOLS_LIST, "tools/list");
}

#[test]
fn method_tools_call() {
    assert_eq!(methods::TOOLS_CALL, "tools/call");
}

#[test]
fn method_resources_list() {
    assert_eq!(methods::RESOURCES_LIST, "resources/list");
}

#[test]
fn method_resources_read() {
    assert_eq!(methods::RESOURCES_READ, "resources/read");
}

#[test]
fn method_prompts_list() {
    assert_eq!(methods::PROMPTS_LIST, "prompts/list");
}

#[test]
fn method_prompts_get() {
    assert_eq!(methods::PROMPTS_GET, "prompts/get");
}

// ===============================================================================
// error_codes constants
// ===============================================================================

#[test]
fn error_code_parse_error() {
    assert_eq!(error_codes::PARSE_ERROR, -32700);
}

#[test]
fn error_code_invalid_request() {
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
}

#[test]
fn error_code_method_not_found() {
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
}

#[test]
fn error_code_invalid_params() {
    assert_eq!(error_codes::INVALID_PARAMS, -32602);
}

#[test]
fn error_code_internal_error() {
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
}

// ===============================================================================
// TransportType & TransportConfig
// ===============================================================================

#[test]
fn transport_type_eq() {
    assert_eq!(TransportType::Stdio, TransportType::Stdio);
    assert_eq!(TransportType::Sse, TransportType::Sse);
    assert_ne!(TransportType::Stdio, TransportType::Sse);
}

#[test]
fn transport_config_stdio() {
    let config = TransportConfig::stdio("uvx", vec!["mcp-server".to_string()]);
    assert_eq!(config.transport_type, TransportType::Stdio);
    assert_eq!(config.command.as_deref(), Some("uvx"));
    assert_eq!(config.args.len(), 1);
    assert!(config.url.is_none());
}

#[test]
fn transport_config_sse() {
    let config = TransportConfig::sse("http://localhost:3000/sse");
    assert_eq!(config.transport_type, TransportType::Sse);
    assert_eq!(config.url.as_deref(), Some("http://localhost:3000/sse"));
    assert!(config.command.is_none());
}

#[test]
fn transport_config_stdio_no_args() {
    let config = TransportConfig::stdio("echo", vec![]);
    assert!(config.args.is_empty());
}

#[test]
fn transport_config_clone() {
    let config = TransportConfig::stdio("cmd", vec!["arg".to_string()]);
    let cloned = config.clone();
    assert_eq!(cloned.transport_type, TransportType::Stdio);
    assert_eq!(cloned.command.as_deref(), Some("cmd"));
}

// ===============================================================================
// Inline tests extracted from protocol.rs
// ===============================================================================

#[test]
fn test_request_id_serialization() {
    let id_str = RequestId::new_string("test-123");
    let json = serde_json::to_string(&id_str).unwrap();
    assert_eq!(json, r#""test-123""#);

    let id_num = RequestId::new_number(42);
    let json = serde_json::to_string(&id_num).unwrap();
    assert_eq!(json, "42");
}

#[test]
fn test_initialize_request() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "HudHudScript".to_string(),
            version: "0.1.0".to_string(),
        },
    };

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("protocolVersion"));
    assert!(json.contains("HudHudScript"));
}

#[test]
fn test_tool_definition() {
    let tool = Tool {
        name: "get_weather".to_string(),
        description: Some("Get weather information".to_string()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "location": { "type": "string" }
            },
            "required": ["location"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("get_weather"));
    assert!(json.contains("inputSchema"));
}

#[test]
fn test_request_id_deserialize_string() {
    let id: RequestId = serde_json::from_str(r#""abc-123""#).unwrap();
    assert_eq!(id, RequestId::String("abc-123".to_string()));
}

#[test]
fn test_request_id_deserialize_number() {
    let id: RequestId = serde_json::from_str("99").unwrap();
    assert_eq!(id, RequestId::Number(99));
}

#[test]
fn test_request_id_equality() {
    let a = RequestId::new_number(1);
    let b = RequestId::new_number(1);
    let c = RequestId::new_number(2);
    assert_eq!(a, b);
    assert_ne!(a, c);

    let x = RequestId::new_string("test");
    let y = RequestId::new_string("test");
    assert_eq!(x, y);
}

#[test]
fn test_jsonrpc_request_serialization() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        method: "test/method".to_string(),
        params: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains(r#""jsonrpc":"2.0""#));
    assert!(json.contains(r#""method":"test/method""#));
    assert!(!json.contains("params"));
}

#[test]
fn test_jsonrpc_request_with_params() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        method: "test".to_string(),
        params: Some(serde_json::json!({"key": "value"})),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("params"));
    assert!(json.contains("key"));
}

#[test]
fn test_jsonrpc_response_success() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: Some(serde_json::json!({"data": 42})),
        error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("result"));
    assert!(!json.contains("error"));
}

#[test]
fn test_jsonrpc_response_error() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: None,
        error: Some(JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Method not found".to_string(),
            data: None,
        }),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("error"));
    assert!(json.contains("-32601"));
    assert!(!json.contains("result"));
}

#[test]
fn test_jsonrpc_response_roundtrip() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_string("req-1"),
        result: Some(serde_json::json!({"ok": true})),
        error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, RequestId::new_string("req-1"));
    assert!(deserialized.result.is_some());
    assert!(deserialized.error.is_none());
}

#[test]
fn test_jsonrpc_error_with_data() {
    let err = JsonRpcError {
        code: error_codes::INVALID_PARAMS,
        message: "Invalid params".to_string(),
        data: Some(serde_json::json!({"detail": "missing field"})),
    };
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("-32602"));
    assert!(json.contains("missing field"));
}

#[test]
fn test_jsonrpc_error_without_data() {
    let err = JsonRpcError {
        code: error_codes::PARSE_ERROR,
        message: "Parse error".to_string(),
        data: None,
    };
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("-32700"));
    assert!(!json.contains("data"));
}

#[test]
fn test_content_text_serialization() {
    let content = Content::Text {
        text: "hello world".to_string(),
    };
    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains(r#""type":"text""#));
    assert!(json.contains("hello world"));
}

#[test]
fn test_content_image_serialization() {
    let content = Content::Image {
        data: "base64data".to_string(),
        mime_type: "image/png".to_string(),
    };
    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains(r#""type":"image""#));
    assert!(json.contains("base64data"));
    assert!(json.contains("image/png"));
}

#[test]
fn test_content_resource_serialization() {
    let content = Content::Resource {
        resource: ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("file content".to_string()),
            blob: None,
        },
    };
    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains(r#""type":"resource""#));
    assert!(json.contains("file:///test.txt"));
}

#[test]
fn test_tool_call_request_without_args() {
    let req = ToolCallRequest {
        name: "my_tool".to_string(),
        arguments: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("my_tool"));
    assert!(!json.contains("arguments"));
}

#[test]
fn test_tool_call_request_with_args() {
    let req = ToolCallRequest {
        name: "calc".to_string(),
        arguments: Some(serde_json::json!({"x": 5})),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("calc"));
    assert!(json.contains("arguments"));
}

#[test]
fn test_tool_call_response() {
    let resp = ToolCallResponse {
        content: vec![Content::Text {
            text: "result".to_string(),
        }],
        is_error: Some(false),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("result"));
    assert!(json.contains("isError"));
}

#[test]
fn test_tool_call_response_no_error_field() {
    let resp = ToolCallResponse {
        content: vec![],
        is_error: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(!json.contains("isError"));
}

#[test]
fn test_tools_list_request_no_cursor() {
    let req = ToolsListRequest { cursor: None };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("cursor"));
}

#[test]
fn test_tools_list_request_with_cursor() {
    let req = ToolsListRequest {
        cursor: Some("page2".to_string()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("page2"));
}

#[test]
fn test_tools_list_response_roundtrip() {
    let resp = ToolsListResponse {
        tools: vec![Tool {
            name: "tool1".to_string(),
            description: None,
            input_schema: serde_json::json!({}),
        }],
        next_cursor: Some("next".to_string()),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: ToolsListResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tools.len(), 1);
    assert_eq!(deserialized.tools[0].name, "tool1");
    assert_eq!(deserialized.next_cursor.unwrap(), "next");
}

#[test]
fn test_resources_list_request() {
    let req = ResourcesListRequest { cursor: None };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("cursor"));
}

#[test]
fn test_resources_list_response_roundtrip() {
    let resp = ResourcesListResponse {
        resources: vec![Resource {
            uri: "file:///data.txt".to_string(),
            name: "data".to_string(),
            description: Some("A data file".to_string()),
            mime_type: Some("text/plain".to_string()),
        }],
        next_cursor: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: ResourcesListResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.resources.len(), 1);
    assert_eq!(deserialized.resources[0].uri, "file:///data.txt");
    assert_eq!(deserialized.resources[0].name, "data");
    assert!(deserialized.next_cursor.is_none());
}

#[test]
fn test_resource_read_request() {
    let req = ResourceReadRequest {
        uri: "file:///test.txt".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("file:///test.txt"));
}

#[test]
fn test_resource_read_response() {
    let resp = ResourceReadResponse {
        contents: vec![ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: None,
            text: Some("hello".to_string()),
            blob: None,
        }],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: ResourceReadResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.contents.len(), 1);
    assert_eq!(deserialized.contents[0].text.as_deref().unwrap(), "hello");
}

#[test]
fn test_server_capabilities_full() {
    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
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
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(json.contains("prompts"));
    assert!(json.contains("resources"));
    assert!(json.contains("tools"));
    assert!(json.contains("listChanged"));
}

#[test]
fn test_server_capabilities_empty() {
    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: None,
        resources: None,
        tools: None,
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(!json.contains("prompts"));
    assert!(!json.contains("resources"));
    assert!(!json.contains("tools"));
}

#[test]
fn test_initialize_response_roundtrip() {
    let resp = InitializeResponse {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(ToolsCapability { list_changed: None }),
        },
        server_info: ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: InitializeResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.protocol_version, "2024-11-05");
    assert_eq!(deserialized.server_info.name, "test-server");
    assert_eq!(deserialized.server_info.version, "1.0.0");
}

#[test]
fn test_method_constants() {
    assert_eq!(methods::INITIALIZE, "initialize");
    assert_eq!(methods::TOOLS_LIST, "tools/list");
    assert_eq!(methods::TOOLS_CALL, "tools/call");
    assert_eq!(methods::RESOURCES_LIST, "resources/list");
    assert_eq!(methods::RESOURCES_READ, "resources/read");
    assert_eq!(methods::PROMPTS_LIST, "prompts/list");
    assert_eq!(methods::PROMPTS_GET, "prompts/get");
}

#[test]
fn test_error_code_constants() {
    assert_eq!(error_codes::PARSE_ERROR, -32700);
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_codes::INVALID_PARAMS, -32602);
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
}

#[test]
fn test_client_capabilities_empty() {
    let caps = ClientCapabilities {
        experimental: None,
        sampling: None,
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(!json.contains("experimental"));
    assert!(!json.contains("sampling"));
}

#[test]
fn test_client_capabilities_with_values() {
    let caps = ClientCapabilities {
        experimental: Some(serde_json::json!({"feature": true})),
        sampling: Some(serde_json::json!({})),
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(json.contains("experimental"));
    assert!(json.contains("sampling"));
}

#[test]
fn test_content_text_roundtrip() {
    let content = Content::Text {
        text: "round trip".to_string(),
    };
    let json = serde_json::to_string(&content).unwrap();
    let deserialized: Content = serde_json::from_str(&json).unwrap();
    match deserialized {
        Content::Text { text } => assert_eq!(text, "round trip"),
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_content_image_roundtrip() {
    let content = Content::Image {
        data: "aGVsbG8=".to_string(),
        mime_type: "image/jpeg".to_string(),
    };
    let json = serde_json::to_string(&content).unwrap();
    let deserialized: Content = serde_json::from_str(&json).unwrap();
    match deserialized {
        Content::Image { data, mime_type } => {
            assert_eq!(data, "aGVsbG8=");
            assert_eq!(mime_type, "image/jpeg");
        }
        _ => panic!("Expected Image variant"),
    }
}

#[test]
fn test_content_resource_roundtrip() {
    let content = Content::Resource {
        resource: ResourceContent {
            uri: "file:///data".to_string(),
            mime_type: None,
            text: None,
            blob: Some("YmxvYg==".to_string()),
        },
    };
    let json = serde_json::to_string(&content).unwrap();
    let deserialized: Content = serde_json::from_str(&json).unwrap();
    match deserialized {
        Content::Resource { resource } => {
            assert_eq!(resource.uri, "file:///data");
            assert!(resource.mime_type.is_none());
            assert!(resource.text.is_none());
            assert_eq!(resource.blob.unwrap(), "YmxvYg==");
        }
        _ => panic!("Expected Resource variant"),
    }
}

#[test]
fn test_jsonrpc_request_roundtrip() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_string("abc"),
        method: "tools/list".to_string(),
        params: Some(serde_json::json!({"cursor": null})),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, RequestId::new_string("abc"));
    assert_eq!(deserialized.method, "tools/list");
    assert!(deserialized.params.is_some());
}

#[test]
fn test_jsonrpc_error_roundtrip() {
    let err = JsonRpcError {
        code: error_codes::INTERNAL_ERROR,
        message: "internal error occurred".to_string(),
        data: Some(serde_json::json!({"trace": "stack"})),
    };
    let json = serde_json::to_string(&err).unwrap();
    let deserialized: JsonRpcError = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.code, -32603);
    assert_eq!(deserialized.message, "internal error occurred");
    assert!(deserialized.data.is_some());
}

#[test]
fn test_tool_call_response_roundtrip() {
    let resp = ToolCallResponse {
        content: vec![
            Content::Text {
                text: "line 1".to_string(),
            },
            Content::Text {
                text: "line 2".to_string(),
            },
        ],
        is_error: Some(true),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: ToolCallResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.content.len(), 2);
    assert_eq!(deserialized.is_error, Some(true));
}

#[test]
fn test_resource_content_minimal() {
    let rc = ResourceContent {
        uri: "test://uri".to_string(),
        mime_type: None,
        text: None,
        blob: None,
    };
    let json = serde_json::to_string(&rc).unwrap();
    assert!(json.contains("test://uri"));
    assert!(!json.contains("mimeType"));
    assert!(!json.contains("text"));
    assert!(!json.contains("blob"));
}

#[test]
fn test_resource_roundtrip() {
    let res = Resource {
        uri: "file:///doc.pdf".to_string(),
        name: "doc".to_string(),
        description: None,
        mime_type: None,
    };
    let json = serde_json::to_string(&res).unwrap();
    let deserialized: Resource = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.uri, "file:///doc.pdf");
    assert_eq!(deserialized.name, "doc");
    assert!(deserialized.description.is_none());
    assert!(deserialized.mime_type.is_none());
}

#[test]
fn test_initialize_request_roundtrip() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: Some(serde_json::json!({"test": true})),
            sampling: None,
        },
        client_info: ClientInfo {
            name: "TestClient".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: InitializeRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.protocol_version, "2024-11-05");
    assert_eq!(deserialized.client_info.name, "TestClient");
    assert!(deserialized.capabilities.experimental.is_some());
    assert!(deserialized.capabilities.sampling.is_none());
}

#[test]
fn test_tool_no_description() {
    let tool = Tool {
        name: "simple".to_string(),
        description: None,
        input_schema: serde_json::json!({}),
    };
    let json = serde_json::to_string(&tool).unwrap();
    assert!(!json.contains("description"));
    let deserialized: Tool = serde_json::from_str(&json).unwrap();
    assert!(deserialized.description.is_none());
}

#[test]
fn test_request_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(RequestId::new_number(1));
    set.insert(RequestId::new_string("abc"));
    set.insert(RequestId::new_number(1)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_jsonrpc_response_deserialize_success_from_json() {
    let raw = r#"{"jsonrpc":"2.0","id":5,"result":{"ok":true}}"#;
    let resp: JsonRpcResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.id, RequestId::Number(5));
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn test_jsonrpc_response_deserialize_error_from_json() {
    let raw =
        r#"{"jsonrpc":"2.0","id":"req-99","error":{"code":-32600,"message":"Invalid Request"}}"#;
    let resp: JsonRpcResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.id, RequestId::String("req-99".to_string()));
    assert!(resp.result.is_none());
    let err = resp.error.unwrap();
    assert_eq!(err.code, error_codes::INVALID_REQUEST);
    assert_eq!(err.message, "Invalid Request");
    assert!(err.data.is_none());
}

#[test]
fn test_initialize_response_from_raw_json() {
    let raw = r#"{
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": { "listChanged": true }
        },
        "serverInfo": {
            "name": "test-mcp-server",
            "version": "0.1.0"
        }
    }"#;
    let resp: InitializeResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.protocol_version, "2024-11-05");
    assert_eq!(resp.server_info.name, "test-mcp-server");
    assert!(resp.capabilities.tools.is_some());
    assert_eq!(resp.capabilities.tools.unwrap().list_changed, Some(true));
}

#[test]
fn test_tools_list_response_from_raw_json() {
    let raw = r#"{
        "tools": [
            {
                "name": "read_file",
                "description": "Read a file",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "write_file",
                "inputSchema": { "type": "object" }
            }
        ]
    }"#;
    let resp: ToolsListResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.tools.len(), 2);
    assert_eq!(resp.tools[0].name, "read_file");
    assert_eq!(resp.tools[0].description.as_deref(), Some("Read a file"));
    assert_eq!(resp.tools[1].name, "write_file");
    assert!(resp.tools[1].description.is_none());
    assert!(resp.next_cursor.is_none());
}

#[test]
fn test_tool_call_response_from_raw_json() {
    let raw = r#"{
        "content": [
            { "type": "text", "text": "File contents here" }
        ],
        "isError": false
    }"#;
    let resp: ToolCallResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.content.len(), 1);
    match &resp.content[0] {
        Content::Text { text } => assert_eq!(text, "File contents here"),
        _ => panic!("Expected Text content"),
    }
    assert_eq!(resp.is_error, Some(false));
}

#[test]
fn test_resources_list_response_with_next_cursor() {
    let raw = r#"{
        "resources": [
            { "uri": "file:///a.txt", "name": "a" }
        ],
        "nextCursor": "page2token"
    }"#;
    let resp: ResourcesListResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(resp.resources.len(), 1);
    assert_eq!(resp.next_cursor.as_deref(), Some("page2token"));
}

#[test]
fn test_content_image_deserialize_from_raw() {
    let raw = r#"{"type":"image","data":"iVBOR...","mime_type":"image/png"}"#;
    let content: Content = serde_json::from_str(raw).unwrap();
    match content {
        Content::Image { data, mime_type } => {
            assert_eq!(data, "iVBOR...");
            assert_eq!(mime_type, "image/png");
        }
        _ => panic!("Expected Image variant"),
    }
}

#[test]
fn test_content_resource_deserialize_from_raw() {
    let raw = r#"{"type":"resource","resource":{"uri":"file:///x","text":"hello"}}"#;
    let content: Content = serde_json::from_str(raw).unwrap();
    match content {
        Content::Resource { resource } => {
            assert_eq!(resource.uri, "file:///x");
            assert_eq!(resource.text.as_deref(), Some("hello"));
            assert!(resource.mime_type.is_none());
            assert!(resource.blob.is_none());
        }
        _ => panic!("Expected Resource variant"),
    }
}

#[test]
fn test_request_id_string_vs_number_not_equal() {
    let str_id = RequestId::new_string("1");
    let num_id = RequestId::new_number(1);
    assert_ne!(str_id, num_id);
}

#[test]
fn test_request_id_negative_number() {
    let id = RequestId::new_number(-42);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "-42");
    let deserialized: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, RequestId::Number(-42));
}

#[test]
fn test_prompts_capability_roundtrip() {
    let cap = PromptsCapability {
        list_changed: Some(false),
    };
    let json = serde_json::to_string(&cap).unwrap();
    let deser: PromptsCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.list_changed, Some(false));
}

#[test]
fn test_prompts_capability_none_list_changed() {
    let cap = PromptsCapability { list_changed: None };
    let json = serde_json::to_string(&cap).unwrap();
    assert!(!json.contains("listChanged"));
}

#[test]
fn test_resources_capability_roundtrip() {
    let cap = ResourcesCapability {
        subscribe: Some(true),
        list_changed: Some(false),
    };
    let json = serde_json::to_string(&cap).unwrap();
    let deser: ResourcesCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.subscribe, Some(true));
    assert_eq!(deser.list_changed, Some(false));
}

#[test]
fn test_tools_capability_none() {
    let cap = ToolsCapability { list_changed: None };
    let json = serde_json::to_string(&cap).unwrap();
    assert!(!json.contains("listChanged"));
}

#[test]
fn test_server_info_roundtrip() {
    let info = ServerInfo {
        name: "my-server".to_string(),
        version: "2.0.0".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let deser: ServerInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "my-server");
    assert_eq!(deser.version, "2.0.0");
}

#[test]
fn test_client_info_roundtrip() {
    let info = ClientInfo {
        name: "client".to_string(),
        version: "1.0.0".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let deser: ClientInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "client");
    assert_eq!(deser.version, "1.0.0");
}

#[test]
fn test_request_id_debug_variants() {
    let num = RequestId::new_number(0);
    let dbg = format!("{:?}", num);
    assert!(dbg.contains("Number"));
    assert!(dbg.contains("0"));

    let s = RequestId::new_string("");
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("String"));
}

#[test]
fn test_request_id_zero() {
    let id = RequestId::new_number(0);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "0");
    let deser: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(deser, RequestId::Number(0));
}

#[test]
fn test_request_id_large_positive() {
    let id = RequestId::new_number(i64::MAX);
    let json = serde_json::to_string(&id).unwrap();
    let deser: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(deser, RequestId::Number(i64::MAX));
}

#[test]
fn test_request_id_empty_string() {
    let id = RequestId::new_string("");
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, r#""""#);
    let deser: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(deser, RequestId::String("".to_string()));
}

#[test]
fn test_jsonrpc_request_clone_and_debug() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(10),
        method: "test".to_string(),
        params: None,
    };
    let cloned = req.clone();
    assert_eq!(cloned.id, RequestId::new_number(10));
    assert_eq!(cloned.method, "test");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("JsonRpcRequest"));
}

#[test]
fn test_jsonrpc_response_clone_and_debug() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_string("x"),
        result: None,
        error: None,
    };
    let cloned = resp.clone();
    assert_eq!(cloned.id, RequestId::new_string("x"));
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("JsonRpcResponse"));
}

#[test]
fn test_jsonrpc_error_clone_and_debug() {
    let err = JsonRpcError {
        code: -1,
        message: "custom error".to_string(),
        data: None,
    };
    let cloned = err.clone();
    assert_eq!(cloned.code, -1);
    assert_eq!(cloned.message, "custom error");
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("JsonRpcError"));
}

#[test]
fn test_initialize_request_debug() {
    let req = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            experimental: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "Test".to_string(),
            version: "1.0".to_string(),
        },
    };
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("InitializeRequest"));
    assert!(dbg.contains("Test"));
}

#[test]
fn test_server_capabilities_clone_debug() {
    let caps = ServerCapabilities {
        experimental: Some(serde_json::json!({"x": 1})),
        logging: Some(serde_json::json!({})),
        prompts: None,
        resources: None,
        tools: None,
    };
    let cloned = caps.clone();
    assert!(cloned.experimental.is_some());
    assert!(cloned.logging.is_some());
    let dbg = format!("{:?}", caps);
    assert!(dbg.contains("ServerCapabilities"));
}

#[test]
fn test_tool_clone_and_debug() {
    let tool = Tool {
        name: "t1".to_string(),
        description: Some("desc".to_string()),
        input_schema: serde_json::json!({}),
    };
    let cloned = tool.clone();
    assert_eq!(cloned.name, "t1");
    assert_eq!(cloned.description.as_deref(), Some("desc"));
    let dbg = format!("{:?}", tool);
    assert!(dbg.contains("Tool"));
}

#[test]
fn test_tool_call_request_clone_and_debug() {
    let req = ToolCallRequest {
        name: "my_tool".to_string(),
        arguments: Some(serde_json::json!({"a": 1})),
    };
    let cloned = req.clone();
    assert_eq!(cloned.name, "my_tool");
    assert!(cloned.arguments.is_some());
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ToolCallRequest"));
}

#[test]
fn test_tool_call_response_clone_and_debug() {
    let resp = ToolCallResponse {
        content: vec![Content::Text {
            text: "ok".to_string(),
        }],
        is_error: None,
    };
    let cloned = resp.clone();
    assert_eq!(cloned.content.len(), 1);
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("ToolCallResponse"));
}

#[test]
fn test_resource_content_clone_debug() {
    let rc = ResourceContent {
        uri: "x://y".to_string(),
        mime_type: Some("text/html".to_string()),
        text: Some("content".to_string()),
        blob: Some("blob".to_string()),
    };
    let cloned = rc.clone();
    assert_eq!(cloned.uri, "x://y");
    assert_eq!(cloned.mime_type.as_deref(), Some("text/html"));
    assert_eq!(cloned.text.as_deref(), Some("content"));
    assert_eq!(cloned.blob.as_deref(), Some("blob"));
    let dbg = format!("{:?}", rc);
    assert!(dbg.contains("ResourceContent"));
}

#[test]
fn test_resource_clone_debug() {
    let res = Resource {
        uri: "file:///a".to_string(),
        name: "a".to_string(),
        description: Some("desc".to_string()),
        mime_type: Some("application/octet-stream".to_string()),
    };
    let cloned = res.clone();
    assert_eq!(cloned.description.as_deref(), Some("desc"));
    assert_eq!(
        cloned.mime_type.as_deref(),
        Some("application/octet-stream")
    );
    let dbg = format!("{:?}", res);
    assert!(dbg.contains("Resource"));
}

#[test]
fn test_tools_list_request_clone_debug() {
    let req = ToolsListRequest {
        cursor: Some("abc".to_string()),
    };
    let cloned = req.clone();
    assert_eq!(cloned.cursor.as_deref(), Some("abc"));
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ToolsListRequest"));
}

#[test]
fn test_tools_list_response_clone_debug() {
    let resp = ToolsListResponse {
        tools: vec![],
        next_cursor: None,
    };
    let cloned = resp.clone();
    assert!(cloned.tools.is_empty());
    assert!(cloned.next_cursor.is_none());
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("ToolsListResponse"));
}

#[test]
fn test_resources_list_request_clone_debug() {
    let req = ResourcesListRequest { cursor: None };
    let cloned = req.clone();
    assert!(cloned.cursor.is_none());
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ResourcesListRequest"));
}

#[test]
fn test_resources_list_response_clone_debug() {
    let resp = ResourcesListResponse {
        resources: vec![],
        next_cursor: Some("next".to_string()),
    };
    let cloned = resp.clone();
    assert_eq!(cloned.next_cursor.as_deref(), Some("next"));
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("ResourcesListResponse"));
}

#[test]
fn test_resource_read_request_clone_debug() {
    let req = ResourceReadRequest {
        uri: "test://x".to_string(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.uri, "test://x");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ResourceReadRequest"));
}

#[test]
fn test_resource_read_response_clone_debug() {
    let resp = ResourceReadResponse { contents: vec![] };
    let cloned = resp.clone();
    assert!(cloned.contents.is_empty());
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("ResourceReadResponse"));
}

#[test]
fn test_initialize_response_clone_debug() {
    let resp = InitializeResponse {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            tools: None,
        },
        server_info: ServerInfo {
            name: "s".to_string(),
            version: "v".to_string(),
        },
    };
    let cloned = resp.clone();
    assert_eq!(cloned.protocol_version, "2024-11-05");
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("InitializeResponse"));
}

#[test]
fn test_content_clone_debug_all_variants() {
    let text = Content::Text {
        text: "t".to_string(),
    };
    let text_c = text.clone();
    match text_c {
        Content::Text { text } => assert_eq!(text, "t"),
        _ => panic!("wrong variant"),
    }
    let dbg = format!("{:?}", text);
    assert!(dbg.contains("Text"));

    let img = Content::Image {
        data: "d".to_string(),
        mime_type: "m".to_string(),
    };
    let img_c = img.clone();
    match img_c {
        Content::Image { data, mime_type } => {
            assert_eq!(data, "d");
            assert_eq!(mime_type, "m");
        }
        _ => panic!("wrong variant"),
    }
    let dbg = format!("{:?}", img);
    assert!(dbg.contains("Image"));

    let res = Content::Resource {
        resource: ResourceContent {
            uri: "u".to_string(),
            mime_type: None,
            text: None,
            blob: None,
        },
    };
    let res_c = res.clone();
    match res_c {
        Content::Resource { resource } => assert_eq!(resource.uri, "u"),
        _ => panic!("wrong variant"),
    }
    let dbg = format!("{:?}", res);
    assert!(dbg.contains("Resource"));
}

#[test]
fn test_prompts_capability_clone_debug() {
    let cap = PromptsCapability {
        list_changed: Some(true),
    };
    let cloned = cap.clone();
    assert_eq!(cloned.list_changed, Some(true));
    let dbg = format!("{:?}", cap);
    assert!(dbg.contains("PromptsCapability"));
}

#[test]
fn test_resources_capability_clone_debug() {
    let cap = ResourcesCapability {
        subscribe: None,
        list_changed: None,
    };
    let cloned = cap.clone();
    assert!(cloned.subscribe.is_none());
    assert!(cloned.list_changed.is_none());
    let dbg = format!("{:?}", cap);
    assert!(dbg.contains("ResourcesCapability"));
}

#[test]
fn test_tools_capability_clone_debug() {
    let cap = ToolsCapability {
        list_changed: Some(false),
    };
    let cloned = cap.clone();
    assert_eq!(cloned.list_changed, Some(false));
    let dbg = format!("{:?}", cap);
    assert!(dbg.contains("ToolsCapability"));
}

#[test]
fn test_server_info_clone_debug() {
    let info = ServerInfo {
        name: "srv".to_string(),
        version: "0.1".to_string(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.name, "srv");
    let dbg = format!("{:?}", info);
    assert!(dbg.contains("ServerInfo"));
}

#[test]
fn test_client_info_clone_debug() {
    let info = ClientInfo {
        name: "cli".to_string(),
        version: "3.0".to_string(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.name, "cli");
    let dbg = format!("{:?}", info);
    assert!(dbg.contains("ClientInfo"));
}

#[test]
fn test_resources_capability_all_none_serialization() {
    let cap = ResourcesCapability {
        subscribe: None,
        list_changed: None,
    };
    let json = serde_json::to_string(&cap).unwrap();
    assert!(!json.contains("subscribe"));
    assert!(!json.contains("listChanged"));
}

#[test]
fn test_server_capabilities_logging_experimental() {
    let caps = ServerCapabilities {
        experimental: Some(serde_json::json!({"beta": true})),
        logging: Some(serde_json::json!({"level": "debug"})),
        prompts: None,
        resources: None,
        tools: None,
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(json.contains("experimental"));
    assert!(json.contains("logging"));
    let deser: ServerCapabilities = serde_json::from_str(&json).unwrap();
    assert!(deser.experimental.is_some());
    assert!(deser.logging.is_some());
}

#[test]
fn test_jsonrpc_response_with_both_result_and_error() {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: Some(serde_json::json!(null)),
        error: Some(JsonRpcError {
            code: -32600,
            message: "weird".to_string(),
            data: None,
        }),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("result"));
    assert!(json.contains("error"));
}

#[test]
fn test_tools_list_response_multiple_tools() {
    let resp = ToolsListResponse {
        tools: vec![
            Tool {
                name: "a".to_string(),
                description: None,
                input_schema: serde_json::json!({}),
            },
            Tool {
                name: "b".to_string(),
                description: Some("B tool".to_string()),
                input_schema: serde_json::json!({"type": "object"}),
            },
            Tool {
                name: "c".to_string(),
                description: None,
                input_schema: serde_json::json!({}),
            },
        ],
        next_cursor: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deser: ToolsListResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.tools.len(), 3);
    assert_eq!(deser.tools[1].description.as_deref(), Some("B tool"));
}

#[test]
fn test_resource_read_response_multiple_contents() {
    let resp = ResourceReadResponse {
        contents: vec![
            ResourceContent {
                uri: "file:///a".to_string(),
                mime_type: Some("text/plain".to_string()),
                text: Some("aaa".to_string()),
                blob: None,
            },
            ResourceContent {
                uri: "file:///b".to_string(),
                mime_type: None,
                text: None,
                blob: Some("YmJi".to_string()),
            },
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deser: ResourceReadResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.contents.len(), 2);
    assert_eq!(deser.contents[0].text.as_deref(), Some("aaa"));
    assert_eq!(deser.contents[1].blob.as_deref(), Some("YmJi"));
}

// ---- lib.rs re-export tests ----

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_reexport_connection_state() {
    use hudhudscript_mcp::ConnectionState;
    let state = ConnectionState::Disconnected;
    assert_eq!(state, ConnectionState::Disconnected);
}

#[test]
fn test_reexport_transport_config() {
    assert_eq!(
        TransportConfig::stdio("echo", vec![]).transport_type,
        TransportType::Stdio
    );
}

#[test]
fn test_reexport_request_id() {
    let id = RequestId::new_number(1);
    assert_eq!(id, RequestId::Number(1));
    let id_str = RequestId::new_string("abc");
    assert_eq!(id_str, RequestId::String("abc".to_string()));
}

#[test]
fn test_reexport_methods() {
    assert_eq!(methods::INITIALIZE, "initialize");
    assert_eq!(methods::TOOLS_LIST, "tools/list");
}

#[test]
fn test_reexport_error_codes() {
    assert_eq!(error_codes::PARSE_ERROR, -32700);
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
}
