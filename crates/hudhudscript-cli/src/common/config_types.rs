use serde::Deserialize;
use std::collections::HashMap;

/// Top-level hudhud.toml configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HudHudConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default, rename = "stream")]
    pub _stream: StreamConfig,
    #[serde(default, rename = "security")]
    pub _security: SecurityConfig,
    #[serde(default)]
    pub host_access: Option<super::HostAccessConfig>,
    #[serde(default)]
    pub providers: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub lint: LintConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub gc: GcConfig,
    #[serde(default)]
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub target: Option<TargetConfig>,
    #[serde(default)]
    pub optimization: Option<OptimizationConfig>,
    #[serde(default)]
    pub link: Option<LinkConfig>,
    #[serde(default)]
    pub profile: HashMap<String, ProfileConfig>,
}

/// [runtime] section — runtime execution and sandbox permissions.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_max_recursion", rename = "max_recursion")]
    pub max_recursion: usize,
    #[serde(default = "default_stack_limit", rename = "stack_limit")]
    pub stack_limit: usize,
    #[serde(default)]
    pub fuel_limit: u64,
    #[serde(default = "default_thread_stack_mb", rename = "thread_stack_mb")]
    pub thread_stack_mb: u32,
    #[serde(default = "default_register_arena_kb", rename = "register_arena_kb")]
    pub register_arena_kb: u32,
    #[serde(default = "default_mailbox_capacity", rename = "mailbox_capacity")]
    pub mailbox_capacity: usize,
    #[serde(default = "default_max_mcp_servers", rename = "max_mcp_servers")]
    pub max_mcp_servers: usize,
    #[serde(default = "default_execution_timeout_ms", rename = "execution_timeout_ms")]
    pub execution_timeout_ms: u64,
    #[serde(default = "default_builtin_max_iter", rename = "builtin_max_iter")]
    pub builtin_max_iter: usize,
    #[serde(default = "default_call_depth_ceiling", rename = "max_call_depth_hard_ceiling")]
    pub max_call_depth_hard_ceiling: usize,
    #[serde(default = "default_stack_bytes", rename = "default_stack_bytes")]
    pub default_stack_bytes: usize,
    #[serde(default)]
    pub allow_network: bool,
    #[serde(default)]
    pub allow_process: bool,
    #[serde(default)]
    pub allow_insecure_http: bool,
    #[serde(default)]
    pub allow_privileged: bool,
    #[serde(default = "default_provider_timeout_secs", rename = "provider_timeout_secs")]
    pub provider_timeout_secs: u64,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub vm: Option<RuntimeVmConfig>,
    #[serde(default)]
    pub jit: Option<RuntimeJitConfig>,
}

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
            allow_process: false,
            allow_insecure_http: false,
            allow_privileged: false,
            provider_timeout_secs: hudhudscript_runtime::provider::DEFAULT_PROVIDER_TIMEOUT_SECS,
            engine: None,
            backend: None,
            vm: None,
            jit: None,
        }
    }
}

/// [runtime.vm] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeVmConfig {
    #[serde(default)]
    pub max_call_depth: Option<usize>,
    #[serde(default)]
    pub stack_limit: Option<usize>,
    #[serde(default)]
    pub fuel_limit: Option<u64>,
}

/// [runtime.jit] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeJitConfig {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default)]
    pub opt_level: Option<String>,
    #[serde(default)]
    pub opt_goal: Option<String>,
    #[serde(default)]
    pub mir_opt_rounds: Option<usize>,
    #[serde(default)]
    pub policy: Option<String>,
    #[serde(default)]
    pub hot_threshold: Option<usize>,
    #[serde(default)]
    pub loop_threshold: Option<usize>,
    #[serde(default)]
    pub stats: Option<bool>,
    #[serde(default)]
    pub verify_with_vm: Option<bool>,
    #[serde(default)]
    pub cache: Option<bool>,
    #[serde(default)]
    pub code_cache_mb: Option<usize>,
}

/// [build] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub aot: Option<BuildAotConfig>,
}

/// [build.aot] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildAotConfig {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

/// [target] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TargetConfig {
    #[serde(default)]
    pub triple: Option<String>,
    #[serde(default)]
    pub cpu: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

/// [optimization] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OptimizationConfig {
    #[serde(default)]
    pub level: Option<u8>,
    #[serde(default)]
    pub goal: Option<String>,
}

/// [link] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LinkConfig {
    #[serde(default)]
    pub runtime: Option<String>,
}

/// [profile.NAME] section (JIT_AOT_ARCHITECTURE §20.3)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub build: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub optimization: Option<toml::Value>,
    #[serde(default)]
    pub target: Option<String>,
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

/// MCP server configuration (single server).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// MCP configuration section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
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
    fn default() -> Self {
        RedeclarePolicy::Warn
    }
}

/// [lint] section
#[derive(Debug, Clone, Deserialize)]
pub struct LintConfig {
    #[serde(default, rename = "redeclare")]
    pub redeclare: RedeclarePolicy,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            redeclare: RedeclarePolicy::Warn,
        }
    }
}

/// [gc] section — GC tuning parameters (Issue #1).
#[derive(Debug, Clone, Deserialize)]
pub struct GcConfig {
    #[serde(default = "default_gc_min_objects", rename = "min_objects")]
    pub min_objects: usize,
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
pub fn default_provider_timeout_secs() -> u64 {
    hudhudscript_runtime::provider::DEFAULT_PROVIDER_TIMEOUT_SECS
}
fn default_chunk_size() -> usize { 1024 }
fn default_timeout() -> u64 { 30_000 }
fn default_max_tokens() -> usize { 4096 }
fn default_buffer_size() -> usize { 8192 }
fn default_gc_min_objects() -> usize {
    hudhudscript_bytecode::gc::DEFAULT_GC_MIN_OBJECTS
}
fn default_gc_growth() -> usize {
    hudhudscript_bytecode::gc::DEFAULT_GC_GROWTH
}
