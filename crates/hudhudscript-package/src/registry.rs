use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry client for package operations
#[derive(Debug, Clone)]
pub struct RegistryClient {
    base_url: String,
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        })
    }

    /// Search packages
    pub async fn search(&self, query: &str) -> Result<Vec<PackageMetadata>> {
        let url = format!("{}/api/v1/packages/search?q={}", self.base_url, query);
        let response = self.client.get(&url).send().await?;
        let results: SearchResponse = response.json().await?;
        Ok(results.packages)
    }

    /// Get package metadata
    pub async fn get_metadata(&self, name: &str) -> Result<PackageMetadata> {
        let url = format!("{}/api/v1/packages/{}", self.base_url, name);
        let response = self.client.get(&url).send().await?;
        let metadata: PackageMetadata = response.json().await?;
        Ok(metadata)
    }

    /// Get package version info
    pub async fn get_version(&self, name: &str, version: &str) -> Result<VersionInfo> {
        let url = format!("{}/api/v1/packages/{}/{}", self.base_url, name, version);
        let response = self.client.get(&url).send().await?;
        let info: VersionInfo = response.json().await?;
        Ok(info)
    }

    /// Download package
    pub async fn download(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/api/v1/packages/{}/{}/download",
            self.base_url, name, version
        );
        let response = self.client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Publish package
    pub async fn publish(&self, package: &PublishRequest, token: &str) -> Result<PublishResponse> {
        let url = format!("{}/api/v1/packages/publish", self.base_url);
        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(package)
            .send()
            .await?;
        let result: PublishResponse = response.json().await?;
        Ok(result)
    }

    /// Return the base URL of this registry.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Return a reference to the underlying HTTP client.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Get package statistics
    pub async fn get_stats(&self, name: &str) -> Result<PackageStats> {
        let url = format!("{}/api/v1/packages/{}/stats", self.base_url, name);
        let response = self.client.get(&url).send().await?;
        let stats: PackageStats = response.json().await?;
        Ok(stats)
    }
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub versions: Vec<String>,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub downloads: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub checksum: String,
    pub size: u64,
    pub published_at: DateTime<Utc>,
}

/// Search response
#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    packages: Vec<PackageMetadata>,
    total: usize,
}

/// Publish request
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub tarball: Vec<u8>,
    pub checksum: String,
}

/// Publish response
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub success: bool,
    pub message: String,
    pub package_url: String,
}

/// Package statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageStats {
    pub downloads_total: u64,
    pub downloads_last_week: u64,
    pub downloads_last_month: u64,
    pub dependents: u64,
}

/// Registry trait for custom registries
pub trait Registry: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<PackageMetadata>>;
    fn get_metadata(&self, name: &str) -> Result<PackageMetadata>;
    fn download(&self, name: &str, version: &str) -> Result<Vec<u8>>;
}
