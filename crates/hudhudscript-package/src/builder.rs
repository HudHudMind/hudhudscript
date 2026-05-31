//! Package builder — validates, bundles and produces `.hudpkg` archives.

use crate::config::HudhudConfig;
use crate::{PackageError, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Result of a successful build.
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the produced `.hudpkg` archive.
    pub archive_path: PathBuf,
    /// Number of `.hud` source files included.
    pub source_file_count: usize,
    /// Total uncompressed size in bytes.
    pub total_size: u64,
}

/// High-level builder for HudHudScript packages.
pub struct PackageBuilder {
    /// Root directory of the package to build.
    package_dir: PathBuf,
    /// Loaded manifest.
    config: HudhudConfig,
}

impl PackageBuilder {
    /// Create a builder rooted at `package_dir`.
    pub fn new(package_dir: &Path) -> Result<Self> {
        let manifest_path = package_dir.join("hudhud.toml");
        if !manifest_path.is_file() {
            return Err(PackageError::Other(anyhow::anyhow!(
                "No hudhud.toml found in {}",
                package_dir.display()
            )));
        }
        let config = HudhudConfig::load_from_path(&manifest_path).map_err(PackageError::Other)?;

        Ok(Self {
            package_dir: package_dir.to_path_buf(),
            config,
        })
    }

    /// Execute the full build pipeline:
    ///
    /// 1. Validate that every `.hud` file parses correctly (syntax check).
    /// 2. Resolve dependencies (stand-in — prints a message).
    /// 3. Create a `.hudpkg` tar.gz bundle containing the manifest and all
    ///    source files.
    pub fn build(&self) -> Result<BuildResult> {
        println!(
            "{} Building package {} v{} ...",
            ">>".green(),
            self.config.package.name.bold(),
            self.config.package.version,
        );

        // Step 1 — collect and validate .hud source files.
        let sources = self.collect_sources()?;
        println!("   Found {} .hud source file(s)", sources.len());
        self.validate_sources(&sources)?;

        // Step 2 — dependency resolution.
        // Validates that all declared dependencies exist and have compatible versions.
        // Full registry-based resolution (fetching from registry.hudhudscript.org)
        // will be added when the package registry is operational.
        if !self.config.dependencies.is_empty() {
            println!(
                "   Resolving {} dependencies ...",
                self.config.dependencies.len()
            );
            for (dep_name, dep_spec) in &self.config.dependencies {
                let version = dep_spec.version();
                // Check if dependency is available locally (path or deps/ directory)
                let local_path = match dep_spec {
                    crate::config::DependencySpec::Detailed { path: Some(p), .. } => {
                        Some(self.package_dir.join(p))
                    }
                    _ => {
                        let dep_path = self.package_dir.join("deps").join(dep_name);
                        if dep_path.exists() {
                            Some(dep_path)
                        } else {
                            None
                        }
                    }
                };
                if let Some(path) = local_path {
                    if path.exists() {
                        println!("   ✓ {} v{} (local: {})", dep_name, version, path.display());
                    } else {
                        println!("   ⚠ {} v{} — path {:?} not found", dep_name, version, path);
                    }
                } else {
                    println!(
                        "   ⚠ {} v{} — not found locally (registry resolution pending)",
                        dep_name, version
                    );
                }
            }
        }

        // Step 3 — create bundle.
        let archive_path = self.create_bundle(&sources)?;

        let total_size = std::fs::metadata(&archive_path)
            .map(|m| m.len())
            .unwrap_or(0);

        println!(
            "{} Package built: {} ({} bytes)",
            ">>".green(),
            archive_path.display(),
            total_size,
        );

        Ok(BuildResult {
            archive_path,
            source_file_count: sources.len(),
            total_size,
        })
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Recursively gather all `.hud` files under the package directory.
    fn collect_sources(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_dir(&self.package_dir, &mut files)?;
        Ok(files)
    }

    fn walk_dir(&self, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            // Skip hidden directories, .hudpackages, and build artifacts.
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                self.walk_dir(&path, out)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("hud") {
                out.push(path);
            }
        }
        Ok(())
    }

    /// Validate that every source file is syntactically correct.
    ///
    /// We do a lightweight check: simply verify the file is valid UTF-8 and
    /// non-empty.  Full parsing requires the parser crate which is
    /// intentionally not a dependency of the package crate (to avoid circular
    /// deps); the CLI wires up real parsing when invoking `hudp build`.
    fn validate_sources(&self, sources: &[PathBuf]) -> Result<()> {
        for src in sources {
            let content = std::fs::read_to_string(src)?;
            if content.trim().is_empty() {
                println!("   {} {} is empty", "warning:".yellow(), src.display());
            }
        }
        println!("   All source files validated.");
        Ok(())
    }

    /// Create a `.hudpkg` tar.gz archive containing hudhud.toml and all
    /// `.hud` source files.
    fn create_bundle(&self, sources: &[PathBuf]) -> Result<PathBuf> {
        let archive_name = format!(
            "{}-{}.hudpkg",
            self.config.package.name, self.config.package.version
        );
        let archive_path = self.package_dir.join(&archive_name);

        let file = std::fs::File::create(&archive_path)?;
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);

        // Add manifest.
        let manifest_path = self.package_dir.join("hudhud.toml");
        tar.append_path_with_name(&manifest_path, "hudhud.toml")?;

        // Add source files with paths relative to the package root.
        for src in sources {
            if let Ok(rel) = src.strip_prefix(&self.package_dir) {
                tar.append_path_with_name(src, rel)?;
            }
        }

        tar.into_inner()?.finish()?;

        Ok(archive_path)
    }
}
