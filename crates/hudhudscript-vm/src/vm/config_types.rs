/// Output locale for formatting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputLocale {
    Default,
    Arabic,
}

/// F3: Object/dizi eşitlik politikası (hudhud.toml → [language] object_equality)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectEquality {
    /// Pointer kimliği: `a == a` true, `{x:1}=={x:1}` false. Varsayılan.
    Identity,
    /// Derin eşitlik: tüm alanlar rekürsif karşılaştırılır.
    Deep,
    /// Hiçbir nesne/dizi eşit değil (geriye uyum, önerilmez).
    Never,
}

/// Sandbox configuration for file, network, and process access control.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub allowed_paths: Vec<String>,
    pub allowed_hosts: Vec<String>,
    pub allow_file_read: bool,
    pub allow_file_write: bool,
    pub allow_network: bool,
    /// Allow spawning child processes (required for stdio MCP servers).
    pub allow_process: bool,
    /// MCP-40: Commands allowed for stdio MCP spawn. Empty = any (if allow_process).
    pub allowed_commands: Vec<String>,
    /// MCP-40: Commands denied regardless of allowed_commands.
    pub denied_commands: Vec<String>,
}
