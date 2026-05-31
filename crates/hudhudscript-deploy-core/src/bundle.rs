//! Bundle `.hud` scripts with the HudHudScript runtime into a distributable layout.

use crate::DeployError;
use std::path::{Path, PathBuf};

/// A file entry (source on build host, destination relative to install prefix).
#[derive(Debug, Clone)]
pub struct BundleFile {
    pub source: PathBuf,
    pub dest: PathBuf,
}

/// Bundles `.hud` scripts, assets, and config templates for distribution.
#[derive(Debug, Clone)]
pub struct Bundle {
    pub name: String,
    pub version: String,
    /// Entry-point `.hud` script (relative path inside the bundle).
    pub entry_point: String,
    /// `.hud` script files.
    pub scripts: Vec<PathBuf>,
    /// Static assets (images, data, etc.).
    pub assets: Vec<BundleFile>,
    /// Configuration template files (installed to `/etc/hudhud/<name>/`).
    pub config_files: Vec<BundleFile>,
}

impl Bundle {
    /// Create a new bundle.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        entry_point: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            entry_point: entry_point.into(),
            scripts: Vec::new(),
            assets: Vec::new(),
            config_files: Vec::new(),
        }
    }

    /// Add a `.hud` script to the bundle.
    pub fn add_script(&mut self, path: impl Into<PathBuf>) {
        self.scripts.push(path.into());
    }

    /// Add a static asset with a destination path relative to the application share dir.
    pub fn add_asset(&mut self, source: impl Into<PathBuf>, dest: impl Into<PathBuf>) {
        self.assets.push(BundleFile {
            source: source.into(),
            dest: dest.into(),
        });
    }

    /// Add a config template; `dest` is relative to `/etc/hudhud/<name>/`.
    pub fn add_config_template(&mut self, source: impl Into<PathBuf>, dest: impl Into<PathBuf>) {
        self.config_files.push(BundleFile {
            source: source.into(),
            dest: dest.into(),
        });
    }

    /// Create the standard directory structure under `base_path`.
    ///
    /// Layout produced:
    /// ```text
    /// <base_path>/
    ///   usr/share/hudhud/<name>/scripts/
    ///   usr/share/hudhud/<name>/assets/
    ///   usr/share/hudhud/plugins/
    ///   etc/hudhud/<name>/
    ///   usr/bin/
    /// ```
    pub fn create_directory_structure(&self, base_path: &Path) -> Result<(), DeployError> {
        let dirs = [
            base_path.join(format!("usr/share/hudhud/{}/scripts", self.name)),
            base_path.join(format!("usr/share/hudhud/{}/assets", self.name)),
            base_path.join("usr/share/hudhud/plugins"),
            base_path.join(format!("etc/hudhud/{}", self.name)),
            base_path.join("usr/bin"),
        ];

        for dir in &dirs {
            std::fs::create_dir_all(dir).map_err(|e| {
                DeployError::BuildFailed(format!("cannot create directory {}: {e}", dir.display()))
            })?;
        }

        // Copy scripts
        for script in &self.scripts {
            let file_name = script
                .file_name()
                .ok_or_else(|| DeployError::BuildFailed("script has no filename".into()))?;
            let target = base_path
                .join(format!("usr/share/hudhud/{}/scripts", self.name))
                .join(file_name);
            std::fs::copy(script, &target).map_err(|e| {
                DeployError::BuildFailed(format!(
                    "cannot copy script {} -> {}: {e}",
                    script.display(),
                    target.display()
                ))
            })?;
        }

        // Copy assets
        for asset in &self.assets {
            let target = base_path
                .join(format!("usr/share/hudhud/{}/assets", self.name))
                .join(&asset.dest);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DeployError::BuildFailed(format!("cannot create asset dir: {e}"))
                })?;
            }
            std::fs::copy(&asset.source, &target).map_err(|e| {
                DeployError::BuildFailed(format!(
                    "cannot copy asset {} -> {}: {e}",
                    asset.source.display(),
                    target.display()
                ))
            })?;
        }

        // Copy config templates
        for cfg in &self.config_files {
            let target = base_path
                .join(format!("etc/hudhud/{}", self.name))
                .join(&cfg.dest);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DeployError::BuildFailed(format!("cannot create config dir: {e}"))
                })?;
            }
            std::fs::copy(&cfg.source, &target).map_err(|e| {
                DeployError::BuildFailed(format!(
                    "cannot copy config {} -> {}: {e}",
                    cfg.source.display(),
                    target.display()
                ))
            })?;
        }

        // Write wrapper script
        let wrapper = self.generate_wrapper_script();
        let wrapper_path = base_path.join(format!("usr/bin/{}", self.name));
        std::fs::write(&wrapper_path, &wrapper).map_err(|e| {
            DeployError::BuildFailed(format!(
                "cannot write wrapper script {}: {e}",
                wrapper_path.display()
            ))
        })?;

        Ok(())
    }

    /// Generate a shell wrapper that invokes the HudHudScript runtime with the
    /// bundle's entry point.
    pub fn generate_wrapper_script(&self) -> String {
        format!(
            r#"#!/bin/sh
# Auto-generated wrapper for HudHudScript bundle: {} v{}
set -e

HUDHUD_HOME="/usr/share/hudhud/{}"
HUDHUD_CONFIG="/etc/hudhud/{}"
export HUDHUD_HOME HUDHUD_CONFIG

exec hudhudscript "$HUDHUD_HOME/scripts/{}" "$@"
"#,
            self.name, self.version, self.name, self.name, self.entry_point
        )
    }
}
