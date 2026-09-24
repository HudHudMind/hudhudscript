//! `hudunit.toml` project configuration + defaults.

use std::path::{Path, PathBuf};

/// Default test source extensions recognized during discovery.
pub const DEFAULT_EXTENSIONS: &[&str] = &["hhs", "hud", "hudhud"];

/// Project-level configuration, optionally loaded from `hudunit.toml`.
#[derive(Debug, Clone)]
pub struct HudunitConfig {
    /// Directory scanned when the CLI is invoked without paths.
    pub test_dir: PathBuf,
    /// Extra extensions beyond the built-in `hhs`/`hud`/`hudhud` set.
    pub extensions: Vec<String>,
    /// Per-test wall-clock timeout in milliseconds (best-effort; the VM fuel
    /// limit is the hard stop).
    pub timeout_ms: u64,
    /// VM fuel limit for a single test (guards infinite loops).
    pub fuel: Option<u64>,
    /// Coverage line-percentage threshold; failing it makes the run fail.
    pub coverage_threshold: Option<f64>,
}

impl Default for HudunitConfig {
    fn default() -> Self {
        Self {
            test_dir: PathBuf::from("tests"),
            extensions: DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            timeout_ms: 10_000,
            fuel: Some(500_000_000),
            coverage_threshold: None,
        }
    }
}

impl HudunitConfig {
    /// Load `hudunit.toml` from `dir` (or its parents are NOT searched — the
    /// file must sit next to where the user runs `hudunit`). Missing file or
    /// unknown keys fall back to defaults; the config is intentionally small.
    pub fn load(dir: &Path) -> Self {
        let mut cfg = Self::default();
        let path = dir.join("hudunit.toml");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return cfg;
        };
        let Ok(value) = text.parse::<toml::Table>() else {
            return cfg;
        };
        if let Some(dir) = value.get("test_dir").and_then(|v| v.as_str()) {
            cfg.test_dir = PathBuf::from(dir);
        }
        if let Some(exts) = value.get("extensions").and_then(|v| v.as_array()) {
            let parsed: Vec<String> = exts
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.trim_start_matches('.').to_string()))
                .collect();
            if !parsed.is_empty() {
                let mut all: Vec<String> =
                    DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect();
                for ext in parsed {
                    if !all.contains(&ext) {
                        all.push(ext);
                    }
                }
                cfg.extensions = all;
            }
        }
        if let Some(ms) = value.get("timeout_ms").and_then(|v| v.as_integer()) {
            if ms > 0 {
                cfg.timeout_ms = ms as u64;
            }
        }
        if let Some(fuel) = value.get("fuel").and_then(|v| v.as_integer()) {
            cfg.fuel = if fuel < 0 { None } else { Some(fuel as u64) };
        }
        if let Some(threshold) = value.get("coverage_threshold").and_then(|v| v.as_float()) {
            cfg.coverage_threshold = Some(threshold.clamp(0.0, 100.0));
        }
        cfg
    }

    /// True when `path` has one of the configured test extensions.
    pub fn is_test_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.extensions.iter().any(|allowed| allowed == ext))
            .unwrap_or(false)
    }
}
