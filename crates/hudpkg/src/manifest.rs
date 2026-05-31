use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::MANIFEST;

/// Root manifest read from `hudhud.toml`.
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct HudHudManifest {
    #[serde(default)]
    pub(crate) package: PackageInfo,
    #[serde(default)]
    pub(crate) dependencies: BTreeMap<String, DependencyValue>,
}

/// Package metadata inside a manifest.
#[derive(Serialize, Deserialize)]
pub(crate) struct PackageInfo {
    #[serde(default = "default_name")]
    pub(crate) name: String,
    #[serde(default = "default_version")]
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) authors: Vec<String>,
}

impl Default for PackageInfo {
    fn default() -> Self {
        Self {
            name: default_name(),
            version: default_version(),
            description: String::new(),
            authors: vec![],
        }
    }
}

/// A dependency can be specified as a simple version string or as a table
/// with path/version/features.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum DependencyValue {
    /// Simple version string, e.g. `"^0.1"`
    Simple(String),
    /// Detailed dependency with optional path override.
    Detailed {
        version: String,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        features: Vec<String>,
    },
}

impl DependencyValue {
    pub(crate) fn version_str(&self) -> &str {
        match self {
            DependencyValue::Simple(v) => v,
            DependencyValue::Detailed { version, .. } => version,
        }
    }

    /// If a `path` override is given, return it.
    pub(crate) fn path_override(&self) -> Option<&str> {
        match self {
            DependencyValue::Simple(_) => None,
            DependencyValue::Detailed { path, .. } => path.as_deref(),
        }
    }
}

/// Local package source manifest (`packages/<name>/hudhud.toml`).
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct LocalPackageManifest {
    #[serde(default)]
    pub(crate) package: PackageInfo,
    #[serde(default)]
    pub(crate) dependencies: BTreeMap<String, DependencyValue>,
}

pub(crate) fn default_name() -> String {
    "my-project".to_string()
}

pub(crate) fn default_version() -> String {
    "0.1.0".to_string()
}

pub(crate) fn load_manifest() -> HudHudManifest {
    if !Path::new(MANIFEST).exists() {
        eprintln!("No hudhud.toml found. Run 'hudpkg init' first.");
        std::process::exit(1);
    }
    let content = fs::read_to_string(MANIFEST).expect("Failed to read hudhud.toml");
    toml::from_str(&content).expect("Failed to parse hudhud.toml")
}

pub(crate) fn save_manifest(manifest: &HudHudManifest) {
    let content = toml::to_string_pretty(manifest).expect("Failed to serialize");
    fs::write(MANIFEST, content).expect("Failed to write hudhud.toml");
}
