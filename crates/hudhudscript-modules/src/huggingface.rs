//! HuggingFace Hub API client
//!
//! Provides model search, metadata retrieval, file listing, and download URL
//! generation against the HuggingFace Hub REST API.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Represents a model hosted on the HuggingFace Hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModel {
    /// Fully-qualified model identifier, e.g. `"TheBloke/Llama-2-7B-GGUF"`.
    pub model_id: String,
    /// Author / organisation name.
    pub author: String,
    /// Tags attached to the model card.
    pub tags: Vec<String>,
    /// All-time download count.
    pub downloads: u64,
    /// ISO-8601 timestamp of the last modification.
    pub last_modified: String,
    /// Pipeline tag (e.g. `"text-generation"`).
    pub pipeline_tag: Option<String>,
}

/// A single file inside a HuggingFace repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfFile {
    /// Path relative to the repository root.
    #[serde(rename = "rfilename")]
    pub filename: String,
    /// File size in bytes (may be absent for LFS pointers in listing).
    pub size: Option<u64>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Lightweight, non-blocking client for the HuggingFace Hub HTTP API.
///
/// All network calls are performed via `reqwest`; the client itself is
/// cheaply cloneable because it wraps an `Arc` internally.
#[derive(Debug, Clone)]
pub struct HfClient {
    /// Base URL (default: `https://huggingface.co`).
    pub base_url: String,
    /// Optional Bearer token for gated / private models.
    pub auth_token: Option<String>,
    client: reqwest::Client,
}

/// Errors that can occur while talking to the Hub.
#[derive(Debug)]
pub enum HfError {
    Http(String),
    Deserialize(String),
}

impl std::fmt::Display for HfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            HfError::Http(s) => write!(f, "HTTP request failed: {}", s),
            HfError::Deserialize(s) => write!(f, "Failed to deserialize response: {}", s),
        }
    }
}

impl std::error::Error for HfError {}

impl HfClient {
    /// Create a new client with the given base URL and optional auth token.
    pub fn new(base_url: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth_token,
            client: reqwest::Client::new(),
        }
    }

    /// Create a client pointing at the public Hub.
    pub fn public() -> Self {
        Self::new("https://huggingface.co", None)
    }

    // -- helpers ------------------------------------------------------------

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.get(url);
        if let Some(ref token) = self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }

    // -- public API ---------------------------------------------------------

    /// Search for models matching `query`, returning at most `limit` results.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<HfModel>, HfError> {
        let url = format!(
            "{}/api/models?search={}&limit={}&sort=downloads&direction=-1",
            self.base_url, query, limit,
        );

        let resp: Vec<serde_json::Value> = self
            .build_request(&url)
            .send()
            .await
            .map_err(|e| HfError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| HfError::Deserialize(e.to_string()))?;

        Ok(resp.into_iter().map(|v| value_to_hf_model(&v)).collect())
    }

    /// Retrieve detailed model metadata for a single `model_id`.
    pub async fn model_info(&self, model_id: &str) -> Result<HfModel, HfError> {
        let url = format!("{}/api/models/{}", self.base_url, model_id);

        let v: serde_json::Value = self
            .build_request(&url)
            .send()
            .await
            .map_err(|e| HfError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| HfError::Deserialize(e.to_string()))?;

        Ok(value_to_hf_model(&v))
    }

    /// List every file in the repository identified by `model_id`.
    pub async fn list_files(&self, model_id: &str) -> Result<Vec<HfFile>, HfError> {
        let url = format!("{}/api/models/{}", self.base_url, model_id);

        let v: serde_json::Value = self
            .build_request(&url)
            .send()
            .await
            .map_err(|e| HfError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| HfError::Deserialize(e.to_string()))?;

        let siblings = v
            .get("siblings")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        siblings
            .into_iter()
            .map(|s| serde_json::from_value(s).map_err(|e| HfError::Deserialize(e.to_string())))
            .collect()
    }

    /// Build the direct download URL for a file inside a model repository.
    pub fn download_url(&self, model_id: &str, filename: &str) -> String {
        format!("{}/{}/resolve/main/{}", self.base_url, model_id, filename)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn value_to_hf_model(v: &serde_json::Value) -> HfModel {
    let model_id = v
        .get("modelId")
        .or_else(|| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let author = v
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let tags = v
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let downloads = v.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);

    let last_modified = v
        .get("lastModified")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let pipeline_tag = v
        .get("pipeline_tag")
        .and_then(|v| v.as_str())
        .map(String::from);

    HfModel {
        model_id,
        author,
        tags,
        downloads,
        last_modified,
        pipeline_tag,
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl HfError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            HfError::Deserialize(..) => hudhudscript_errors::ErrorCode::HfDeserialize,
            HfError::Http(..) => hudhudscript_errors::ErrorCode::HfHttp,
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

impl From<HfError> for hudhudscript_errors::Error {
    fn from(e: HfError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
