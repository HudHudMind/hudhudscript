use hudhudscript_sandbox::{NetworkSandbox, SandboxConfig};

use super::ToolError;

thread_local! {
    /// Ambient sandbox config — set by the interpreter when a script runs
    /// inside a sandboxed context. Reset to None outside that context.
    pub static AMBIENT_SANDBOX: std::cell::RefCell<Option<SandboxConfig>> =
        const { std::cell::RefCell::new(None) };
}

/// Extract host and port from a URL string, then check the sandbox network
/// policy.  Returns `Err(ToolError::SecurityViolation)` when access is denied.
pub(crate) fn check_url_against_sandbox(url_str: &str, cfg: &SandboxConfig) -> Result<(), ToolError> {
    let parsed = url::Url::parse(url_str)
        .map_err(|e| ToolError::InvalidArguments(format!("invalid URL '{}': {}", url_str, e)))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| ToolError::InvalidArguments(format!("URL '{}' has no host", url_str)))?;

    let default_port = match parsed.scheme() {
        "https" => 443,
        "http" => 80,
        _ => 0,
    };
    let port = parsed.port().unwrap_or(default_port);

    let net_sandbox = NetworkSandbox::new(cfg.network.clone());
    net_sandbox.check_access(host, port).map_err(|e| {
        ToolError::SecurityViolation(format!("http access denied for '{}': {}", url_str, e))
    })
}
