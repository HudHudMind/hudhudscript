use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::io::Read;

pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub fn thumbnail(args: &[Value16]) -> HudHudResult<Value16> {
    let video_path = require_str(args, 0, "media.thumbnail")?;
    let output_path = require_str(args, 1, "media.thumbnail")?;
    let time_seconds = require_num(args, 2, "media.thumbnail")?;

    let output = run_cmd(
        std::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .arg("-ss")
            .arg(format!("{}", time_seconds))
            .arg("-vframes")
            .arg("1")
            .arg(&output_path),
        "media.thumbnail",
    )?;
    let _ = output;
    Ok(Value16::string(output_path))
}

pub fn file_type(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "media.file_type")?;
    let mut f =
        std::fs::File::open(&path).map_err(|e| runtime_error(format!("media.file_type: {}", e)))?;
    let mut buf = [0u8; 16];
    let n = f
        .read(&mut buf)
        .map_err(|e| runtime_error(format!("media.file_type: {}", e)))?;
    let (kind, subkind) = detect_magic(&buf[..n]);
    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("kind".to_string(), Value16::string(kind.to_string()));
    m.insert("subkind".to_string(), Value16::string(subkind.to_string()));
    Ok(Value16::object(m))
}

pub fn detect_magic(buf: &[u8]) -> (&'static str, &'static str) {
    if buf.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return ("image", "png");
    }
    if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ("image", "jpeg");
    }
    if buf.starts_with(b"GIF89a") || buf.starts_with(b"GIF87a") {
        return ("image", "gif");
    }
    if buf.starts_with(b"BM") {
        return ("image", "bmp");
    }
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return ("image", "webp");
    }
    if buf.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return ("video", "mkv");
    }
    if buf.starts_with(b"ftyp") {
        return ("video", "mp4");
    }
    if buf.starts_with(&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]) {
        return ("video", "mp4");
    }
    if buf.starts_with(&[0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70]) {
        return ("video", "mp4");
    }
    if buf.starts_with(b"ID3") {
        return ("audio", "mp3");
    }
    if buf.starts_with(&[0xFF, 0xFB])
        || buf.starts_with(&[0xFF, 0xF3])
        || buf.starts_with(&[0xFF, 0xF2])
    {
        return ("audio", "mp3");
    }
    if buf.starts_with(b"OggS") {
        return ("audio", "ogg");
    }
    if buf.starts_with(b"fLaC") {
        return ("audio", "flac");
    }
    if buf.starts_with(b"RIFF") {
        return ("audio", "wav");
    }
    if buf.starts_with(b"FORM") {
        return ("audio", "aiff");
    }
    if buf.starts_with(b"MThd") {
        return ("audio", "midi");
    }
    if buf.len() >= 4
        && (&buf[0..4] == b"\x00\x00\x00 ftyp"
            || &buf[0..4] == b"\x00\x00\x00\x18ftyp"
            || &buf[0..4] == b"\x00\x00\x00\x14ftyp")
    {
        return ("video", "mp4");
    }
    ("unknown", "")
}

pub fn require_str(args: &[Value16], idx: usize, method: &str) -> HudHudResult<String> {
    let v = args
        .get(idx)
        .ok_or_else(|| runtime_error(format!("{}: missing arg {}", method, idx)))?;
    v.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| type_error("string", &v.type_name_str(), method))
}

pub fn require_num(args: &[Value16], idx: usize, method: &str) -> HudHudResult<f64> {
    let v = args
        .get(idx)
        .ok_or_else(|| runtime_error(format!("{}: missing arg {}", method, idx)))?;
    v.as_number()
        .ok_or_else(|| type_error("number", &v.type_name_str(), method))
}

pub fn run_cmd(cmd: &mut std::process::Command, context: &str) -> HudHudResult<Vec<u8>> {
    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("{}: command failed: {}", context, e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(runtime_error(format!("{}: {}", context, stderr)));
    }
    Ok(output.stdout)
}

pub fn file_size(path: &str) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    if let Some(start) = json.find(&pattern) {
        let after = &json[start + pattern.len()..];
        if let Some(end) = after.find('"') {
            return Some(after[..end].to_string());
        }
    }
    let pattern2 = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern2) {
        let after = &json[start + pattern2.len()..];
        if let Some(end) = after.find('"') {
            return Some(after[..end].to_string());
        }
    }
    None
}

pub fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    if let Some(start) = json.find(&pattern) {
        let after = &json[start + pattern.len()..];
        let trimmed = after.trim_start();
        let end = trimmed
            .find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())
            .unwrap_or(trimmed.len());
        let num_str = &trimmed[..end];
        return num_str.parse().ok();
    }
    None
}
