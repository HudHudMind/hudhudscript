use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Package type enumeration
// ─────────────────────────────────────────────────────────────────────────────

/// The kind of package: application (has an entry point) or library (exposes
/// modules for import by other packages).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum PackageType {
    #[default]
    Application,
    Library,
}

// ─────────────────────────────────────────────────────────────────────────────
// Top-level configuration
// ─────────────────────────────────────────────────────────────────────────────

/// HudHud project configuration (hudhud.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudhudConfig {
    pub package: PackageConfig,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, DependencySpec>,
    #[serde(default, rename = "native-dependencies")]
    pub native_dependencies: HashMap<String, NativeDependencySpec>,
    #[serde(default, rename = "mcp-servers")]
    pub mcp_servers: HashMap<String, McpServerSpec>,
    #[serde(default, rename = "ai-providers")]
    pub ai_providers: HashMap<String, AiProviderConfig>,
    #[serde(default)]
    pub registry_url: String,
    #[serde(default)]
    pub cache_dir: PathBuf,
}

impl Default for HudhudConfig {
    fn default() -> Self {
        Self {
            package: PackageConfig::default(),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            native_dependencies: HashMap::new(),
            mcp_servers: HashMap::new(),
            ai_providers: HashMap::new(),
            registry_url: crate::DEFAULT_REGISTRY.to_string(),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("hudhudscript"),
        }
    }
}

impl HudhudConfig {
    /// Load from file
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from a `Path` reference (convenience wrapper)
    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to file
    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Whether this config describes an application package (has entry point).
    pub fn is_application(&self) -> bool {
        self.package.package_type == PackageType::Application
    }

    /// Whether this config describes a library package.
    pub fn is_library(&self) -> bool {
        self.package.package_type == PackageType::Library
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Package metadata
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    /// Package type – `"application"` (default) or `"library"`.
    #[serde(default, rename = "type")]
    pub package_type: PackageType,
    /// Entry point file, only meaningful for application packages.
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            name: "my-project".to_string(),
            version: "0.1.0".to_string(),
            package_type: PackageType::Application,
            entry: None,
            description: String::new(),
            authors: vec![],
            license: "MIT".to_string(),
            repository: String::new(),
            homepage: String::new(),
            keywords: vec![],
            categories: vec![],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dependency specifications
// ─────────────────────────────────────────────────────────────────────────────

/// Dependency specification – either a simple version string or a detailed
/// table with optional features, git source, path, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string, e.g. `"^1.0"`
    Simple(String),
    /// Detailed dependency with extra fields.
    Detailed {
        version: String,
        #[serde(default)]
        features: Vec<String>,
        #[serde(default)]
        registry: Option<String>,
        #[serde(default)]
        git: Option<String>,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default)]
        tag: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        optional: bool,
    },
}

impl DependencySpec {
    /// Return the version string regardless of variant.
    pub fn version(&self) -> &str {
        match self {
            Self::Simple(v) => v,
            Self::Detailed { version, .. } => version,
        }
    }
}

/// Native (non-HudHudScript) dependency specification, e.g. a C/C++ library
/// built with cmake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeDependencySpec {
    /// Path to the native project directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Build system type: `"cmake"`, `"make"`, `"cargo"`, etc.
    #[serde(default, rename = "type")]
    pub build_type: Option<String>,
    /// Version constraint (when fetched from a registry).
    #[serde(default)]
    pub version: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP server specification
// ─────────────────────────────────────────────────────────────────────────────

/// MCP server entry in the manifest.  Accepts either a bare version string
/// (`github = "^1.0"`) or a table with extra fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerSpec {
    /// Bare version string.
    Simple(String),
    /// Full specification.
    Detailed {
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        server: Option<String>,
        #[serde(default)]
        registry: Option<String>,
        #[serde(default)]
        config: HashMap<String, serde_json::Value>,
        #[serde(default)]
        disabled: bool,
        #[serde(default)]
        auto_approve: Vec<String>,
    },
}

impl McpServerSpec {
    /// Return the version string if present.
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::Simple(v) => Some(v),
            Self::Detailed { version, .. } => version.as_deref(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AI provider configuration
// ─────────────────────────────────────────────────────────────────────────────

/// AI provider entry.  Accepts either a simple table with just `model` or a
/// richer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    /// Name of the environment variable holding the API key.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Inline API key (discouraged; prefer `api_key_env`).
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}
