//! Shared PDF (poppler-utils) wrapper (Kural 7).

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
    Read,
    Info,
    Merge,
    Split,
    ToImages,
    PageCount,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "info" => Ok(Self::Info),
            "merge" => Ok(Self::Merge),
            "split" => Ok(Self::Split),
            "to_images" => Ok(Self::ToImages),
            "page_count" => Ok(Self::PageCount),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Read => pdf_read(args),
        ScriptMethodId::Info => pdf_info(args),
        ScriptMethodId::Merge => pdf_merge(args),
        ScriptMethodId::Split => pdf_split(args),
        ScriptMethodId::ToImages => pdf_to_images(args),
        ScriptMethodId::PageCount => pdf_page_count(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn pdf_read(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "pdf.read")?.to_string();
    let output = std::process::Command::new("pdftotext")
        .arg(&path)
        .arg("-")
        .output()
        .map_err(|e| runtime_error(format!("pdf.read: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.read: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(Value16::string(
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

pub fn pdf_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "pdf.info")?.to_string();
    let output = std::process::Command::new("pdfinfo")
        .arg(&path)
        .output()
        .map_err(|e| runtime_error(format!("pdf.info: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.info: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut info: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Pages" => {
                    let n = value.parse::<f64>().unwrap_or(0.0);
                    info.insert("pages".to_string(), Value16::number(n));
                }
                "Title" => {
                    info.insert("title".to_string(), Value16::string(value.to_string()));
                }
                "Author" => {
                    info.insert("author".to_string(), Value16::string(value.to_string()));
                }
                "Creator" => {
                    info.insert("creator".to_string(), Value16::string(value.to_string()));
                }
                "Producer" => {
                    info.insert("producer".to_string(), Value16::string(value.to_string()));
                }
                "Page size" => {
                    info.insert("page_size".to_string(), Value16::string(value.to_string()));
                }
                _ => {}
            }
        }
    }
    for key in &[
        "pages",
        "title",
        "author",
        "creator",
        "producer",
        "page_size",
    ] {
        info.entry(key.to_string()).or_insert_with(Value16::null);
    }
    Ok(Value16::object(info))
}

pub fn pdf_merge(args: &[Value16]) -> HudHudResult<Value16> {
    let inputs = args
        .first()
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            args.first().map_or_else(
                || type_error("array of paths", "missing argument", "pdf.merge"),
                |v| type_error("array of paths", v.type_name_str(), "pdf.merge"),
            )
        })?
        .clone();
    let output_path = require_str(args, 1, "pdf.merge")?.to_string();

    let mut cmd = std::process::Command::new("pdfunite");
    for input in &inputs {
        let s = input
            .as_str()
            .ok_or_else(|| type_error("string", input.type_name_str(), "pdf.merge"))?;
        cmd.arg(s);
    }
    cmd.arg(&output_path);

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("pdf.merge: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.merge: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(Value16::bool_(true))
}

pub fn pdf_split(args: &[Value16]) -> HudHudResult<Value16> {
    let input_path = require_str(args, 0, "pdf.split")?.to_string();
    let output_dir = require_str(args, 1, "pdf.split")?.to_string();
    std::fs::create_dir_all(&output_dir).map_err(|e| runtime_error(format!("pdf.split: {}", e)))?;
    let pattern = format!("{}/page-%d.pdf", output_dir);
    let output = std::process::Command::new("pdfseparate")
        .arg(&input_path)
        .arg(&pattern)
        .output()
        .map_err(|e| runtime_error(format!("pdf.split: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.split: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let mut files: Vec<Value16> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("page-") && name.ends_with(".pdf") {
                    files.push(Value16::string(p.to_string_lossy().into_owned()));
                }
            }
        }
    }
    files.sort_by(|a, b| match (a.as_str(), b.as_str()) {
        (Some(sa), Some(sb)) => sa.cmp(sb),
        _ => std::cmp::Ordering::Equal,
    });
    Ok(Value16::array(files))
}

pub fn pdf_to_images(args: &[Value16]) -> HudHudResult<Value16> {
    let input_path = require_str(args, 0, "pdf.to_images")?.to_string();
    let output_dir = require_str(args, 1, "pdf.to_images")?.to_string();
    let format = args
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("png")
        .to_string();

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| runtime_error(format!("pdf.to_images: {}", e)))?;
    let prefix = format!("{}/page", output_dir);
    let fmt_flag = match format.as_str() {
        "jpeg" | "jpg" => "-jpeg",
        _ => "-png",
    };
    let output = std::process::Command::new("pdftoppm")
        .arg(fmt_flag)
        .arg(&input_path)
        .arg(&prefix)
        .output()
        .map_err(|e| runtime_error(format!("pdf.to_images: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.to_images: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let ext = match format.as_str() {
        "jpeg" | "jpg" => "jpg",
        _ => "png",
    };
    let mut files: Vec<Value16> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("page") && name.ends_with(ext) {
                    files.push(Value16::string(p.to_string_lossy().into_owned()));
                }
            }
        }
    }
    files.sort_by(|a, b| match (a.as_str(), b.as_str()) {
        (Some(sa), Some(sb)) => sa.cmp(sb),
        _ => std::cmp::Ordering::Equal,
    });
    Ok(Value16::array(files))
}

pub fn pdf_page_count(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "pdf.page_count")?.to_string();
    let output = std::process::Command::new("pdfinfo")
        .arg(&path)
        .output()
        .map_err(|e| runtime_error(format!("pdf.page_count: {}", e)))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "pdf.page_count: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Pages:") {
            if let Ok(n) = rest.trim().parse::<f64>() {
                return Ok(Value16::number(n));
            }
        }
    }
    Err(runtime_error(
        "pdf.page_count: could not parse page count from pdfinfo output",
    ))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, op: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(type_error("string", "missing argument", op)),
    }
}
