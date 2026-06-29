//! HudHud encoding primitives (no builtins dependency).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Base64MethodId {
    Encode,
    Decode,
}

impl std::str::FromStr for Base64MethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "encode" => Ok(Self::Encode),
            "decode" => Ok(Self::Decode),
            _ => Err(runtime_error(format!("Unknown Base64 method: {}", s))),
        }
    }
}

impl Base64MethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Encode => base64_encode_args(args),
            Self::Decode => base64_decode_args(args),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexMethodId {
    Encode,
    Decode,
}

impl std::str::FromStr for HexMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "encode" => Ok(Self::Encode),
            "decode" => Ok(Self::Decode),
            _ => Err(runtime_error(format!("Unknown Hex method: {}", s))),
        }
    }
}

impl HexMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Encode => hex_encode_args(args),
            Self::Decode => hex_decode_args(args),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlMethodId {
    Encode,
    Decode,
}

impl std::str::FromStr for UrlMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "encode" => Ok(Self::Encode),
            "decode" => Ok(Self::Decode),
            _ => Err(runtime_error(format!("Unknown URL method: {}", s))),
        }
    }
}

impl UrlMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Encode => url_encode_args(args),
            Self::Decode => url_decode_args(args),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UuidMethodId {
    V4,
    V7,
    Nil,
    Parse,
}

impl std::str::FromStr for UuidMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v4" => Ok(Self::V4),
            "v7" => Ok(Self::V7),
            "nil" => Ok(Self::Nil),
            "parse" => Ok(Self::Parse),
            _ => Err(runtime_error(format!("Unknown UUID method: {}", s))),
        }
    }
}

impl UuidMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::V4 => uuid_v4_args(args),
            Self::V7 => uuid_v7_args(args),
            Self::Nil => uuid_nil_args(args),
            Self::Parse => uuid_parse_args(args),
        }
    }
}

// ── Low-level primitives (String/Vec<u8>) ──────────────────────────────

pub fn base64_encode(s: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

pub fn base64_decode(s: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s.as_bytes())
        .map_err(|e| format!("Base64.decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("Base64.decode error: {}", e))
}

pub fn hex_encode(s: &str) -> String {
    hex::encode(s.as_bytes())
}

pub fn hex_decode(s: &str) -> Result<String, String> {
    let bytes = hex::decode(s).map_err(|e| format!("Hex.decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("Hex.decode error: {}", e))
}

pub fn url_encode(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

pub fn url_decode(s: &str) -> Result<String, String> {
    urlencoding::decode(s)
        .map_err(|e| format!("URL.decode error: {}", e))
        .map(|cow| cow.into_owned())
}

pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn uuid_v7() -> String {
    uuid::Uuid::now_v7().to_string()
}

pub fn uuid_nil() -> String {
    uuid::Uuid::nil().to_string()
}

pub fn uuid_parse(s: &str) -> Result<uuid::Uuid, String> {
    uuid::Uuid::parse_str(s).map_err(|e| format!("uuid.parse error: {}", e))
}

// ── Value16 handlers (moved from builtins) ─────────────────────────────

pub fn base64_encode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Base64.encode() requires a string argument"))?;
    Ok(Value16::string(base64_encode(s)))
}

pub fn base64_decode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Base64.decode() requires a string argument"))?;
    let decoded =
        base64_decode(s).map_err(|e| runtime_error(format!("Base64.decode error: {}", e)))?;
    Ok(Value16::string(decoded))
}

pub fn hex_encode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Hex.encode() requires a string argument"))?;
    Ok(Value16::string(hex_encode(s)))
}

pub fn hex_decode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("Hex.decode() requires a string argument"))?;
    let decoded = hex_decode(s).map_err(|e| runtime_error(format!("Hex.decode error: {}", e)))?;
    Ok(Value16::string(decoded))
}

pub fn url_encode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("URL.encode() requires a string argument"))?;
    Ok(Value16::string(url_encode(s)))
}

pub fn url_decode_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("URL.decode() requires a string argument"))?;
    let decoded = url_decode(s).map_err(|e| runtime_error(format!("URL.decode error: {}", e)))?;
    Ok(Value16::string(decoded))
}

pub fn uuid_v4_args(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(uuid_v4()))
}

pub fn uuid_v7_args(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(uuid_v7()))
}

pub fn uuid_nil_args(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(uuid_nil()))
}

pub fn uuid_parse_args(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("uuid.parse() requires a string argument"))?;
    let parsed = uuid_parse(s).map_err(|e| runtime_error(format!("uuid.parse error: {}", e)))?;
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("value".to_string(), Value16::string(parsed.to_string()));
    obj.insert(
        "version".to_string(),
        Value16::number(parsed.get_version_num() as f64),
    );
    obj.insert(
        "variant".to_string(),
        Value16::string(format!("{:?}", parsed.get_variant())),
    );
    Ok(Value16::object(obj))
}
