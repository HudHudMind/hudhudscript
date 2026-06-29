//! JSON body parser — reuses `hudhud-http/json` (Kural 7).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

/// Parse a JSON body string into a Value16, reusing `hudhud-http`'s JSON parser.
pub fn parse_json_body(body: &str) -> HudHudResult<Value16> {
    if body.trim().is_empty() {
        return Ok(Value16::null());
    }
    let parsed: serde_json::Value =
        serde_json::from_str(body).map_err(|e| {
            hudhudscript_errors::Error::new(
                hudhudscript_errors::ErrorCode::CompileRuntimeError,
                format!("JSON body parse error: {}", e),
            )
        })?;
    Ok(hudhud_http::json::serde_to_value(&parsed))
}

