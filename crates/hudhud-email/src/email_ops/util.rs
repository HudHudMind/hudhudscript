//! Shared email / messaging builtins — SMTP via msmtp/sendmail, MIME
//! parsing, Maildir listing, Telegram, webhook POST.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).

use super::*;
use hudhudscript_bytecode::Value16;

pub fn send_via_sendmail(from: &str, to: &str, subject: &str, body: &str) -> HudHudResult<Value16> {
    let message = if from.is_empty() {
        format!("To: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}", to, subject, body)
    } else {
        format!("From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}", from, to, subject, body)
    };

    let mut cmd = std::process::Command::new("sendmail");
    cmd.arg("-t");
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        runtime_error(format!(
            "Failed to spawn sendmail: {}. Ensure sendmail or a compatible MTA is installed",
            e
        ))
    })?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        let _ = stdin.write_all(message.as_bytes());
    }

    let output = child
        .wait_with_output()
        .map_err(|e| runtime_error(format!("sendmail error: {}", e)))?;

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("ok".to_string(), Value16::bool_(output.status.success()));
    result.insert(
        "message".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stderr).to_string()),
    );
    Ok(Value16::object(result))
}

pub fn obj_str(obj: &hudhudscript_bytecode::ObjMap, key: &str, ctx: &str) -> HudHudResult<String> {
    match obj.get(key) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), &format!("{} {}", ctx, key))),
        None => Err(runtime_error(format!(
            "Missing required field '{}' in config object ({})",
            key, ctx
        ))),
    }
}

pub fn obj_str_opt(obj: &hudhudscript_bytecode::ObjMap, key: &str) -> Option<String> {
    obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

pub fn value_to_json_string(value: &Value16) -> String {
    if value.is_null() {
        return "null".to_string();
    }
    if let Some(b) = value.as_bool() {
        return b.to_string();
    }
    if let Some(n) = value.as_number() {
        return format_number(n);
    }
    if let Some(i) = value.as_int() {
        return format_number(i as f64);
    }
    if let Some(s) = value.as_str() {
        return format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    }
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr.iter().map(|v| value_to_json_string(v)).collect();
        return format!("[{}]", items.join(","));
    }
    if let Some(obj) = value.as_object() {
        let mut pairs: Vec<String> = obj
            .iter()
            .filter(|(k, _)| !k.to_string().starts_with("__"))
            .map(|(k, v)| format!("\"{}\":{}", k, value_to_json_string(v)))
            .collect();
        pairs.sort();
        return format!("{{{}}}", pairs.join(","));
    }
    format!("\"{}\"", value.display_string().replace('"', "\\\""))
}
