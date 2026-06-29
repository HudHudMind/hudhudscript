//! Shared Tesseract OCR wrapper (Kural 7).

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
    Extract,
    ExtractWithConfidence,
    Languages,
    Pdf,
    IsAvailable,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "extract" => Ok(Self::Extract),
            "extract_with_confidence" => Ok(Self::ExtractWithConfidence),
            "languages" => Ok(Self::Languages),
            "pdf" => Ok(Self::Pdf),
            "is_available" => Ok(Self::IsAvailable),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Extract => ocr_extract(args),
        ScriptMethodId::ExtractWithConfidence => ocr_extract_with_confidence(args),
        ScriptMethodId::Languages => ocr_languages(args),
        ScriptMethodId::Pdf => ocr_pdf(args),
        ScriptMethodId::IsAvailable => ocr_is_available(args),
    }
}

/// Main entry point (kept for backward compat).

fn is_tesseract_available() -> bool {
    std::process::Command::new("which")
        .arg("tesseract")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ensure_tesseract() -> HudHudResult<()> {
    if !is_tesseract_available() {
        return Err(runtime_error(
            "tesseract is not installed. Install it with: sudo apt install tesseract-ocr",
        ));
    }
    Ok(())
}

pub fn ocr_extract(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "ocr.extract() requires at least 1 argument: image_path",
        ));
    }
    ensure_tesseract()?;
    let image_path = require_str(&args[0], "ocr.extract image_path")?.to_string();
    let language = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cmd = std::process::Command::new("tesseract");
    cmd.arg(&image_path).arg("stdout");
    if let Some(ref lang) = language {
        cmd.arg("-l").arg(lang);
    }
    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("Failed to execute tesseract: {}", e)))?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Value16::string(text))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(runtime_error(format!(
            "tesseract failed: {}",
            stderr.trim()
        )))
    }
}

pub fn ocr_extract_with_confidence(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "ocr.extract_with_confidence() requires at least 1 argument: image_path",
        ));
    }
    ensure_tesseract()?;
    let image_path = require_str(&args[0], "ocr.extract_with_confidence image_path")?.to_string();
    let language = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cmd = std::process::Command::new("tesseract");
    cmd.arg(&image_path).arg("stdout").arg("tsv");
    if let Some(ref lang) = language {
        cmd.arg("-l").arg(lang);
    }
    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("Failed to execute tesseract: {}", e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(runtime_error(format!(
            "tesseract failed: {}",
            stderr.trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut text_parts: Vec<String> = Vec::new();
    let mut total_conf: f64 = 0.0;
    let mut conf_count: usize = 0;
    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 12 {
            let conf_str = cols[10].trim();
            let word = cols[11].trim();
            if !word.is_empty() {
                text_parts.push(word.to_string());
                if let Ok(c) = conf_str.parse::<f64>() {
                    if c >= 0.0 {
                        total_conf += c;
                        conf_count += 1;
                    }
                }
            }
        }
    }
    let text = text_parts.join(" ");
    let avg_confidence = if conf_count > 0 {
        total_conf / conf_count as f64
    } else {
        0.0
    };
    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("text".to_string(), Value16::string(text));
    result.insert("confidence".to_string(), Value16::number(avg_confidence));
    Ok(Value16::object(result))
}

pub fn ocr_languages(_args: &[Value16]) -> HudHudResult<Value16> {
    ensure_tesseract()?;
    let output = std::process::Command::new("tesseract")
        .arg("--list-langs")
        .output()
        .map_err(|e| runtime_error(format!("Failed to execute tesseract: {}", e)))?;
    // --list-langs may write to stderr on some versions
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all_output = format!("{}{}", stdout, stderr);
    let langs: Vec<Value16> = all_output
        .lines()
        .skip(1)
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| Value16::string(l.to_string()))
        .collect();
    if langs.is_empty() && !output.status.success() {
        return Err(runtime_error(format!(
            "tesseract --list-langs failed: {}",
            stderr.trim()
        )));
    }
    Ok(Value16::array(langs))
}

pub fn ocr_pdf(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "ocr.pdf() requires at least 2 arguments: image_path, output_path",
        ));
    }
    ensure_tesseract()?;
    let image_path = require_str(&args[0], "ocr.pdf image_path")?.to_string();
    let output_path = require_str(&args[1], "ocr.pdf output_path")?.to_string();
    let language = args.get(2).and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cmd = std::process::Command::new("tesseract");
    cmd.arg(&image_path).arg(&output_path);
    if let Some(ref lang) = language {
        cmd.arg("-l").arg(lang);
    }
    cmd.arg("pdf");

    let mut result = hudhudscript_bytecode::ObjMap::default();
    match cmd.output() {
        Ok(output) => {
            let success = output.status.success();
            result.insert("ok".to_string(), Value16::bool_(success));
            if success {
                result.insert(
                    "path".to_string(),
                    Value16::string(format!("{}.pdf", output_path)),
                );
            } else {
                result.insert(
                    "error".to_string(),
                    Value16::string(String::from_utf8_lossy(&output.stderr).trim().to_string()),
                );
            }
        }
        Err(e) => {
            result.insert("ok".to_string(), Value16::bool_(false));
            result.insert(
                "error".to_string(),
                Value16::string(format!("Failed to execute tesseract: {}", e)),
            );
        }
    }
    Ok(Value16::object(result))
}

pub fn ocr_is_available(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::bool_(is_tesseract_available()))
}

fn require_str<'a>(val: &'a Value16, ctx: &str) -> HudHudResult<&'a str> {
    val.as_str()
        .ok_or_else(|| type_error("string", val.type_name_str(), ctx))
}
