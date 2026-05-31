//! Package installation — install, add, remove, update.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;

use crate::{DependencySpec, HudhudConfig, PackageError, ResolvedDependency, Result};

use super::Installer;

impl Installer {
    /// Install all resolved dependencies into the local `.hudpackages/` directory.
    pub async fn install_all(&self, deps: &[ResolvedDependency]) -> Result<()> {
        println!("{} Installing {} packages...", ">>".blue(), deps.len());

        let pb = ProgressBar::new(deps.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap(),
        );

        for dep in deps {
            pb.set_message(format!("Installing {}", dep.name));
            self.install_package(&dep.name, &dep.version).await?;
            pb.inc(1);
        }

        pb.finish_with_message("Done!");
        println!("{} All packages installed successfully!", ">>".green());
        Ok(())
    }

    /// Install a single package into the global cache.
    async fn install_package(&self, name: &str, version: &str) -> Result<()> {
        if self.cache.is_installed(name, version)? {
            return Ok(());
        }

        let tarball = self.download_package(name, version).await?;

        self.security
            .verify_checksum(&tarball, name, version)
            .await?;

        self.cache.install(name, version, &tarball)?;

        Ok(())
    }

    /// Download a package tarball from the registry.
    async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        if let Some(ref registry) = self.registry {
            let data = registry.download(name, version).await.map_err(|e| {
                PackageError::PackageNotFound(format!(
                    "Failed to download {}@{}: {}",
                    name, version, e
                ))
            })?;
            Ok(data)
        } else {
            Ok(vec![])
        }
    }

    /// Add new package
    pub async fn add_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        println!("{} Adding package: {}", ">>".blue(), package.bold());

        let version = version.unwrap_or("latest");
        self.install_package(package, version).await?;

        let config_path = "hudhud.toml";
        if std::path::Path::new(config_path).exists() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(mut doc) = content.parse::<toml::Table>() {
                    let deps = doc
                        .entry("dependencies")
                        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
                    if let toml::Value::Table(deps_table) = deps {
                        deps_table.insert(
                            package.to_string(),
                            toml::Value::String(format!("^{}", version)),
                        );
                    }
                    if let Ok(updated) = toml::to_string_pretty(&doc) {
                        let _ = std::fs::write(config_path, updated);
                    }
                }
            }
        }

        println!("{} Package added successfully!", ">>".green());
        Ok(())
    }

    /// Remove package
    pub async fn remove_package(&self, package: &str) -> Result<()> {
        println!("{} Removing package: {}", ">>".blue(), package.bold());

        self.cache.remove(package)?;

        let config_path = "hudhud.toml";
        if std::path::Path::new(config_path).exists() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(mut doc) = content.parse::<toml::Table>() {
                    if let Some(toml::Value::Table(deps_table)) = doc.get_mut("dependencies") {
                        deps_table.remove(package);
                    }
                    if let Ok(updated) = toml::to_string_pretty(&doc) {
                        let _ = std::fs::write(config_path, updated);
                    }
                }
            }
        }

        println!("{} Package removed successfully!", ">>".green());
        Ok(())
    }

    /// Update all packages to the newest versions matching their constraints.
    pub async fn update_all(&self) -> Result<()> {
        println!("{} Updating all packages...", ">>".blue());

        let config_path = "hudhud.toml";
        let deps: HashMap<String, DependencySpec> = if std::path::Path::new(config_path).exists() {
            let config = HudhudConfig::load(config_path).map_err(PackageError::Other)?;
            config.dependencies
        } else {
            HashMap::new()
        };

        if deps.is_empty() {
            println!("{} No dependencies to update.", ">>".yellow());
            return Ok(());
        }

        if let Some(ref resolver) = self.resolver {
            let resolved = resolver.resolve(&deps).await?;

            let mut updated_count = 0u32;
            for dep in &resolved {
                if !self.cache.is_installed(&dep.name, &dep.version)? {
                    println!(
                        "  {} Updating {} to {}",
                        ">>".cyan(),
                        dep.name.bold(),
                        dep.version
                    );
                    self.install_package(&dep.name, &dep.version).await?;
                    updated_count += 1;
                }
            }

            if updated_count == 0 {
                println!("{} All packages are already up to date.", ">>".green());
            } else {
                println!("{} Updated {} package(s).", ">>".green(), updated_count);
            }
        } else {
            println!(
                "{} No registry configured; cannot check for updates.",
                ">>".yellow()
            );
        }

        println!("{} All packages updated successfully!", ">>".green());
        Ok(())
    }
}
