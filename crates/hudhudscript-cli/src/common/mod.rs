//! Shared functions for HudHudScript CLI binaries
//!
//! This module contains common functionality used by hudhudscript, hudi, hudc, and hudp.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
#[cfg(feature = "mcp")]
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_runtime::{
    AnthropicProvider, OllamaProvider, OpenAICompatibleProvider, OpenAIProvider, ProviderConfig,
    ProviderRegistry, ProviderType,
};
pub use hudhudscript_ui_bridge::{create_bridge, Framework};
use hudhudscript_vm::{OutputLocale, VM};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// CLI Error type with structured exit codes (Issue #1003)
// ═══════════════════════════════════════════════════════════════════════════════

/// Structured CLI error with distinct exit codes.
///
/// - Exit 0: success (no error)
/// - Exit 1: runtime error
/// - Exit 2: parse/compile error
/// - Exit 3: file not found / IO error
#[derive(Debug)]
pub enum CliError {
    /// Runtime error (exit code 1)
    Runtime(String),
    /// Parse or compile error (exit code 2)
    ParseCompile(String),
    /// File not found or IO error (exit code 3)
    Io(String),
}

impl CliError {
    /// Return the exit code for this error category.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Runtime(_) => 1,
            CliError::ParseCompile(_) => 2,
            CliError::Io(_) => 3,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Runtime(msg) => write!(f, "{}", msg),
            CliError::ParseCompile(msg) => write!(f, "{}", msg),
            CliError::Io(msg) => write!(f, "{}", msg),
        }
    }
}

pub mod config_types;
pub mod config_include;
pub mod config_build;

pub use config_types::*;
pub use config_build::*;

mod compile;
mod config;
mod debug;
mod deploy;
mod format;
mod host_access;
mod locale;
pub(crate) mod provider;
mod repl;
mod run;
#[cfg(feature = "telemetry")]
pub mod telemetry_writer;
mod ui;

pub use compile::*;
pub use config::*;
pub use debug::*;
pub use deploy::*;
pub use format::*;
pub use host_access::*;
pub use locale::*;
pub use provider::*;
pub use repl::*;
pub use run::*;
pub use ui::*;

/// BOLEM-B Adım2: Locale-aware error rendering.
/// ERR-1 fix: Show localized title + FULL English details (no info loss).
pub fn render_error(e: &CliError) -> String {
    let locale = std::env::var("HUDHUD_LOCALE").unwrap_or_else(|_| "en".to_string());
    let msg = format!("{}", e);
    let prefix = locale_prefix(&locale);

    if locale != "en" {
        if let Some(code) = extract_error_code(&msg) {
            if let Some(entry) =
                hudhudscript_errors::embedded_translations::localized_by_short_code(&code, &locale)
            {
                // ERR-1: Show localized title + full English body (was: title-only, lost all detail)
                let title = entry.title;
                return format!(
                    "{}: [{}] {}
{}",
                    prefix, code, title, msg
                );
            }
        }
    }
    format!("{}: {}", prefix, msg)
}

fn locale_prefix(locale: &str) -> &str {
    match locale {
        "tr" => "Hata",
        "ar" => "خطأ",
        "ja" => "エラー",
        "ru" => "Ошибка",
        "zh" => "错误",
        _ => "Error",
    }
}

fn extract_error_code(msg: &str) -> Option<String> {
    let start = msg.find("[E")?;
    let rel_end = msg[start..].find(']')?;
    let code = &msg[start + 1..start + rel_end];
    if code.len() > 1 && code[1..].chars().all(|c| c.is_ascii_digit()) {
        Some(code.to_string())
    } else {
        None
    }
}

/// ERR-2: Locale-aware eprintln! replacement — uses translated "Error"/"Hata" etc.
pub fn eprint_error(msg: &str) {
    let locale = std::env::var("HUDHUD_LOCALE").unwrap_or_else(|_| "en".to_string());
    eprintln!("{}: {}", locale_prefix(&locale), msg);
}
