//! Shared XDG Desktop Integration — base dirs, .desktop files, MIME, launch
//! (Kural 7 — single source of truth for VM + interpreter).

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
    DataHome,
    ConfigHome,
    CacheHome,
    RuntimeDir,
    DesktopFiles,
    ParseDesktop,
    MimeType,
    Launch,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "data_home" => Ok(Self::DataHome),
            "config_home" => Ok(Self::ConfigHome),
            "cache_home" => Ok(Self::CacheHome),
            "runtime_dir" => Ok(Self::RuntimeDir),
            "desktop_files" => Ok(Self::DesktopFiles),
            "parse_desktop" => Ok(Self::ParseDesktop),
            "mime_type" => Ok(Self::MimeType),
            "launch" => Ok(Self::Launch),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::DataHome => xdg_data_home(args),
        ScriptMethodId::ConfigHome => xdg_config_home(args),
        ScriptMethodId::CacheHome => xdg_cache_home(args),
        ScriptMethodId::RuntimeDir => xdg_runtime_dir(args),
        ScriptMethodId::DesktopFiles => xdg_desktop_files(args),
        ScriptMethodId::ParseDesktop => xdg_parse_desktop(args),
        ScriptMethodId::MimeType => xdg_mime_type(args),
        ScriptMethodId::Launch => xdg_launch(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn xdg_data_home(_args: &[Value16]) -> HudHudResult<Value16> {
    let path = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        format!("{}/.local/share", home)
    });
    Ok(Value16::string(path))
}

pub fn xdg_config_home(_args: &[Value16]) -> HudHudResult<Value16> {
    let path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        format!("{}/.config", home)
    });
    Ok(Value16::string(path))
}

pub fn xdg_cache_home(_args: &[Value16]) -> HudHudResult<Value16> {
    let path = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        format!("{}/.cache", home)
    });
    Ok(Value16::string(path))
}

pub fn xdg_runtime_dir(_args: &[Value16]) -> HudHudResult<Value16> {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(path) => Ok(Value16::string(path)),
        Err(_) => Ok(Value16::null()),
    }
}

pub fn xdg_desktop_files(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut dirs: Vec<String> = vec!["/usr/share/applications".to_string()];
    let data_home = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::new());
        format!("{}/.local/share", home)
    });
    dirs.push(format!("{}/applications", data_home));
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for dir in data_dirs.split(':') {
        let app_dir = format!("{}/applications", dir);
        if !dirs.contains(&app_dir) {
            dirs.push(app_dir);
        }
    }

    let mut files: Vec<Value16> = Vec::new();
    for dir in &dirs {
        let dir_path = std::path::Path::new(dir);
        if dir_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        files.push(Value16::string(path.to_string_lossy().to_string()));
                    }
                }
            }
        }
    }
    Ok(Value16::array(files))
}

pub fn xdg_parse_desktop(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "xdg.parse_desktop")?.to_string();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| runtime_error(format!("xdg.parse_desktop: cannot read '{}': {}", path, e)))?;

    let mut result = HashMap::new();
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') && in_desktop_entry {
            break;
        }
        if !in_desktop_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Name" | "Exec" | "Icon" | "Categories" | "Type" | "Comment" => {
                    result.insert(key.to_string(), Value16::string(value.to_string()));
                }
                _ => {}
            }
        }
    }

    for key in &["Name", "Exec", "Icon", "Categories", "Type", "Comment"] {
        result.entry(key.to_string()).or_insert_with(Value16::null);
    }
    Ok(Value16::object(result))
}

pub fn xdg_mime_type(args: &[Value16]) -> HudHudResult<Value16> {
    let file_path = require_str(args, 0, "xdg.mime_type")?.to_string();
    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "csv" => "text/csv",
        "xml" => "text/xml",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "md" => "text/markdown",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        "ini" | "cfg" => "text/plain",
        "sh" | "bash" | "zsh" => "application/x-shellscript",
        "py" => "text/x-python",
        "rs" => "text/x-rust",
        "ts" => "application/typescript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application/vnd.oasis.opendocument.text",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "bz2" => "application/x-bzip2",
        "xz" => "application/x-xz",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        "desktop" => "application/x-desktop",
        _ => "application/octet-stream",
    };
    Ok(Value16::string(mime.to_string()))
}

pub fn xdg_launch(args: &[Value16]) -> HudHudResult<Value16> {
    let desktop_path = require_str(args, 0, "xdg.launch")?.to_string();
    let content = std::fs::read_to_string(&desktop_path)
        .map_err(|e| runtime_error(format!("xdg.launch: cannot read '{}': {}", desktop_path, e)))?;

    let mut exec_line: Option<String> = None;
    let mut in_desktop_entry = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') && in_desktop_entry {
            break;
        }
        if in_desktop_entry {
            if let Some(val) = line.strip_prefix("Exec=") {
                exec_line = Some(val.trim().to_string());
                break;
            }
        }
    }

    let exec = exec_line.ok_or_else(|| {
        runtime_error(format!(
            "xdg.launch: no Exec field found in '{}'",
            desktop_path
        ))
    })?;

    let cleaned: String = exec
        .split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<&str>>()
        .join(" ");

    let mut parts = cleaned.split_whitespace();
    let program = parts.next().ok_or_else(|| {
        runtime_error(format!(
            "xdg.launch: empty Exec field in '{}'",
            desktop_path
        ))
    })?;
    let cmd_args: Vec<&str> = parts.collect();

    let child = std::process::Command::new(program)
        .args(&cmd_args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| runtime_error(format!("xdg.launch: failed to launch '{}': {}", program, e)))?;

    let mut result = HashMap::new();
    result.insert("pid".to_string(), Value16::number(child.id() as f64));
    result.insert("command".to_string(), Value16::string(cleaned));
    Ok(Value16::object(result))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, op: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            op, idx
        ))),
    }
}
