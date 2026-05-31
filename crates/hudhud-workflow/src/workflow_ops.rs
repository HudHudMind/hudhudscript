//! Shared n8n workflow API client (Kural 7).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Trigger,
    List,
    Execute,
    Status,
    CreateWebhook,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trigger" => Ok(Self::Trigger),
            "list" => Ok(Self::List),
            "execute" => Ok(Self::Execute),
            "status" => Ok(Self::Status),
            "create_webhook" => Ok(Self::CreateWebhook),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Trigger => workflow_trigger(args),
        ScriptMethodId::List => workflow_list(args),
        ScriptMethodId::Execute => workflow_execute(args),
        ScriptMethodId::Status => workflow_status(args),
        ScriptMethodId::CreateWebhook => workflow_create_webhook(args),
    }
}

/// Main entry point (kept for backward compat).

fn build_client() -> HudHudResult<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| runtime_error(format!("workflow HTTP client error: {}", e)))
}

fn response_to_value(resp: reqwest::blocking::Response) -> HudHudResult<Value16> {
    let status = resp.status().as_u16();
    let ok = resp.status().is_success();
    let mut headers_map: HashMap<String, Value16> = HashMap::new();
    for (k, v) in resp.headers().iter() {
        if let Ok(val) = v.to_str() {
            headers_map.insert(k.as_str().to_string(), Value16::string(val.to_string()));
        }
    }
    let body_text = resp.text().unwrap_or_default();
    let json_value = serde_json::from_str::<serde_json::Value>(&body_text)
        .map(|j| hudhud_http::json::serde_to_value(&j))
        .unwrap_or_else(|_| Value16::null());

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value16::number(status as f64));
    result.insert("ok".to_string(), Value16::bool_(ok));
    result.insert("headers".to_string(), Value16::object(headers_map));
    result.insert("body".to_string(), Value16::string(body_text));
    result.insert("json".to_string(), json_value);
    Ok(Value16::object(result))
}

pub fn workflow_trigger(args: &[Value16]) -> HudHudResult<Value16> {
    let webhook_url = require_str(args, 0, "workflow.trigger")?.to_string();
    let default_data = Value16::object(HashMap::new());
    let data = args.get(1).unwrap_or(&default_data);
    let json_body = {
        let json_str = hudhud_http::json::value_to_json_string(data);
        serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    };

    let client = build_client()?;
    let resp = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .map_err(|e| runtime_error(format!("workflow.trigger failed: {}", e)))?;
    response_to_value(resp)
}

pub fn workflow_list(args: &[Value16]) -> HudHudResult<Value16> {
    let base_url = require_str(args, 0, "workflow.list")?.to_string();
    let api_key = require_str(args, 1, "workflow.list")?.to_string();
    let url = format!("{}/api/v1/workflows", base_url.trim_end_matches('/'));
    let client = build_client()?;
    let resp = client
        .get(&url)
        .header("X-N8N-API-KEY", &api_key)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| runtime_error(format!("workflow.list failed: {}", e)))?;
    response_to_value(resp)
}

pub fn workflow_execute(args: &[Value16]) -> HudHudResult<Value16> {
    let base_url = require_str(args, 0, "workflow.execute")?.to_string();
    let api_key = require_str(args, 1, "workflow.execute")?.to_string();
    let workflow_id = require_str(args, 2, "workflow.execute")?.to_string();
    let url = format!(
        "{}/api/v1/workflows/{}/execute",
        base_url.trim_end_matches('/'),
        workflow_id
    );
    let client = build_client()?;
    let resp = client
        .post(&url)
        .header("X-N8N-API-KEY", &api_key)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .map_err(|e| runtime_error(format!("workflow.execute failed: {}", e)))?;
    response_to_value(resp)
}

pub fn workflow_status(args: &[Value16]) -> HudHudResult<Value16> {
    let base_url = require_str(args, 0, "workflow.status")?.to_string();
    let api_key = require_str(args, 1, "workflow.status")?.to_string();
    let execution_id = require_str(args, 2, "workflow.status")?.to_string();
    let url = format!(
        "{}/api/v1/executions/{}",
        base_url.trim_end_matches('/'),
        execution_id
    );
    let client = build_client()?;
    let resp = client
        .get(&url)
        .header("X-N8N-API-KEY", &api_key)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| runtime_error(format!("workflow.status failed: {}", e)))?;
    response_to_value(resp)
}

pub fn workflow_create_webhook(args: &[Value16]) -> HudHudResult<Value16> {
    let port = args
        .first()
        .and_then(|v| v.as_number())
        .map(|n| n as u16)
        .ok_or_else(|| runtime_error("workflow.create_webhook: port argument required (number)"))?;

    let path = require_str(args, 1, "workflow.create_webhook")?.to_string();
    let callback_name = require_str(args, 2, "workflow.create_webhook")?.to_string();

    let path_normalized = if path.starts_with('/') {
        path.clone()
    } else {
        format!("/{}", path)
    };

    let listen_addr = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&listen_addr).map_err(|e| {
        runtime_error(format!(
            "workflow.create_webhook: cannot bind to {}: {}",
            listen_addr, e
        ))
    })?;
    listener.set_nonblocking(false).ok();

    let webhook_path = path_normalized.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                use std::io::{BufRead, BufReader, Write};
                let peer = stream.peer_addr().ok();
                let reader = BufReader::new(&stream);
                let mut lines = Vec::new();
                for line in reader.lines() {
                    match line {
                        Ok(l) if l.is_empty() => break,
                        Ok(l) => lines.push(l),
                        Err(_) => break,
                    }
                }
                let request_line = lines.first().cloned().unwrap_or_default();
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                let method = parts.first().copied().unwrap_or("");
                let req_path = parts.get(1).copied().unwrap_or("");
                if method == "POST" && req_path == webhook_path {
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 16\r\n\r\n{\"received\":true}";
                    let _ = stream.write_all(response.as_bytes());
                    eprintln!(
                        "[workflow webhook] received POST on {} from {:?}",
                        webhook_path, peer
                    );
                } else {
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
    });

    let url = format!("http://localhost:{}{}", port, path_normalized);
    let mut result = HashMap::new();
    result.insert("ok".to_string(), Value16::bool_(true));
    result.insert(
        "message".to_string(),
        Value16::string(format!(
            "Webhook server listening on {} (callback: {})",
            url, callback_name
        )),
    );
    result.insert("url".to_string(), Value16::string(url));
    Ok(Value16::object(result))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, op: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(runtime_error(format!("{}: argument {} required", op, idx))),
    }
}
