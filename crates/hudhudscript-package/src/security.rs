use crate::{DependencySpec, PackageError, RegistryClient, Result, DEFAULT_REGISTRY};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SecurityChecker {
    pub registry: Option<RegistryClient>,
    /// Local advisory database.
    pub advisories: Vec<Advisory>,
}

/// A single security advisory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub package: String,
    pub affected_versions: String,
    pub title: String,
    pub severity: String,
    pub url: Option<String>,
}

impl Default for SecurityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityChecker {
    pub fn new() -> Self {
        Self {
            registry: None,
            advisories: Vec::new(),
        }
    }

    /// Create a SecurityChecker backed by a registry client for remote
    /// checksum lookups and an optional local advisory database.
    pub fn with_registry(registry: RegistryClient, advisories: Vec<Advisory>) -> Self {
        Self {
            registry: Some(registry),
            advisories,
        }
    }

    /// Load advisories from a JSON file. Returns empty Vec on failure.
    pub fn load_advisories_from_file(path: &str) -> Vec<Advisory> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Verify SHA-256 checksum of `data` against the registry's expected value.
    pub async fn verify_checksum(&self, data: &[u8], name: &str, version: &str) -> Result<()> {
        let actual = calculate_checksum(data);

        if let Some(ref registry) = self.registry {
            let version_info = registry.get_version(name, version).await.map_err(|e| {
                PackageError::ChecksumMismatch(format!(
                    "Failed to fetch checksum for {}@{}: {}",
                    name, version, e
                ))
            })?;

            if actual != version_info.checksum {
                return Err(PackageError::ChecksumMismatch(format!(
                    "Checksum mismatch for {}@{}: expected {}, got {}",
                    name, version, version_info.checksum, actual
                )));
            }
        }

        Ok(())
    }

    /// Audit dependencies against the local advisory database and remote endpoint.
    pub async fn audit_dependencies(
        &self,
        deps: &HashMap<String, DependencySpec>,
    ) -> Result<Vec<String>> {
        let mut warnings: Vec<String> = Vec::new();

        // Check against local advisories
        for (name, spec) in deps {
            let version_str = spec.version().trim();
            if let Ok(version) = Version::parse(version_str) {
                for advisory in &self.advisories {
                    if advisory.package != *name {
                        continue;
                    }
                    if let Ok(req) = semver::VersionReq::parse(&advisory.affected_versions) {
                        if req.matches(&version) {
                            warnings.push(format!(
                                "[{}] {} ({}@{}) - severity: {}{}",
                                advisory.id,
                                advisory.title,
                                name,
                                version,
                                advisory.severity,
                                advisory
                                    .url
                                    .as_ref()
                                    .map(|u| format!(" — {}", u))
                                    .unwrap_or_default(),
                            ));
                        }
                    }
                }
            }
        }

        // Try remote audit endpoint (best-effort)
        if let Some(ref _registry) = self.registry {
            if let Ok(remote_warnings) = self.remote_audit(deps).await {
                warnings.extend(remote_warnings)
            }
        }

        Ok(warnings)
    }

    /// Query the registry's audit endpoint for known vulnerabilities.
    async fn remote_audit(
        &self,
        deps: &HashMap<String, DependencySpec>,
    ) -> std::result::Result<Vec<String>, anyhow::Error> {
        let packages: Vec<AuditQueryEntry> = deps
            .iter()
            .map(|(name, spec)| AuditQueryEntry {
                name: name.clone(),
                version: spec.version().to_string(),
            })
            .collect();

        let query = AuditQuery { packages };

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/audit", DEFAULT_REGISTRY);
        let response = client.post(&url).json(&query).send().await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let result: AuditResponse = response.json().await?;
        Ok(result
            .advisories
            .into_iter()
            .map(|a| {
                format!(
                    "[{}] {} ({}@{}) - severity: {}",
                    a.id, a.title, a.package, a.affected_versions, a.severity
                )
            })
            .collect())
    }
}

#[derive(Debug, Serialize)]
struct AuditQuery {
    packages: Vec<AuditQueryEntry>,
}

#[derive(Debug, Serialize)]
struct AuditQueryEntry {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct AuditResponse {
    advisories: Vec<Advisory>,
}

pub type Checksum = String;

pub fn calculate_checksum(data: &[u8]) -> Checksum {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
