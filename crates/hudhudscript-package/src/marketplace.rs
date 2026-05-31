//! Plugin marketplace protocol — search, metadata, versioning, categories.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::registry::RegistryClient;
use crate::signature::PackageSignature;
use crate::{PackageError, Result};

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// A single entry in the marketplace index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub rating: f64,
    pub tags: Vec<String>,
    pub signature: Option<PackageSignature>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sort criteria for marketplace search results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    Relevance,
    Downloads,
    Rating,
    RecentlyUpdated,
    Name,
}

/// Query parameters for a marketplace search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sort_by: SortBy,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

pub fn default_page() -> u32 {
    1
}
pub fn default_per_page() -> u32 {
    20
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            tags: Vec::new(),
            sort_by: SortBy::default(),
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

/// Paginated search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entries: Vec<MarketplaceEntry>,
    pub total_count: u64,
    pub page: u32,
}

/// A marketplace category with a human-readable label and entry count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub description: String,
    pub package_count: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Marketplace client
// ─────────────────────────────────────────────────────────────────────────────

/// Client for the HudHudScript plugin marketplace.
#[derive(Debug, Clone)]
pub struct Marketplace {
    registry: RegistryClient,
}

impl Marketplace {
    /// Create a new marketplace client backed by `registry`.
    pub fn new(registry: RegistryClient) -> Self {
        Self { registry }
    }

    /// Search the marketplace using the given query parameters.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/api/v1/marketplace/search", self.registry.base_url());

        let response = self
            .registry
            .client()
            .post(&url)
            .json(query)
            .send()
            .await
            .map_err(PackageError::Network)?;

        let result: SearchResult = response.json().await.map_err(PackageError::Network)?;
        Ok(result)
    }

    /// Fetch a single marketplace entry by package name.
    pub async fn get_entry(&self, name: &str) -> Result<MarketplaceEntry> {
        let url = format!(
            "{}/api/v1/marketplace/packages/{}",
            self.registry.base_url(),
            name,
        );

        let response = self
            .registry
            .client()
            .get(&url)
            .send()
            .await
            .map_err(PackageError::Network)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageError::PackageNotFound(name.to_string()));
        }

        let entry: MarketplaceEntry = response.json().await.map_err(PackageError::Network)?;
        Ok(entry)
    }

    /// List all available categories.
    pub async fn list_categories(&self) -> Result<Vec<Category>> {
        let url = format!("{}/api/v1/marketplace/categories", self.registry.base_url(),);

        let response = self
            .registry
            .client()
            .get(&url)
            .send()
            .await
            .map_err(PackageError::Network)?;

        let categories: Vec<Category> = response.json().await.map_err(PackageError::Network)?;
        Ok(categories)
    }

    /// Fetch the list of featured / editor-pick packages.
    pub async fn featured(&self) -> Result<Vec<MarketplaceEntry>> {
        let url = format!("{}/api/v1/marketplace/featured", self.registry.base_url(),);

        let response = self
            .registry
            .client()
            .get(&url)
            .send()
            .await
            .map_err(PackageError::Network)?;

        let entries: Vec<MarketplaceEntry> =
            response.json().await.map_err(PackageError::Network)?;
        Ok(entries)
    }
}
