//! HudHudScript Deploy Core — deployment IR, pipeline definitions, and adapter trait
//!
//! Defines the intermediate representation for deployment:
//! - Deploy targets (web, desktop, mobile, wasm)
//! - CI/CD pipeline definitions
//! - Adapter trait for GitHub, GitLab, Jenkins, Docker, Kubernetes, etc.
//! - Custom deploy adapter support
//! - `.deb` package generation
//! - systemd service/timer file generation
//! - Bundle `.hud` scripts with runtime for distribution

pub mod adapters;
pub mod bundle;
pub mod deb;
pub mod systemd;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Deploy Target ───────────────────────────────────────────────────

/// Deployment target platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPlatform {
    Web,
    Desktop,
    Mobile,
    Wasm,
    Custom(String),
}

/// Target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub platform: TargetPlatform,
    pub framework: String,
    pub config: HashMap<String, String>,
}

// ── CI/CD Pipeline ──────────────────────────────────────────────────

/// CI/CD provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CIProvider {
    GitHub,
    GitLab,
    Jenkins,
    Custom(String),
}

/// Pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub provider: CIProvider,
    pub config: HashMap<String, String>,
    pub triggers: Vec<Trigger>,
    pub steps: Vec<PipelineStep>,
}

/// Pipeline trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub event: String, // "push", "pull_request", "schedule"
    pub branch: Option<String>,
    pub cron: Option<String>,
}

/// Single pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub action: StepAction,
}

/// Pipeline step actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepAction {
    Build { target: Target },
    Test,
    Deploy { target: Target, host: String },
    Docker { image: String, registry: String },
    Custom { command: String },
}

// ── Deploy Plan ─────────────────────────────────────────────────────

/// Complete deployment plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployPlan {
    pub app_name: String,
    pub targets: Vec<Target>,
    pub pipelines: Vec<Pipeline>,
    pub docker: Option<DockerConfig>,
    pub kubernetes: Option<KubernetesConfig>,
}

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image: String,
    pub registry: String,
    pub dockerfile: Option<String>,
}

/// Kubernetes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    pub namespace: String,
    pub replicas: u32,
    pub resources: HashMap<String, String>,
}

// ── Deploy Adapter Trait ────────────────────────────────────────────

/// Trait that deploy adapters must implement
pub trait DeployAdapter: Send {
    /// Generate deployment artifacts (YAML, Dockerfile, manifests, etc.)
    fn generate(&self, plan: &DeployPlan) -> Result<Vec<DeployArtifact>, DeployError>;

    /// Execute deployment
    fn deploy(&self, plan: &DeployPlan) -> Result<DeployResult, DeployError>;

    /// Rollback to previous version
    fn rollback(&self, app_name: &str) -> Result<(), DeployError>;

    /// Get adapter name
    fn name(&self) -> &str;
}

/// Generated deployment artifact
#[derive(Debug, Clone)]
pub struct DeployArtifact {
    pub filename: String,
    pub content: String,
}

/// Deployment result
#[derive(Debug, Clone)]
pub struct DeployResult {
    pub success: bool,
    pub url: Option<String>,
    pub message: String,
}

/// Deploy error.
///
/// Variants currently use `String` payloads for backward compatibility, but
/// constructors now accept structured input that gets formatted into the
/// human-readable string. New code should use the typed constructors.
#[derive(Debug, Clone)]
pub enum DeployError {
    ConfigError(String),
    BuildFailed(String),
    DeployFailed(String),
    RollbackFailed(String),
    AdapterError(String),
}

impl DeployError {
    /// Configuration error with structured context.
    pub fn config(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ConfigError(format!("key '{}': {}", key.into(), reason.into()))
    }

    /// Build failure with the failing step and reason.
    pub fn build_failed(step: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::BuildFailed(format!("step '{}': {}", step.into(), reason.into()))
    }

    /// Deploy failure with the target and reason.
    pub fn deploy_failed(target: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DeployFailed(format!("target '{}': {}", target.into(), reason.into()))
    }

    /// Rollback failure with the app name and reason.
    pub fn rollback_failed(app: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::RollbackFailed(format!("app '{}': {}", app.into(), reason.into()))
    }

    /// Adapter error with the adapter name and underlying issue.
    pub fn adapter(name: impl Into<String>, issue: impl Into<String>) -> Self {
        Self::AdapterError(format!("adapter '{}': {}", name.into(), issue.into()))
    }

    /// Stable error code for documentation lookup. (v0.4.47.9 — Issue #847)
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ConfigError(_) => "E_DEPLOY_CONFIG",
            Self::BuildFailed(_) => "E_DEPLOY_BUILD",
            Self::DeployFailed(_) => "E_DEPLOY_FAILED",
            Self::RollbackFailed(_) => "E_DEPLOY_ROLLBACK",
            Self::AdapterError(_) => "E_DEPLOY_ADAPTER",
        }
    }
}

impl std::fmt::Display for DeployError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeployError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            DeployError::BuildFailed(msg) => write!(f, "Build failed: {}", msg),
            DeployError::DeployFailed(msg) => write!(f, "Deploy failed: {}", msg),
            DeployError::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            DeployError::AdapterError(msg) => write!(f, "Adapter error: {}", msg),
        }
    }
}

impl std::error::Error for DeployError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl DeployError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            DeployError::AdapterError(..) => hudhudscript_errors::ErrorCode::DeployAdapterError,
            DeployError::BuildFailed(..) => hudhudscript_errors::ErrorCode::DeployBuildFailed,
            DeployError::ConfigError(..) => hudhudscript_errors::ErrorCode::DeployConfigError,
            DeployError::DeployFailed(..) => hudhudscript_errors::ErrorCode::DeployDeployFailed,
            DeployError::RollbackFailed(..) => hudhudscript_errors::ErrorCode::DeployRollbackFailed,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<DeployError> for hudhudscript_errors::Error {
    fn from(e: DeployError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
