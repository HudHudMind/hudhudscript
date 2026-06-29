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

// ═══════════════════════════════════════════════════════════════════════════════
// hudhud.toml configuration types (Issues #446, #447, #448)
// ═══════════════════════════════════════════════════════════════════════════════

/// Top-level hudhud.toml configuration (subset we care about at runtime).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HudHudConfig {
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default, rename = "stream")]
    pub _stream: StreamConfig,
    #[serde(default, rename = "security")]
    pub _security: SecurityConfig,
    /// HOST-1: host access policy from hudhud.toml [host_access].
    #[serde(default)]
    pub host_access: Option<HostAccessConfig>,
    /// ENV0003: provider defaults from hudhud.toml [providers.NAME]
    /// Values use ${VAR} syntax for env interpolation.
    #[serde(default)]
    pub providers: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// BOLEM-A: lint configuration
    #[serde(default)]
    pub lint: LintConfig,
    /// MCP server definitions from hudhud.toml [mcp.servers.NAME]
    #[serde(default)]
    pub mcp: McpConfig,
    /// ISSUE-1: GC tuning parameters from hudhud.toml [gc].
    #[serde(default)]
    pub gc: GcConfig,
}

/// MCP server configuration (single server).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// MCP configuration section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: std::collections::HashMap<String, McpServerConfig>,
}

/// Shadowing severity policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedeclarePolicy {
    Allow,
    Warn,
    Error,
}

impl Default for RedeclarePolicy {
    fn default() -> Self { RedeclarePolicy::Warn }
}

/// [lint] section
#[derive(Debug, Clone, Deserialize)]
pub struct LintConfig {
    #[serde(default, rename = "redeclare")]
    pub redeclare: RedeclarePolicy,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self { redeclare: RedeclarePolicy::Warn }
    }
}

/// [runtime] section — interpreter / VM limits (Issue #446).
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum recursion / call-stack depth for the interpreter.
    #[serde(default = "default_max_recursion")]
    pub max_recursion: usize,
    /// VM register arena stack limit (in slots).
    #[serde(default = "default_stack_limit", rename = "stack_limit")]
    pub stack_limit: usize,
    /// Fuel/gas limit (0 = unlimited).
    #[serde(default)]
    pub fuel_limit: u64,
    /// Thread stack size in megabytes (default: 64).
    #[serde(default = "default_thread_stack_mb", rename = "thread_stack_mb")]
    pub thread_stack_mb: u32,
    /// Register arena initial size in KB (default: 64).
    #[serde(default = "default_register_arena_kb", rename = "register_arena_kb")]
    pub register_arena_kb: u32,
    /// Actor mailbox capacity (default: 128).
    #[serde(default = "default_mailbox_capacity", rename = "mailbox_capacity")]
    pub mailbox_capacity: usize,
    /// Max MCP servers (default: 128).
    #[serde(default = "default_max_mcp_servers", rename = "max_mcp_servers")]
    pub max_mcp_servers: usize,
    /// Execution timeout in milliseconds (0 = no timeout).
    #[serde(default = "default_execution_timeout_ms", rename = "execution_timeout_ms")]
    pub execution_timeout_ms: u64,
    /// Builtin iteration limit (default: 10_000).
    #[serde(default = "default_builtin_max_iter", rename = "builtin_max_iter")]
    pub builtin_max_iter: usize,
    /// Hard ceiling for max_call_depth (default: 4000).
    #[serde(default = "default_call_depth_ceiling", rename = "max_call_depth_hard_ceiling")]
    pub max_call_depth_hard_ceiling: usize,
    /// Non-Linux default stack bytes (default: 8MB).
    #[serde(default = "default_stack_bytes", rename = "default_stack_bytes")]
    pub default_stack_bytes: usize,
    /// Allow network access (default: false). Set to true for provider/LLM calls.
    #[serde(default)]
    pub allow_network: bool,
}

/// [gc] section — GC tuning parameters (Issue #1).
#[derive(Debug, Clone, Deserialize)]
pub struct GcConfig {
    /// Minimum number of allocated objects before a collection is considered.
    #[serde(default = "default_gc_min_objects", rename = "min_objects")]
    pub min_objects: usize,
    /// Growth factor applied to the GC threshold after each collection.
    #[serde(default = "default_gc_growth", rename = "growth_factor")]
    pub growth_factor: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            min_objects: hudhudscript_bytecode::gc::DEFAULT_GC_MIN_OBJECTS,
            growth_factor: hudhudscript_bytecode::gc::DEFAULT_GC_GROWTH,
        }
    }
}

fn default_gc_min_objects() -> usize {
    hudhudscript_bytecode::gc::DEFAULT_GC_MIN_OBJECTS
}

fn default_gc_growth() -> usize {
    hudhudscript_bytecode::gc::DEFAULT_GC_GROWTH
}

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
            if let Some(entry) = hudhudscript_errors::embedded_translations::localized_by_short_code(&code, &locale) {
                // ERR-1: Show localized title + full English body (was: title-only, lost all detail)
                let title = entry.title;
                return format!("{}: [{}] {}
{}", prefix, code, title, msg);
            }
        }
    }
    format!("{}: {}", prefix, msg)
}

fn locale_prefix(locale: &str) -> &str {
    match locale {
        "tr" => "Hata", "ar" => "خطأ", "ja" => "エラー", "ru" => "Ошибка", "zh" => "错误", _ => "Error",
    }
}

fn extract_error_code(msg: &str) -> Option<String> {
    let start = msg.find("[E")?;
    let rel_end = msg[start..].find(']')?;
    let code = &msg[start+1..start+rel_end];
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
