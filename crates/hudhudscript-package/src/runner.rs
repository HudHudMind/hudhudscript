//! Package runner — locates and executes the entry point of a HudHudScript
//! package.

use crate::config::{HudhudConfig, PackageType};
use crate::Result;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Entry-point discovery
// ─────────────────────────────────────────────────────────────────────────────

/// Search for the entry point script inside `package_dir`.
///
/// Resolution order:
/// 1. The explicit `entry` field from the manifest (if present).
/// 2. `main.hud` in the package root.
/// 3. `src/main.hud`.
///
/// Returns `None` when no candidate exists on disk.
pub fn find_entry_point(package_dir: &Path) -> Option<PathBuf> {
    // Try loading the manifest first so we can honour the `entry` field.
    let manifest_path = package_dir.join("hudhud.toml");
    if let Ok(config) = HudhudConfig::load_from_path(&manifest_path) {
        if let Some(entry) = &config.package.entry {
            let p = package_dir.join(entry);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // Fallback: conventional locations.
    for candidate in &["main.hud", "src/main.hud"] {
        let p = package_dir.join(candidate);
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

/// Returns `true` when the config describes an application package (i.e. it
/// is expected to have an executable entry point).
pub fn is_application_package(config: &HudhudConfig) -> bool {
    config.package.package_type == PackageType::Application
}

// ─────────────────────────────────────────────────────────────────────────────
// Library module discovery
// ─────────────────────────────────────────────────────────────────────────────

/// Collect all `.hud` files that a library package exports.
///
/// Exported modules are discovered from:
/// 1. `lib/` directory (conventional location for library sources).
/// 2. Any `.hud` file in the package root (excluding `main.hud`).
///
/// Module names are returned without the `.hud` extension, using `/` as the
/// separator for nested modules.
pub fn get_exported_modules(package_dir: &Path) -> Vec<String> {
    let mut modules = Vec::new();

    // Scan lib/ directory recursively
    let lib_dir = package_dir.join("lib");
    if lib_dir.is_dir() {
        collect_modules(&lib_dir, &lib_dir, &mut modules);
    }

    // Also scan root-level .hud files (but not main.hud)
    if let Ok(entries) = std::fs::read_dir(package_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "hud" {
                        let stem = path.file_stem().unwrap().to_string_lossy();
                        if stem != "main" {
                            modules.push(stem.to_string());
                        }
                    }
                }
            }
        }
    }

    modules.sort();
    modules.dedup();
    modules
}

/// Recursively collect `.hud` files relative to `base`.
pub fn collect_modules(dir: &Path, base: &Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_modules(&path, base, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("hud") {
            if let Ok(rel) = path.strip_prefix(base) {
                let module_name = rel
                    .with_extension("")
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/");
                out.push(module_name);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Package runner
// ─────────────────────────────────────────────────────────────────────────────

/// High-level package runner that loads configuration, locates the entry
/// point, and prepares the execution environment.
pub struct PackageRunner {
    /// Root directory of the package.
    pub package_dir: PathBuf,
    /// Loaded configuration.
    pub config: HudhudConfig,
}

impl PackageRunner {
    /// Create a runner for the package at `package_dir`.
    pub fn new(package_dir: &Path) -> Result<Self> {
        let manifest = package_dir.join("hudhud.toml");
        let config = if manifest.is_file() {
            HudhudConfig::load_from_path(&manifest).map_err(crate::PackageError::Other)?
        } else {
            HudhudConfig::default()
        };

        Ok(Self {
            package_dir: package_dir.to_path_buf(),
            config,
        })
    }

    /// Locate the entry point. Returns an error if this is an application
    /// package but no entry point was found.
    pub fn entry_point(&self) -> Result<PathBuf> {
        find_entry_point(&self.package_dir).ok_or_else(|| {
            crate::PackageError::Other(anyhow::anyhow!(
                "No entry point found in {}. \
                 Application packages require a main.hud file or an `entry` field in hudhud.toml.",
                self.package_dir.display()
            ))
        })
    }

    /// Return the list of AI provider names declared in the manifest.
    pub fn ai_providers(&self) -> Vec<String> {
        self.config.ai_providers.keys().cloned().collect()
    }

    /// Return the list of MCP server names declared in the manifest.
    pub fn mcp_servers(&self) -> Vec<String> {
        self.config.mcp_servers.keys().cloned().collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
