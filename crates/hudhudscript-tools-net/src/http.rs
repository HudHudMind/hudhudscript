//! HttpTool — Built-in REST API tool for HudHudScript agents
//!
//! Allows agents to make HTTP requests (GET, POST, PUT, DELETE) with
//! header, auth, JSON body, timeout, and retry support.

use hudhudscript_utils::RetryConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// HTTP method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
        }
    }
}

/// Authentication configuration for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HttpAuth {
    /// Bearer token: Authorization: Bearer <token>
    Bearer { token: String },
    /// API key header: X-API-Key: <key>
    ApiKey { key: String, header: Option<String> },
    /// Basic auth: Authorization: Basic base64(user:pass)
    Basic { username: String, password: String },
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method
    pub method: HttpMethod,
    /// Full URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Authentication
    pub auth: Option<HttpAuth>,
    /// JSON body (for POST/PUT/PATCH)
    pub body: Option<serde_json::Value>,
    /// Timeout in seconds (default: 30)
    pub timeout_secs: Option<u64>,
    /// Number of retries on failure (default: 0)
    pub retries: Option<u32>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as JSON (if parseable) or string
    pub body: serde_json::Value,
    /// Whether the request was successful (2xx)
    pub ok: bool,
}

/// HTTP tool errors
#[derive(Debug)]
pub enum HttpToolError {
    RequestFailed(String),
    ParseError(String),
    InvalidUrl(String),
    Timeout(u64),
}

impl std::fmt::Display for HttpToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            HttpToolError::RequestFailed(s) => write!(f, "HTTP request failed: {}", s),
            HttpToolError::ParseError(s) => write!(f, "Response parse error: {}", s),
            HttpToolError::InvalidUrl(s) => write!(f, "Invalid URL: {}", s),
            HttpToolError::Timeout(secs) => write!(f, "Timeout after {}s", secs),
        }
    }
}

impl std::error::Error for HttpToolError {}

impl From<reqwest::Error> for HttpToolError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            HttpToolError::Timeout(30)
        } else {
            HttpToolError::RequestFailed(e.to_string())
        }
    }
}

/// Built-in HTTP tool for agent REST API calls
pub struct HttpTool {
    client: reqwest::Client,
}

impl HttpTool {
    /// Create a new HttpTool with default settings
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Execute an HTTP request with optional retry (powered by hudhudscript-utils).
    pub async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, HttpToolError> {
        let retries = request.retries.unwrap_or(0);
        if retries == 0 {
            return self.do_request(&request).await;
        }

        let config = RetryConfig::new(retries, Duration::from_millis(100));
        hudhudscript_utils::retry(&config, || self.do_request(&request))
            .await
            .map_err(|e| e.last_error)
    }

    async fn do_request(&self, request: &HttpRequest) -> Result<HttpResponse, HttpToolError> {
        let timeout = Duration::from_secs(request.timeout_secs.unwrap_or(30));

        let mut builder = match request.method {
            HttpMethod::Get => self.client.get(&request.url),
            HttpMethod::Post => self.client.post(&request.url),
            HttpMethod::Put => self.client.put(&request.url),
            HttpMethod::Delete => self.client.delete(&request.url),
            HttpMethod::Patch => self.client.patch(&request.url),
            HttpMethod::Head => self.client.head(&request.url),
        };

        builder = builder.timeout(timeout);

        // Add custom headers
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        // Add authentication
        if let Some(auth) = &request.auth {
            builder = match auth {
                HttpAuth::Bearer { token } => {
                    builder.header("Authorization", format!("Bearer {}", token))
                }
                HttpAuth::ApiKey { key, header } => {
                    let header_name = header.as_deref().unwrap_or("X-API-Key");
                    builder.header(header_name, key)
                }
                HttpAuth::Basic { username, password } => {
                    builder.basic_auth(username, Some(password))
                }
            };
        }

        // Add JSON body
        if let Some(body) = &request.body {
            builder = builder.json(body);
        }

        let response = builder.send().await?;

        let status = response.status().as_u16();
        let ok = response.status().is_success();

        // Collect response headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        // Parse body as JSON, fall back to string
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
        let body = serde_json::from_str(&body_text).unwrap_or(serde_json::Value::String(body_text));

        Ok(HttpResponse {
            status,
            headers,
            body,
            ok,
        })
    }
}

impl Default for HttpTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource configuration for HudHudScript `kaynak` declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResource {
    /// Base URL for the API
    pub base_url: String,
    /// Default authentication
    pub auth: Option<HttpAuth>,
    /// Default headers
    pub headers: HashMap<String, String>,
}

impl RestResource {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth: None,
            headers: HashMap::new(),
        }
    }

    /// Build a full request for this resource
    pub fn request(&self, method: HttpMethod, path: &str) -> HttpRequest {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        HttpRequest {
            method,
            url,
            headers: self.headers.clone(),
            auth: self.auth.clone(),
            body: None,
            timeout_secs: None,
            retries: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl HttpToolError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            HttpToolError::InvalidUrl(..) => hudhudscript_errors::ErrorCode::HttpToolInvalidUrl,
            HttpToolError::ParseError(..) => hudhudscript_errors::ErrorCode::HttpToolParseError,
            HttpToolError::RequestFailed(..) => {
                hudhudscript_errors::ErrorCode::HttpToolRequestFailed
            }
            HttpToolError::Timeout(..) => hudhudscript_errors::ErrorCode::HttpToolTimeout,
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

impl From<HttpToolError> for hudhudscript_errors::Error {
    fn from(e: HttpToolError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
