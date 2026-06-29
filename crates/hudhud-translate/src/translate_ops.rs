//! Shared LibreTranslate API client (Kural 7).

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

fn default_api_url() -> String {
    std::env::var("LIBRETRANSLATE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Text,
    Detect,
    Languages,
    Batch,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "detect" => Ok(Self::Detect),
            "languages" => Ok(Self::Languages),
            "batch" => Ok(Self::Batch),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Text => translate_text(args),
        ScriptMethodId::Detect => translate_detect(args),
        ScriptMethodId::Languages => translate_languages(args),
        ScriptMethodId::Batch => translate_batch(args),
    }
}

/// Main entry point (kept for backward compat).

fn get_api_url(args: &[Value16], idx: usize) -> String {
    args.get(idx)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(default_api_url)
}

fn require_str(args: &[Value16], idx: usize, name: &str, method: &str) -> HudHudResult<String> {
    match args.get(idx) {
        Some(v) => v.as_str().map(|s| s.to_string()).ok_or_else(|| {
            type_error(
                "string",
                v.type_name_str(),
                &format!("{} '{}'", method, name),
            )
        }),
        None => Err(runtime_error(format!(
            "{} requires '{}' argument at position {}",
            method, name, idx
        ))),
    }
}

fn build_client(timeout_secs: u64) -> HudHudResult<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| runtime_error(format!("Failed to create HTTP client: {}", e)))
}

pub fn translate_text(args: &[Value16]) -> HudHudResult<Value16> {
    let method = "translate.text";
    if args.len() < 3 {
        return Err(runtime_error(
            "translate.text() requires at least 3 arguments: text, source_lang, target_lang",
        ));
    }
    let text = require_str(args, 0, "text", method)?;
    let source = require_str(args, 1, "source_lang", method)?;
    let target = require_str(args, 2, "target_lang", method)?;
    let api_url = get_api_url(args, 3);
    let url = format!("{}/translate", api_url);

    let mut body = serde_json::Map::new();
    body.insert("q".to_string(), serde_json::Value::String(text));
    body.insert("source".to_string(), serde_json::Value::String(source));
    body.insert("target".to_string(), serde_json::Value::String(target));

    let client = build_client(30)?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&serde_json::Value::Object(body))
        .send()
        .map_err(|e| runtime_error(format!("LibreTranslate request failed: {}", e)))?;
    let status = resp.status().as_u16();
    let resp_text = resp
        .text()
        .map_err(|e| runtime_error(format!("Failed to read response body: {}", e)))?;
    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| runtime_error(format!("Failed to parse JSON response: {}", e)))?;
    if status >= 400 {
        let error_msg = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("<no error field in API response>");
        return Err(runtime_error(format!(
            "LibreTranslate error ({}): {}",
            status, error_msg
        )));
    }
    match json.get("translatedText").and_then(|v| v.as_str()) {
        Some(translated) => Ok(Value16::string(translated.to_string())),
        None => Ok(hudhud_http::json::serde_to_value(&json)),
    }
}

pub fn translate_detect(args: &[Value16]) -> HudHudResult<Value16> {
    let method = "translate.detect";
    if args.is_empty() {
        return Err(runtime_error(
            "translate.detect() requires at least 1 argument: text",
        ));
    }
    let text = require_str(args, 0, "text", method)?;
    let api_url = get_api_url(args, 1);
    let url = format!("{}/detect", api_url);

    let mut body = serde_json::Map::new();
    body.insert("q".to_string(), serde_json::Value::String(text));
    let client = build_client(30)?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&serde_json::Value::Object(body))
        .send()
        .map_err(|e| runtime_error(format!("LibreTranslate request failed: {}", e)))?;
    let status = resp.status().as_u16();
    let resp_text = resp
        .text()
        .map_err(|e| runtime_error(format!("Failed to read response body: {}", e)))?;
    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| runtime_error(format!("Failed to parse JSON response: {}", e)))?;
    if status >= 400 {
        let error_msg = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("<no error field in API response>");
        return Err(runtime_error(format!(
            "LibreTranslate error ({}): {}",
            status, error_msg
        )));
    }
    if let Some(arr) = json.as_array() {
        if let Some(first) = arr.first() {
            let mut result: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
            if let Some(lang) = first.get("language").and_then(|v| v.as_str()) {
                result.insert("language".to_string(), Value16::string(lang.to_string()));
            }
            if let Some(conf) = first.get("confidence").and_then(|v| v.as_f64()) {
                result.insert("confidence".to_string(), Value16::number(conf));
            }
            return Ok(Value16::object(result));
        }
    }
    Ok(hudhud_http::json::serde_to_value(&json))
}

pub fn translate_languages(args: &[Value16]) -> HudHudResult<Value16> {
    let api_url = get_api_url(args, 0);
    let url = format!("{}/languages", api_url);
    let client = build_client(30)?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| runtime_error(format!("LibreTranslate request failed: {}", e)))?;
    let status = resp.status().as_u16();
    let resp_text = resp
        .text()
        .map_err(|e| runtime_error(format!("Failed to read response body: {}", e)))?;
    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| runtime_error(format!("Failed to parse JSON response: {}", e)))?;
    if status >= 400 {
        return Err(runtime_error(format!(
            "LibreTranslate error ({}): {}",
            status, resp_text
        )));
    }
    if let Some(arr) = json.as_array() {
        let langs: Vec<Value16> = arr
            .iter()
            .map(|entry| {
                let mut obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
                if let Some(code) = entry.get("code").and_then(|v| v.as_str()) {
                    obj.insert("code".to_string(), Value16::string(code.to_string()));
                }
                if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                    obj.insert("name".to_string(), Value16::string(name.to_string()));
                }
                Value16::object(obj)
            })
            .collect();
        return Ok(Value16::array(langs));
    }
    Ok(hudhud_http::json::serde_to_value(&json))
}

pub fn translate_batch(args: &[Value16]) -> HudHudResult<Value16> {
    let method = "translate.batch";
    if args.len() < 3 {
        return Err(runtime_error(
            "translate.batch() requires at least 3 arguments: texts_array, source_lang, target_lang",
        ));
    }
    let texts = args[0]
        .as_array()
        .ok_or_else(|| {
            type_error(
                "array",
                args[0].type_name_str(),
                "translate.batch texts_array",
            )
        })?
        .clone();
    let source = require_str(args, 1, "source_lang", method)?;
    let target = require_str(args, 2, "target_lang", method)?;
    let api_url = get_api_url(args, 3);
    let url = format!("{}/translate", api_url);

    let client = build_client(60)?;
    let mut results: Vec<Value16> = Vec::with_capacity(texts.len());

    for (i, text_val) in texts.iter().enumerate() {
        let text = text_val.as_str().ok_or_else(|| {
            type_error(
                "string",
                text_val.type_name_str(),
                &format!("translate.batch texts_array[{}]", i),
            )
        })?;

        let mut body = serde_json::Map::new();
        body.insert("q".to_string(), serde_json::Value::String(text.to_string()));
        body.insert(
            "source".to_string(),
            serde_json::Value::String(source.clone()),
        );
        body.insert(
            "target".to_string(),
            serde_json::Value::String(target.clone()),
        );

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::Value::Object(body))
            .send()
            .map_err(|e| {
                runtime_error(format!(
                    "LibreTranslate request failed for item {}: {}",
                    i, e
                ))
            })?;
        let status = resp.status().as_u16();
        let resp_text = resp.text().map_err(|e| {
            runtime_error(format!(
                "Failed to read response body for item {}: {}",
                i, e
            ))
        })?;
        let json: serde_json::Value = serde_json::from_str(&resp_text).map_err(|e| {
            runtime_error(format!(
                "Failed to parse JSON response for item {}: {}",
                i, e
            ))
        })?;
        if status >= 400 {
            let error_msg = json
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("<no error field in API response>");
            return Err(runtime_error(format!(
                "LibreTranslate error ({}) for item {}: {}",
                status, i, error_msg
            )));
        }
        match json.get("translatedText").and_then(|v| v.as_str()) {
            Some(translated) => results.push(Value16::string(translated.to_string())),
            None => results.push(hudhud_http::json::serde_to_value(&json)),
        }
    }
    Ok(Value16::array(results))
}
