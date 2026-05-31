//! Shared HTTP client builtin — used by both VM and interpreter.
//!
//! Provides: http.get, post, put, delete, patch
//!
//! Returns standardised response: `{ status, ok, headers, body, json }`

use crate::json::{serde_to_value, value_to_json_string};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
use std::collections::HashMap;

/// Parsed HTTP request configuration extracted from SharedValue args.
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub query_params: Vec<(String, String)>,
    pub timeout_secs: u64,
}

fn value_display(val: &Value16) -> String {
    if let Some(s) = val.as_str() {
        s.to_string()
    } else if let Some(n) = val.as_number() {
        crate::json::format_number(n)
    } else if let Some(b) = val.as_bool() {
        if b {
            "true".to_string()
        } else {
            "false".to_string()
        }
    } else if val.is_null() {
        "null".to_string()
    } else {
        val.display_string()
    }
}

fn execute_request(http_method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(format!(
            "http.{}() requires a URL argument",
            http_method.to_lowercase()
        )));
    }
    let url = args[0]
        .as_str()
        .ok_or_else(|| runtime_error("http URL must be a string"))?
        .to_string();

    let config = args.get(1).and_then(|v| v.as_object());
    let mut headers_map = HashMap::new();
    let mut body_str = None;
    let mut query_params = Vec::new();
    let mut timeout_secs = 30u64;

    if let Some(cfg) = config {
        if let Some(hdrs) = cfg.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in hdrs {
                headers_map.insert(k.clone(), value_display(v));
            }
        }
        if let Some(body_val) = cfg.get("body") {
            body_str = Some(value_to_json_string(body_val));
        }
        if let Some(qp) = cfg.get("query").and_then(|v| v.as_object()) {
            for (k, v) in qp {
                query_params.push((k.clone(), value_display(v)));
            }
        }
        if let Some(t) = cfg.get("timeout").and_then(|v| v.as_number()) {
            timeout_secs = t as u64;
        }
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| runtime_error(format!("HTTP client error: {}", e)))?;

    let mut req = match http_method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => unreachable!(),
    };

    for (k, v) in &headers_map {
        let clean_k = k.replace(['\r', '\n'], "");
        let clean_v = v.replace(['\r', '\n'], "");
        req = req.header(clean_k.as_str(), clean_v.as_str());
    }
    for (k, v) in &query_params {
        req = req.query(&[(k.as_str(), v.as_str())]);
    }
    if let Some(body) = &body_str {
        req = req
            .header("Content-Type", "application/json")
            .body(body.clone());
    }

    let response = req
        .send()
        .map_err(|e| runtime_error(format!("HTTP request failed: {}", e)))?;

    let status = response.status().as_u16();
    let ok = response.status().is_success();
    let resp_headers: HashMap<String, Value16> = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                Value16::string(v.to_str().unwrap_or("").to_string()),
            )
        })
        .collect();
    let body_text = response
        .text()
        .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
    let json_val: Value16 = serde_json::from_str::<serde_json::Value>(&body_text)
        .ok()
        .map(|j| serde_to_value(&j))
        .unwrap_or_else(Value16::null);

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value16::number(status as f64));
    result.insert("ok".to_string(), Value16::bool_(ok));
    result.insert("headers".to_string(), Value16::object(resp_headers));
    result.insert("body".to_string(), Value16::string(body_text));
    result.insert("json".to_string(), json_val);

    Ok(Value16::object(result))
}

pub fn get(args: &[Value16]) -> HudHudResult<Value16> {
    execute_request("GET", args)
}

pub fn post(args: &[Value16]) -> HudHudResult<Value16> {
    execute_request("POST", args)
}

pub fn put(args: &[Value16]) -> HudHudResult<Value16> {
    execute_request("PUT", args)
}

pub fn delete(args: &[Value16]) -> HudHudResult<Value16> {
    execute_request("DELETE", args)
}

pub fn patch(args: &[Value16]) -> HudHudResult<Value16> {
    execute_request("PATCH", args)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethodId {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl std::str::FromStr for HttpMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            "patch" => Ok(Self::Patch),
            _ => Err(runtime_error(format!("Unknown HTTP method: {}", s))),
        }
    }
}

impl HttpMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Get => get(args),
            Self::Post => post(args),
            Self::Put => put(args),
            Self::Delete => delete(args),
            Self::Patch => patch(args),
        }
    }
}

/// Return the URL's host string, for sandbox allowed-hosts checks.
pub fn parse_url_host(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}
