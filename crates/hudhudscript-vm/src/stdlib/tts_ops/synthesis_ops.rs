use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct TtsOptions {
    pub voice: Option<String>,
    pub speed: Option<i64>,
    pub pitch: Option<i64>,
    pub volume: Option<i64>,
}

pub(crate) fn extract_options(val: &Value16) -> TtsOptions {
    let obj = match val.as_object() {
        Some(o) => o,
        None => {
            return TtsOptions {
                voice: None,
                speed: None,
                pitch: None,
                volume: None,
            }
        }
    };

    let voice = obj
        .get("voice")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let speed = obj
        .get("speed")
        .and_then(|v| v.as_number())
        .map(|n| n as i64);
    let pitch = obj
        .get("pitch")
        .and_then(|v| v.as_number())
        .map(|n| n as i64);
    let volume = obj
        .get("volume")
        .and_then(|v| v.as_number())
        .map(|n| n as i64);

    TtsOptions {
        voice,
        speed,
        pitch,
        volume,
    }
}

pub(crate) fn empty_opts() -> TtsOptions {
    TtsOptions {
        voice: None,
        speed: None,
        pitch: None,
        volume: None,
    }
}

pub(crate) fn espeak_speak_args(text: &str, opts: &TtsOptions) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(ref v) = opts.voice {
        args.push("-v".to_string());
        args.push(v.clone());
    }
    if let Some(s) = opts.speed {
        args.push("-s".to_string());
        args.push(s.to_string());
    }
    if let Some(p) = opts.pitch {
        args.push("-p".to_string());
        args.push(p.to_string());
    }
    if let Some(a) = opts.volume {
        args.push("-a".to_string());
        args.push(a.to_string());
    }
    args.push(text.to_string());
    args
}

pub(crate) fn espeak_save_args(text: &str, output: &str, opts: &TtsOptions) -> Vec<String> {
    let mut args = vec!["-w".to_string(), output.to_string()];
    if let Some(ref v) = opts.voice {
        args.push("-v".to_string());
        args.push(v.clone());
    }
    if let Some(s) = opts.speed {
        args.push("-s".to_string());
        args.push(s.to_string());
    }
    if let Some(p) = opts.pitch {
        args.push("-p".to_string());
        args.push(p.to_string());
    }
    if let Some(a) = opts.volume {
        args.push("-a".to_string());
        args.push(a.to_string());
    }
    args.push(text.to_string());
    args
}

pub(crate) fn run_command(bin: &str, args: &[String]) -> SharedResult<Value16> {
    let result = Command::new(bin).args(args).output();
    match result {
        Ok(output) => {
            let mut obj = HashMap::new();
            let success = output.status.success();
            obj.insert("ok".to_string(), Value16::boolean(success));
            obj.insert(
                "code".to_string(),
                Value16::number(output.status.code().unwrap_or(-1) as f64),
            );
            if !success {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                obj.insert("error".to_string(), Value16::string(stderr));
            }
            Ok(Value16::object(obj))
        }
        Err(e) => {
            let mut obj = HashMap::new();
            obj.insert("ok".to_string(), Value16::boolean(false));
            obj.insert("code".to_string(), Value16::number(-1.0));
            obj.insert(
                "error".to_string(),
                Value16::string(format!("Failed to execute {}: {}", bin, e)),
            );
            Ok(Value16::object(obj))
        }
    }
}

pub(crate) fn error_obj(msg: String) -> Value16 {
    let mut obj = HashMap::new();
    obj.insert("ok".to_string(), Value16::boolean(false));
    obj.insert("code".to_string(), Value16::number(-1.0));
    obj.insert("error".to_string(), Value16::string(msg));
    Value16::object(obj)
}

pub(crate) fn require_string(val: &Value16, ctx: &str) -> SharedResult<String> {
    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| type_error("string", val.type_name_str(), ctx))
}
