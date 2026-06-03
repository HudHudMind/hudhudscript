use super::RuntimeConfig;
use crate::common::{CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_vm::{OutputLocale, VM};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn default_max_recursion() -> usize {
    hudhudscript_errors::constants::MAX_CALL_DEPTH
}
pub fn default_stack_limit() -> usize {
    hudhudscript_errors::constants::MAX_STACK_SIZE
}
pub fn default_thread_stack_mb() -> u32 { 64 }
pub fn default_register_arena_kb() -> u32 { 64 }
pub fn default_mailbox_capacity() -> usize { 128 }
pub fn default_max_mcp_servers() -> usize { 128 }
pub fn default_execution_timeout_ms() -> u64 { 0 }
pub fn default_builtin_max_iter() -> usize { 10_000 }
pub fn default_call_depth_ceiling() -> usize { 4000 }
pub fn default_stack_bytes() -> usize { 8 * 1024 * 1024 }

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_recursion: default_max_recursion(),
            stack_limit: default_stack_limit(),
            fuel_limit: 0,
            thread_stack_mb: default_thread_stack_mb(),
            register_arena_kb: default_register_arena_kb(),
            mailbox_capacity: default_mailbox_capacity(),
            max_mcp_servers: default_max_mcp_servers(),
            execution_timeout_ms: default_execution_timeout_ms(),
            builtin_max_iter: default_builtin_max_iter(),
            max_call_depth_hard_ceiling: default_call_depth_ceiling(),
            default_stack_bytes: default_stack_bytes(),
            allow_network: false,
        }
    }
}

/// [stream] section — streaming API configuration (Issue #447).
#[derive(Debug, Clone, Deserialize)]
pub struct StreamConfig {
    #[serde(default = "default_chunk_size", rename = "chunk_size")]
    pub _chunk_size: usize,
    #[serde(default = "default_timeout", rename = "timeout")]
    pub _timeout: u64,
    #[serde(default = "default_max_tokens", rename = "max_tokens")]
    pub _max_tokens: usize,
    #[serde(default = "default_buffer_size", rename = "buffer_size")]
    pub _buffer_size: usize,
}

fn default_chunk_size() -> usize {
    1024
}
fn default_timeout() -> u64 {
    30_000
}
fn default_max_tokens() -> usize {
    4096
}
fn default_buffer_size() -> usize {
    8192
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            _chunk_size: default_chunk_size(),
            _timeout: default_timeout(),
            _max_tokens: default_max_tokens(),
            _buffer_size: default_buffer_size(),
        }
    }
}

/// [security] section — sandbox and command classification (Issue #448).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecurityConfig {
    #[serde(default, rename = "sandbox")]
    pub _sandbox: bool,
    #[serde(default, rename = "commands")]
    pub _commands: CommandsConfig,
}

/// [security.commands] section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CommandsConfig {
    #[serde(default, rename = "safe")]
    pub _safe: Vec<String>,
    #[serde(default, rename = "ask")]
    pub _ask: Vec<String>,
    #[serde(default, rename = "dangerous")]
    pub _dangerous: Vec<String>,
    #[serde(default, rename = "blocked")]
    pub _blocked: Vec<String>,
}

/// Load hudhud.toml with 3-layer config resolution (#798):
///
/// 1. System global: /etc/hudhud/script/hudhud.toml (Linux)
///    /Library/Application Support/hudhud/script/hudhud.toml (macOS)
/// 2. User global: ~/.config/hudhud/script/hudhud.toml (XDG)
/// 3. Project local: ./hudhud.toml (walks up from cwd)
///
/// Each layer overrides the previous. Missing layers are skipped.
/// Load config using the 3-layer resolution (backward-compatible wrapper).
#[allow(dead_code)]
pub fn load_hudhud_config(debug: bool) -> HudHudConfig {
    load_hudhud_config_with_path(debug, None)
}

/// Load hudhud.toml with optional explicit path override (Issue #1006).
///
/// When `explicit_path` is provided (via `--config` CLI flag), it is loaded as
/// the highest-priority layer, overriding all other config sources.
pub fn load_hudhud_config_with_path(
    debug: bool,
    explicit_path: Option<&std::path::Path>,
) -> HudHudConfig {
    let mut config = HudHudConfig::default();

    // Layer 1: System global
    let system_paths: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![PathBuf::from(
            "/Library/Application Support/hudhud/script/hudhud.toml",
        )]
    } else {
        vec![PathBuf::from("/etc/hudhud/script/hudhud.toml")]
    };
    for path in &system_paths {
        if let Some(loaded) = try_load_config(path, debug) {
            config = merge_config(config, loaded);
        }
    }

    // Layer 2: User global (XDG_CONFIG_HOME or ~/.config)
    let user_config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs_fallback_home().join(".config"));
    let user_path = user_config_dir.join("hudhud/script/hudhud.toml");
    if let Some(loaded) = try_load_config(&user_path, debug) {
        config = merge_config(config, loaded);
    }

    // Layer 3: Project local (walk up from cwd)
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = start.as_path();
    loop {
        let candidate = dir.join("hudhud.toml");
        if candidate.is_file() {
            if let Some(loaded) = try_load_config(&candidate, debug) {
                config = merge_config(config, loaded);
            }
            break;
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Layer 4: Explicit --config flag (highest priority, Issue #1006)
    if let Some(explicit) = explicit_path {
        if let Some(loaded) = try_load_config(explicit, debug) {
            if debug {
                eprintln!("[config] Using explicit config: {}", explicit.display());
            }
            config = merge_config(config, loaded);
        } else {
            eprintln!(
                "Warning: --config file not found or invalid: {}",
                explicit.display()
            );
        }
    }

    config
}

/// Try to load a config file, returning None if not found or invalid.
fn try_load_config(path: &std::path::Path, debug: bool) -> Option<HudHudConfig> {
    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<HudHudConfig>(&content) {
            Ok(cfg) => {
                if debug {
                    eprintln!("[config] Loaded: {}", path.display());
                }
                Some(cfg)
            }
            Err(e) => {
                if debug {
                    eprintln!("[config] Parse error in {}: {}", path.display(), e);
                }
                None
            }
        },
        Err(_) => None,
    }
}

/// Merge two configs: values from `overlay` override `base`.
/// Only non-default values in overlay take effect.
fn merge_config(base: HudHudConfig, overlay: HudHudConfig) -> HudHudConfig {
    HudHudConfig {
        runtime: RuntimeConfig {
            max_recursion: if overlay.runtime.max_recursion != default_max_recursion() {
                overlay.runtime.max_recursion
            } else {
                base.runtime.max_recursion
            },
            stack_limit: if overlay.runtime.stack_limit != default_stack_limit() {
                overlay.runtime.stack_limit
            } else {
                base.runtime.stack_limit
            },
            fuel_limit: if overlay.runtime.fuel_limit != 0 {
                overlay.runtime.fuel_limit
            } else {
                base.runtime.fuel_limit
            },
            thread_stack_mb: if overlay.runtime.thread_stack_mb != default_thread_stack_mb() {
                overlay.runtime.thread_stack_mb
            } else {
                base.runtime.thread_stack_mb
            },
            register_arena_kb: overlay.runtime.register_arena_kb,
            mailbox_capacity: overlay.runtime.mailbox_capacity,
            max_mcp_servers: overlay.runtime.max_mcp_servers,
            execution_timeout_ms: overlay.runtime.execution_timeout_ms,
            builtin_max_iter: overlay.runtime.builtin_max_iter,
            max_call_depth_hard_ceiling: overlay.runtime.max_call_depth_hard_ceiling,
            default_stack_bytes: overlay.runtime.default_stack_bytes,
            allow_network: overlay.runtime.allow_network || base.runtime.allow_network,
        },
        _stream: base._stream, // stream config from first found
        _security: base._security,
        providers: overlay.providers, // overlay wins (project > user > system)
        lint: overlay.lint,
    }
}

/// Fallback home directory when dirs crate is not available.
fn dirs_fallback_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
