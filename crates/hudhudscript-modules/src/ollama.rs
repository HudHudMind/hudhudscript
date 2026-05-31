//! Ollama registry API client
//!
//! Communicates with a running Ollama instance to list local models and
//! retrieve manifests for layer-level deduplication and tagging.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A locally available Ollama model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    /// Model name (e.g. `"llama2"`).
    pub name: String,
    /// Tag (e.g. `"7b-q4_0"`).
    pub tag: String,
    /// Total size in bytes.
    pub size: u64,
    /// Digest (SHA-256).
    pub digest: String,
    /// ISO-8601 last-modified timestamp.
    pub modified_at: String,
}

/// A single layer inside an Ollama manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaLayer {
    /// Media type of the layer.
    #[serde(rename = "mediaType")]
    pub media_type: String,
    /// Content digest.
    pub digest: String,
    /// Layer size in bytes.
    pub size: u64,
}

/// An Ollama manifest describing a model image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaManifest {
    /// Layers that compose the model.
    pub layers: Vec<OllamaLayer>,
    /// Config layer (optional).
    pub config: Option<OllamaLayer>,
    /// Top-level media type.
    #[serde(rename = "mediaType")]
    pub media_type: String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for the local Ollama REST API (default port 11434).
#[derive(Debug, Clone)]
pub struct OllamaClient {
    /// Base URL (default: `http://localhost:11434`).
    pub base_url: String,
    client: reqwest::Client,
}

/// Errors originating from Ollama API calls.
#[derive(Debug)]
pub enum OllamaError {
    Http(String),
    Deserialize(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            OllamaError::Http(s) => write!(f, "HTTP request failed: {}", s),
            OllamaError::Deserialize(s) => write!(f, "Failed to deserialize response: {}", s),
        }
    }
}

impl std::error::Error for OllamaError {}

impl OllamaClient {
    /// Create a new client pointing at `base_url`.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a client with the default local endpoint.
    pub fn local() -> Self {
        Self::new("http://localhost:11434")
    }

    /// List all locally available models.
    pub async fn list_local(&self) -> Result<Vec<OllamaModel>, OllamaError> {
        let url = format!("{}/api/tags", self.base_url);

        let resp: serde_json::Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| OllamaError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| OllamaError::Deserialize(e.to_string()))?;

        let models = resp
            .get("models")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(models
            .into_iter()
            .map(|v| value_to_ollama_model(&v))
            .collect())
    }

    /// Pull the manifest for a given model `name` (e.g. `"llama2:7b"`).
    pub async fn pull_manifest(&self, name: &str) -> Result<OllamaManifest, OllamaError> {
        let url = format!("{}/api/show", self.base_url);

        let body = serde_json::json!({ "name": name });

        let resp: serde_json::Value = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| OllamaError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| OllamaError::Deserialize(e.to_string()))?;

        value_to_manifest(&resp)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn value_to_ollama_model(v: &serde_json::Value) -> OllamaModel {
    let full_name = v
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    let (name, tag) = if let Some(idx) = full_name.find(':') {
        (
            full_name[..idx].to_string(),
            full_name[idx + 1..].to_string(),
        )
    } else {
        (full_name.clone(), "latest".to_string())
    };

    let size = v.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
    let digest = v
        .get("digest")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();
    let modified_at = v
        .get("modified_at")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();

    OllamaModel {
        name,
        tag,
        size,
        digest,
        modified_at,
    }
}

pub fn value_to_manifest(v: &serde_json::Value) -> Result<OllamaManifest, OllamaError> {
    let media_type = v
        .get("mediaType")
        .and_then(|m| m.as_str())
        .unwrap_or("application/vnd.ollama.image.manifest.v1+json")
        .to_string();

    let layers: Vec<OllamaLayer> = v
        .get("layers")
        .and_then(|l| serde_json::from_value(l.clone()).ok())
        .unwrap_or_default();

    let config: Option<OllamaLayer> = v
        .get("config")
        .and_then(|c| serde_json::from_value(c.clone()).ok());

    Ok(OllamaManifest {
        layers,
        config,
        media_type,
    })
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl OllamaError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            OllamaError::Deserialize(..) => hudhudscript_errors::ErrorCode::OllamaDeserialize,
            OllamaError::Http(..) => hudhudscript_errors::ErrorCode::OllamaHttp,
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

impl From<OllamaError> for hudhudscript_errors::Error {
    fn from(e: OllamaError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
