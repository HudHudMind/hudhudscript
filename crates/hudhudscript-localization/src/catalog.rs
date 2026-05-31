//! Message catalog for i18n / multi-language support
//!
//! Loads translated message strings from JSON or YAML files and provides
//! key-based lookup with fallback chain support.

use std::collections::HashMap;
use std::path::Path;

/// A single-locale message catalog mapping keys to translated strings.
#[derive(Debug, Clone)]
pub struct MessageCatalog {
    /// BCP-47 locale tag, e.g. "tr-TR", "en"
    locale: String,
    /// key -> translated message
    messages: HashMap<String, String>,
}

/// Errors that can occur when loading a catalog.
#[derive(Debug)]
pub enum CatalogError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CatalogError::Io(e) => write!(f, "IO error: {}", e),
            CatalogError::Json(e) => write!(f, "JSON parse error: {}", e),
            CatalogError::Yaml(e) => write!(f, "YAML parse error: {}", e),
        }
    }
}

impl std::error::Error for CatalogError {}

impl From<std::io::Error> for CatalogError {
    fn from(e: std::io::Error) -> Self {
        CatalogError::Io(e)
    }
}
impl From<serde_json::Error> for CatalogError {
    fn from(e: serde_json::Error) -> Self {
        CatalogError::Json(e)
    }
}
impl From<serde_yaml::Error> for CatalogError {
    fn from(e: serde_yaml::Error) -> Self {
        CatalogError::Yaml(e)
    }
}

impl MessageCatalog {
    /// Create a new empty catalog for the given locale.
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
            messages: HashMap::new(),
        }
    }

    /// Create a catalog from an existing map.
    pub fn from_map(locale: impl Into<String>, messages: HashMap<String, String>) -> Self {
        Self {
            locale: locale.into(),
            messages,
        }
    }

    /// Load a catalog from a JSON file.
    ///
    /// The file must contain a flat `{ "key": "value" }` object.
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_json_str(&content)
    }

    /// Parse a catalog from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, CatalogError> {
        let raw: serde_json::Value = serde_json::from_str(json)?;
        let mut messages = HashMap::new();
        let mut locale = String::new();

        if let Some(obj) = raw.as_object() {
            if let Some(loc) = obj.get("_locale").and_then(|v| v.as_str()) {
                locale = loc.to_string();
            }
            for (k, v) in obj {
                if k.starts_with('_') {
                    continue; // skip metadata keys
                }
                if let Some(s) = v.as_str() {
                    messages.insert(k.clone(), s.to_string());
                }
            }
        }

        Ok(Self { locale, messages })
    }

    /// Load a catalog from a YAML file.
    pub fn load_yaml(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_yaml_str(&content)
    }

    /// Parse a catalog from a YAML string.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, CatalogError> {
        let raw: serde_yaml::Value = serde_yaml::from_str(yaml)?;
        let mut messages = HashMap::new();
        let mut locale = String::new();

        if let Some(map) = raw.as_mapping() {
            for (k, v) in map {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    if key == "_locale" {
                        locale = val.to_string();
                    } else if !key.starts_with('_') {
                        messages.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }

        Ok(Self { locale, messages })
    }

    /// Get the locale of this catalog.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Look up a message by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.messages.get(key).map(|s| s.as_str())
    }

    /// Look up a message by key, falling back through a chain of catalogs.
    ///
    /// Returns the first matching value, or the key itself if no catalog has it.
    pub fn get_with_fallback<'a>(
        &'a self,
        key: &'a str,
        fallbacks: &'a [MessageCatalog],
    ) -> &'a str {
        if let Some(v) = self.get(key) {
            return v;
        }
        for fb in fallbacks {
            if let Some(v) = fb.get(key) {
                return v;
            }
        }
        key
    }

    /// Insert a message into the catalog.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.messages.insert(key.into(), value.into());
    }

    /// Number of messages in this catalog.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether this catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl CatalogError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CatalogError::Io(..) => hudhudscript_errors::ErrorCode::CatalogIo,
            CatalogError::Json(..) => hudhudscript_errors::ErrorCode::CatalogJson,
            CatalogError::Yaml(..) => hudhudscript_errors::ErrorCode::CatalogYaml,
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

impl From<CatalogError> for hudhudscript_errors::Error {
    fn from(e: CatalogError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
