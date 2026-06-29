//! Shared download-manager builtins — single source of truth for the VM and
//! interpreter runtimes (Kural 7).
//!
//! HTTP via `reqwest::blocking`. Resume support relies on the server
//! advertising `Accept-Ranges: bytes` and honours the `Range` header.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use crate::download_helpers;

pub(crate) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub(crate) fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;
use std::io::Write;

/// Main entry point used by the VM's module dispatcher.
/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    File,
    FileWithProgress,
    Resume,
    Head,
    Text,
    Json,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(Self::File),
            "file_with_progress" => Ok(Self::FileWithProgress),
            "resume" => Ok(Self::Resume),
            "head" => Ok(Self::Head),
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::File => download_file(args),
        ScriptMethodId::FileWithProgress => download_file_with_progress(args),
        ScriptMethodId::Resume => download_resume(args),
        ScriptMethodId::Head => download_head(args),
        ScriptMethodId::Text => download_text(args),
        ScriptMethodId::Json => download_json(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn download_file(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.file")?.to_string();
    let output_path = download_helpers::require_str(args, 1, "download.file")?.to_string();

    let client = download_helpers::build_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.file request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(runtime_error(format!(
            "download.file got HTTP {}",
            response.status().as_u16()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| runtime_error(format!("download.file read body failed: {}", e)))?;

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&output_path, &bytes).map_err(|e| {
        runtime_error(format!(
            "download.file write '{}' failed: {}",
            output_path, e
        ))
    })?;

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("size".to_string(), Value16::number(bytes.len() as f64));
    result.insert("path".to_string(), Value16::string(output_path));
    result.insert(
        "status".to_string(),
        Value16::string("complete".to_string()),
    );
    Ok(Value16::object(result))
}

pub fn download_file_with_progress(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.file_with_progress")?.to_string();
    let output_path = download_helpers::require_str(args, 1, "download.file_with_progress")?.to_string();

    let client = download_helpers::build_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.file_with_progress request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(runtime_error(format!(
            "download.file_with_progress got HTTP {}",
            response.status().as_u16()
        )));
    }

    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = std::fs::File::create(&output_path).map_err(|e| {
        runtime_error(format!(
            "download.file_with_progress create '{}' failed: {}",
            output_path, e
        ))
    })?;

    let mut downloaded: u64 = 0;
    let mut reader = response;
    let mut buf = [0u8; 8192];
    let mut progress_events: Vec<Value16> = Vec::new();

    loop {
        let n = std::io::Read::read(&mut reader, &mut buf).map_err(|e| {
            runtime_error(format!("download.file_with_progress read failed: {}", e))
        })?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| {
            runtime_error(format!("download.file_with_progress write failed: {}", e))
        })?;
        downloaded += n as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let mut evt = hudhudscript_bytecode::ObjMap::default();
        evt.insert("downloaded".to_string(), Value16::number(downloaded as f64));
        evt.insert("total".to_string(), Value16::number(total_size as f64));
        evt.insert("percent".to_string(), Value16::number(percent));
        progress_events.push(Value16::object(evt));
    }

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("size".to_string(), Value16::number(downloaded as f64));
    result.insert("path".to_string(), Value16::string(output_path));
    result.insert(
        "status".to_string(),
        Value16::string("complete".to_string()),
    );
    result.insert("progress".to_string(), Value16::array(progress_events));
    Ok(Value16::object(result))
}

pub fn download_resume(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.resume")?.to_string();
    let output_path = download_helpers::require_str(args, 1, "download.resume")?.to_string();

    let client = download_helpers::build_client()?;

    let existing_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let head_resp = client
        .head(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.resume HEAD failed: {}", e)))?;

    let accept_ranges = head_resp
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none")
        .to_string();

    let total_size = head_resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    if existing_size > 0 && existing_size >= total_size && total_size > 0 {
        let mut result = hudhudscript_bytecode::ObjMap::default();
        result.insert("size".to_string(), Value16::number(existing_size as f64));
        result.insert("path".to_string(), Value16::string(output_path));
        result.insert(
            "status".to_string(),
            Value16::string("already_complete".to_string()),
        );
        result.insert("resumed".to_string(), Value16::bool_(false));
        return Ok(Value16::object(result));
    }

    let supports_range = accept_ranges.contains("bytes");
    let req = if supports_range && existing_size > 0 {
        client
            .get(&url)
            .header("Range", format!("bytes={}-", existing_size))
    } else {
        client.get(&url)
    };

    let response = req
        .send()
        .map_err(|e| runtime_error(format!("download.resume GET failed: {}", e)))?;

    let status_code = response.status().as_u16();
    if status_code != 200 && status_code != 206 {
        return Err(runtime_error(format!(
            "download.resume got HTTP {}",
            status_code
        )));
    }

    let resumed = status_code == 206;

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let bytes = response
        .bytes()
        .map_err(|e| runtime_error(format!("download.resume read body failed: {}", e)))?;

    if resumed {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&output_path)
            .map_err(|e| {
                runtime_error(format!(
                    "download.resume open '{}' failed: {}",
                    output_path, e
                ))
            })?;
        file.write_all(&bytes)
            .map_err(|e| runtime_error(format!("download.resume write failed: {}", e)))?;
    } else {
        std::fs::write(&output_path, &bytes).map_err(|e| {
            runtime_error(format!(
                "download.resume write '{}' failed: {}",
                output_path, e
            ))
        })?;
    }

    let final_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("size".to_string(), Value16::number(final_size as f64));
    result.insert("path".to_string(), Value16::string(output_path));
    result.insert(
        "status".to_string(),
        Value16::string("complete".to_string()),
    );
    result.insert("resumed".to_string(), Value16::bool_(resumed));
    Ok(Value16::object(result))
}

pub fn download_head(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.head")?.to_string();

    let client = download_helpers::build_client()?;
    let response = client
        .head(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.head request failed: {}", e)))?;

    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let accept_ranges = response
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert(
        "content_length".to_string(),
        Value16::number(content_length),
    );
    result.insert("accept_ranges".to_string(), Value16::string(accept_ranges));
    result.insert("content_type".to_string(), Value16::string(content_type));
    Ok(Value16::object(result))
}

pub fn download_text(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.text")?.to_string();

    let client = download_helpers::build_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.text request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(runtime_error(format!(
            "download.text got HTTP {}",
            response.status().as_u16()
        )));
    }

    let text = response
        .text()
        .map_err(|e| runtime_error(format!("download.text read body failed: {}", e)))?;
    Ok(Value16::string(text))
}

pub fn download_json(args: &[Value16]) -> HudHudResult<Value16> {
    let url = download_helpers::require_str(args, 0, "download.json")?.to_string();

    let client = download_helpers::build_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|e| runtime_error(format!("download.json request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(runtime_error(format!(
            "download.json got HTTP {}",
            response.status().as_u16()
        )));
    }

    let text = response
        .text()
        .map_err(|e| runtime_error(format!("download.json read body failed: {}", e)))?;

    let json_val: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| runtime_error(format!("download.json parse failed: {}", e)))?;

    Ok(download_helpers::serde_json_to_value16(&json_val))
}

