//! Shared URL parser builtin — used by both VM and interpreter.
//!
//! Provides `URLParser.parse()` and `URLParser.format()`.

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

/// Enum identifying each URLParser operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlParserMethodId {
    Parse,
    Format,
}

impl std::str::FromStr for UrlParserMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "format" => Ok(Self::Format),
            _ => Err(runtime_error(format!("Unknown URLParser method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for URLParser operations.
pub fn dispatch(method: UrlParserMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        UrlParserMethodId::Parse => {
            let s = args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error("URLParser.parse: expected string argument"))?;

            let parsed = url::Url::parse(s)
                .map_err(|e| runtime_error(format!("URLParser.parse error: {}", e)))?;

            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert(
                "scheme".to_string(),
                Value16::string(parsed.scheme().to_string()),
            );
            obj.insert(
                "host".to_string(),
                match parsed.host_str() {
                    Some(h) => Value16::string(h.to_string()),
                    None => Value16::null(),
                },
            );
            obj.insert(
                "port".to_string(),
                match parsed.port() {
                    Some(p) => Value16::number(p as f64),
                    None => Value16::null(),
                },
            );
            obj.insert(
                "path".to_string(),
                Value16::string(parsed.path().to_string()),
            );
            obj.insert(
                "query".to_string(),
                match parsed.query() {
                    Some(q) => Value16::string(q.to_string()),
                    None => Value16::null(),
                },
            );
            obj.insert(
                "fragment".to_string(),
                match parsed.fragment() {
                    Some(f) => Value16::string(f.to_string()),
                    None => Value16::null(),
                },
            );
            obj.insert(
                "username".to_string(),
                if parsed.username().is_empty() {
                    Value16::null()
                } else {
                    Value16::string(parsed.username().to_string())
                },
            );
            obj.insert(
                "password".to_string(),
                match parsed.password() {
                    Some(p) => Value16::string(p.to_string()),
                    None => Value16::null(),
                },
            );
            obj.insert("href".to_string(), Value16::string(parsed.to_string()));
            Ok(Value16::object(obj))
        }
        UrlParserMethodId::Format => {
            let obj = args
                .first()
                .and_then(|v| v.as_object())
                .ok_or_else(|| runtime_error("URLParser.format() requires an object argument"))?;

            // If href exists, just return it
            if let Some(href_val) = obj.get("href") {
                if let Some(href) = href_val.as_str() {
                    return Ok(Value16::string(href.to_string()));
                }
            }

            let scheme = obj
                .get("scheme")
                .and_then(|v| v.as_str())
                .unwrap_or("https");
            let host = obj
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("localhost");
            let port_str = obj
                .get("port")
                .and_then(|v| v.as_number())
                .map(|n| format!(":{}", n as u16))
                .unwrap_or_default();
            let path = obj.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            let query_str = obj
                .get("query")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| format!("?{}", s))
                .unwrap_or_default();
            let fragment_str = obj
                .get("fragment")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| format!("#{}", s))
                .unwrap_or_default();

            Ok(Value16::string(format!(
                "{}://{}{}{}{}{}",
                scheme, host, port_str, path, query_str, fragment_str
            )))
        }
    }
}
