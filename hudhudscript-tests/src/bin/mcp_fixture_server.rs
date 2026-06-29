//! MCP fixture server — real stdio JSON-RPC server for integration tests.
//!
//! Reads newline-delimited JSON-RPC from stdin, responds on stdout.
//! Supports: initialize, tools/list, tools/call (echo, add).
//!
//! Usage: #[test] uses env!("CARGO_BIN_EXE_mcp_fixture_server") to spawn.

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = handle_request(&line);
        writeln!(stdout, "{}", response).unwrap();
        stdout.flush().unwrap();
    }
}

fn handle_request(line: &str) -> String {
    let req: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return jsonrpc_error(
                serde_json::Value::Null,
                -32700,
                &format!("Parse error: {}", e),
            );
        }
    };

    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");

    match method {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "hudhud-mcp-fixture",
                    "version": "1.0.0"
                }
            });
            jsonrpc_result(id, &result)
        }
        "tools/list" => {
            let result = serde_json::json!({
                "tools": [
                    {
                        "name": "echo",
                        "description": "Echo back the input text",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string", "description": "Text to echo" }
                            },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "add",
                        "description": "Add two numbers",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "a": { "type": "number", "description": "First number" },
                                "b": { "type": "number", "description": "Second number" }
                            },
                            "required": ["a", "b"]
                        }
                    }
                ]
            });
            jsonrpc_result(id, &result)
        }
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or_default();

            match tool_name {
                "echo" => {
                    let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let result = serde_json::json!({
                        "content": [{ "type": "text", "text": text }]
                    });
                    jsonrpc_result(id, &result)
                }
                "add" => {
                    let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let sum = a + b;
                    let result = serde_json::json!({
                        "content": [{ "type": "text", "text": sum.to_string() }]
                    });
                    jsonrpc_result(id, &result)
                }
                _ => jsonrpc_error(id, -32601, &format!("Unknown tool: {}", tool_name)),
            }
        }
        "notifications/initialized" => {
            // No response for notifications
            String::new()
        }
        _ => jsonrpc_error(id, -32601, &format!("Method not found: {}", method)),
    }
}

fn jsonrpc_result(id: serde_json::Value, result: &serde_json::Value) -> String {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    resp.to_string()
}

fn jsonrpc_error(id: serde_json::Value, code: i64, message: &str) -> String {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    });
    resp.to_string()
}
