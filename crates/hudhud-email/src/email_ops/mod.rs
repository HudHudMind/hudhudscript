//! Shared email / messaging builtins — SMTP via msmtp/sendmail, MIME
//! parsing, Maildir listing, Telegram, webhook POST.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).

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

/// Main entry point used by the VM's module dispatcher.
/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Send,
    SendSimple,
    ParseMime,
    ListMaildir,
    TelegramSend,
    Webhook,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(Self::Send),
            "send_simple" => Ok(Self::SendSimple),
            "parse_mime" => Ok(Self::ParseMime),
            "list_maildir" => Ok(Self::ListMaildir),
            "telegram_send" => Ok(Self::TelegramSend),
            "webhook" => Ok(Self::Webhook),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Send => email_send(args),
        ScriptMethodId::SendSimple => email_send_simple(args),
        ScriptMethodId::ParseMime => email_parse_mime(args),
        ScriptMethodId::ListMaildir => email_list_maildir(args),
        ScriptMethodId::TelegramSend => email_telegram_send(args),
        ScriptMethodId::Webhook => email_webhook(args),
    }
}

mod messaging;
mod parse;
/// Main entry point (kept for backward compat).
mod send;
mod util;

pub use messaging::*;
pub use parse::*;
pub use send::*;
pub use util::*;
