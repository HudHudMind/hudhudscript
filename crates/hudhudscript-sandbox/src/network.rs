//! Network sandbox

use crate::{NetworkConfig, Result, SandboxError};

pub struct NetworkSandbox {
    config: NetworkConfig,
}

impl NetworkSandbox {
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    /// Check if network access is allowed
    pub fn check_access(&self, host: &str, port: u16) -> Result<()> {
        // Check deny list first
        for denied in &self.config.deny_hosts {
            if self.host_matches(host, denied) {
                return Err(SandboxError::NetworkDenied(format!(
                    "Access to {} is explicitly denied",
                    host
                )));
            }
        }

        // Check port if port list is not empty
        if !self.config.allow_ports.is_empty() && !self.config.allow_ports.contains(&port) {
            return Err(SandboxError::NetworkDenied(format!(
                "Port {} is not allowed",
                port
            )));
        }

        // Check domain allow list
        for allowed in &self.config.allow_domains {
            if self.host_matches(host, allowed) {
                return Ok(());
            }
        }

        Err(SandboxError::NetworkDenied(format!(
            "Access to {} is not allowed",
            host
        )))
    }

    /// Check if a host matches a pattern
    fn host_matches(&self, host: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(suffix) = pattern.strip_prefix("*.") {
            // *.example.com should match sub.example.com and example.com
            // but NOT notexample.com — require dot boundary
            if host == suffix {
                return true;
            }
            return host.ends_with(suffix)
                && host.len() > suffix.len()
                && host.as_bytes()[host.len() - suffix.len() - 1] == b'.';
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            // 192.168.* should match 192.168.1.1 — ensure dot boundary
            if !prefix.is_empty() && prefix.ends_with('.') {
                return host.starts_with(prefix);
            }
            // Boundary check: after prefix must be '.' or ':' or end of string
            if let Some(rest) = host.strip_prefix(prefix) {
                if rest.is_empty() || rest.starts_with('.') || rest.starts_with(':') {
                    return true;
                }
            }
            return false;
        }

        host == pattern
    }
}
