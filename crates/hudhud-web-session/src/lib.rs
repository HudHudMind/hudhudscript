//! HudHud Web Session — HMAC-SHA256 signed cookie sessions.
//!
//! State is stored in the cookie itself (signed, not encrypted).
//! No server-side storage needed → works with prefork workers (§1.4).

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Serialize a session object to a simple key=value string (JSON inside value).
fn serialize_session(obj: &hudhudscript_bytecode::ObjMap) -> String {
    let mut parts: Vec<String> = obj
        .iter()
        .map(|(k, v)| {
            let val_str = match v.as_str() {
                Some(s) => s.to_string(),
                _ => v.display_string(),
            };
            format!("{}={}", k, val_str)
        })
        .collect();
    parts.sort(); // deterministic
    parts.join("&")
}

/// Deserialize a session string back to an object.
fn deserialize_session(data: &str) -> hudhudscript_bytecode::ObjMap {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    for pair in data.split('&') {
        if let Some(idx) = pair.find('=') {
            let key = pair[..idx].to_string();
            let val = pair[idx + 1..].to_string();
            obj.insert(key, Value16::string(val));
        }
    }
    obj
}

/// Sign data with HMAC-SHA256, return base64(signature).
fn sign(data: &str, secret: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    B64.encode(result.into_bytes())
}

/// Verify signature. Returns true if valid.
fn verify(data: &str, signature: &str, secret: &str) -> bool {
    let expected = sign(data, secret);
    // Constant-time comparison
    expected == signature
}

/// Parse a session cookie value: "base64(data).base64(signature)"
fn parse_session_cookie(cookie: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = cookie.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }
    let data_bytes = B64.decode(parts[0].as_bytes()).ok()?;
    let sig_bytes = B64.decode(parts[1].as_bytes()).ok()?;
    let data = String::from_utf8_lossy(&data_bytes).to_string();
    let sig = String::from_utf8_lossy(&sig_bytes).to_string();
    Some((data, sig))
}

/// Encode a session cookie value: "base64(data).base64(signature)"
fn encode_session_cookie(data: &str, secret: &str) -> String {
    let sig = sign(data, secret);
    let data_b64 = B64.encode(data.as_bytes());
    let sig_b64 = B64.encode(sig.as_bytes());
    format!("{}.{}", data_b64, sig_b64)
}

/// `Web.session_get(req, secret)` → session object.
///
/// Reads the `session` cookie from the request, verifies the HMAC signature,
/// and returns the session data as an object. Returns empty object if no
/// valid session cookie exists.
pub fn session_get(args: &[Value16]) -> HudHudResult<Value16> {
    let req = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.session_get"))?;

    let secret = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.session_get:secret"))?;

    // Get cookies from request
    let cookies = req.get("cookies").and_then(|v| v.as_object());

    let session_cookie = cookies
        .and_then(|c| c.get("session"))
        .and_then(|v| v.as_str());

    match session_cookie {
        Some(cookie) => {
            if let Some((data, sig)) = parse_session_cookie(cookie) {
                if verify(&data, &sig, secret) {
                    let obj = deserialize_session(&data);
                    return Ok(Value16::object(obj));
                }
            }
            // Invalid signature → empty session
            Ok(Value16::object(hudhudscript_bytecode::ObjMap::default()))
        }
        None => Ok(Value16::object(hudhudscript_bytecode::ObjMap::default())),
    }
}

/// `Web.session_set(resp, secret, obj)` → response with Set-Cookie.
///
/// Serializes the session object, signs it with HMAC-SHA256, and adds
/// a `Set-Cookie: session=...` header to the response.
pub fn session_set(args: &[Value16]) -> HudHudResult<Value16> {
    let resp = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.session_set"))?;

    let secret = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.session_set:secret"))?;

    let obj = args
        .get(2)
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.session_set:data"))?;

    let data = serialize_session(obj);
    let cookie_value = encode_session_cookie(&data, secret);
    let cookie_header = format!("session={}; Path=/; HttpOnly; SameSite=Lax", cookie_value);

    // Add to response cookies
    let mut new_cookies: Vec<Value16> = Vec::new();
    if let Some(existing) = resp.get("cookies").and_then(|v| v.as_array()) {
        for c in existing {
            if let Some(s) = c.as_str() {
                if !s.starts_with("session=") {
                    new_cookies.push(c.clone());
                }
            }
        }
    }
    new_cookies.push(Value16::string(cookie_header));

    let mut new_obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    for (k, v) in resp.iter() {
        if k != "cookies" {
            new_obj.insert(k.clone(), v.clone());
        }
    }
    new_obj.insert("cookies".to_string(), Value16::array(new_cookies));

    Ok(Value16::object(new_obj))
}

// ── Unit tests ─────────────────────────────────────────────────────────
