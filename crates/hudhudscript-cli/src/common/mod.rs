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
    /// ENV0003: provider defaults from hudhud.toml [providers.NAME]
    /// Values use ${VAR} syntax for env interpolation.
    #[serde(default)]
    pub providers: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
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

mod compile;
mod config;
mod debug;
mod deploy;
mod format;
mod locale;
mod provider;
mod repl;
mod run;
mod ui;

pub use compile::*;
pub use config::*;
pub use debug::*;
pub use deploy::*;
pub use format::*;
pub use locale::*;
pub use provider::*;
pub use repl::*;
pub use run::*;
pub use ui::*;
