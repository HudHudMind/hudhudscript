//! Shared email / messaging builtins — SMTP via msmtp/sendmail, MIME
//! parsing, Maildir listing, Telegram, webhook POST.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).

use super::*;
use hudhudscript_bytecode::Value16;

pub fn email_telegram_send(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 3 {
        return Err(runtime_error(
            "email.telegram_send() requires 3 arguments: bot_token, chat_id, text",
        ));
    }
    let bot_token = args[0]
        .as_str()
        .ok_or_else(|| {
            type_error(
                "string",
                args[0].type_name_str(),
                "email.telegram_send bot_token",
            )
        })?
        .to_string();
    let chat_id = args[1]
        .as_str()
        .ok_or_else(|| {
            type_error(
                "string",
                args[1].type_name_str(),
                "email.telegram_send chat_id",
            )
        })?
        .to_string();
    let text = args[2]
        .as_str()
        .ok_or_else(|| {
            type_error(
                "string",
                args[2].type_name_str(),
                "email.telegram_send text",
            )
        })?
        .to_string();

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let mut payload = serde_json::Map::new();
    payload.insert("chat_id".to_string(), serde_json::Value::String(chat_id));
    payload.insert("text".to_string(), serde_json::Value::String(text));

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| runtime_error(format!("HTTP client error: {}", e)))?;

    let resp = client
        .post(&url)
        .json(&serde_json::Value::Object(payload))
        .send()
        .map_err(|e| runtime_error(format!("Telegram API error: {}", e)))?;

    let status = resp.status().as_u16();
    let body_text = resp
        .text()
        .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value16::number(status as f64));
    result.insert(
        "ok".to_string(),
        Value16::bool_((200..300).contains(&status)),
    );
    result.insert("body".to_string(), Value16::string(body_text));
    Ok(Value16::object(result))
}

pub fn email_webhook(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "email.webhook() requires 2 arguments: url, payload_object",
        ));
    }
    let url = args[0]
        .as_str()
        .ok_or_else(|| type_error("string", args[0].type_name_str(), "email.webhook url"))?
        .to_string();

    let json_payload = {
        let json_str = value_to_json_string(&args[1]);
        serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| runtime_error(format!("HTTP client error: {}", e)))?;

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json_payload)
        .send()
        .map_err(|e| runtime_error(format!("Webhook error: {}", e)))?;

    let status = resp.status().as_u16();
    let body_text = resp
        .text()
        .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));

    let mut result = HashMap::new();
    result.insert("status".to_string(), Value16::number(status as f64));
    result.insert(
        "ok".to_string(),
        Value16::bool_((200..300).contains(&status)),
    );
    result.insert("body".to_string(), Value16::string(body_text));
    Ok(Value16::object(result))
}

// ── helpers ────────────────────────────────────────────────────────────────
