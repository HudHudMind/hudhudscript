use crate::registry::PublishRequest;
use crate::security::calculate_checksum;
use crate::{HudhudConfig, PackageError, RegistryClient, Result, SecurityChecker};
use colored::Colorize;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[derive(Clone)]
pub struct Publisher {
    registry: RegistryClient,
    #[allow(dead_code)]
    security: SecurityChecker,
}

impl Publisher {
    pub fn new(registry: RegistryClient, security: SecurityChecker) -> Self {
        Self { registry, security }
    }

    /// Publish the package in the current directory to the registry.
    ///
    /// Steps:
    ///   1. Load hudhud.toml to get package metadata.
    ///   2. Optionally verify working tree cleanliness.
    ///   3. Create a gzipped tar archive of the package source.
    ///   4. Compute SHA-256 checksum.
    ///   5. Read auth token from ~/.hudhud/credentials.
    ///   6. If dry_run, print a summary and return.
    ///   7. Upload via registry client.
    pub async fn publish(&self, options: PublishOptions) -> Result<()> {
        println!("{} Preparing to publish...", ">>".blue());

        // 1. Load manifest
        let config_path = "hudhud.toml";
        if !Path::new(config_path).exists() {
            return Err(PackageError::Other(anyhow::anyhow!(
                "No hudhud.toml found in the current directory"
            )));
        }
        let config = HudhudConfig::load(config_path).map_err(PackageError::Other)?;

        let pkg = &config.package;

        // Validate required fields
        if pkg.name.is_empty() {
            return Err(PackageError::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }
        if pkg.version.is_empty() {
            return Err(PackageError::InvalidVersion(
                "Package version cannot be empty".to_string(),
            ));
        }

        // Validate version is valid semver
        if semver::Version::parse(&pkg.version).is_err() {
            return Err(PackageError::InvalidVersion(format!(
                "'{}' is not a valid semver version",
                pkg.version
            )));
        }

        // 2. Check working tree cleanliness (best-effort, requires git)
        if !options.allow_dirty {
            if let Ok(output) = std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    return Err(PackageError::Other(anyhow::anyhow!(
                        "Working tree has uncommitted changes. Use --allow-dirty to override."
                    )));
                }
            }
        }

        // 3. Create tarball
        println!("{} Creating package archive...", ">>".blue());
        let tarball = create_tarball(".")?;

        // 4. Compute checksum
        let checksum = calculate_checksum(&tarball);
        println!("{} Checksum: {}", ">>".blue(), &checksum[..16]);

        // 5. Read auth token
        let token = read_auth_token()?;

        // Build dependency map
        let dependencies: HashMap<String, String> = config
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.version().to_string()))
            .collect();

        println!(
            "{} Package: {}@{}",
            ">>".blue(),
            pkg.name.bold(),
            pkg.version
        );
        println!("{} Size: {} bytes", ">>".blue(), tarball.len());
        println!("{} Dependencies: {}", ">>".blue(), dependencies.len());

        // 6. Dry run
        if options.dry_run {
            println!(
                "{} Dry run complete. Package would be published as {}@{}",
                ">>".yellow(),
                pkg.name,
                pkg.version
            );
            return Ok(());
        }

        // 7. Upload
        println!("{} Uploading to registry...", ">>".blue());

        let request = PublishRequest {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            description: pkg.description.clone(),
            authors: pkg.authors.clone(),
            license: pkg.license.clone(),
            repository: if pkg.repository.is_empty() {
                None
            } else {
                Some(pkg.repository.clone())
            },
            homepage: if pkg.homepage.is_empty() {
                None
            } else {
                Some(pkg.homepage.clone())
            },
            keywords: pkg.keywords.clone(),
            categories: pkg.categories.clone(),
            dependencies,
            tarball,
            checksum,
        };

        let response = self
            .registry
            .publish(&request, &token)
            .await
            .map_err(PackageError::Other)?;

        if response.success {
            println!(
                "{} Published successfully! {}",
                ">>".green(),
                response.package_url
            );
        } else {
            return Err(PackageError::Other(anyhow::anyhow!(
                "Publish failed: {}",
                response.message
            )));
        }

        Ok(())
    }
}

/// Create a gzipped tar archive of the package directory, respecting
/// `.hudhudignore` if present.
pub fn create_tarball(dir: &str) -> Result<Vec<u8>> {
    let base = Path::new(dir);
    let ignore_patterns = load_ignore_patterns(base);

    let tar_data = {
        let mut builder = tar::Builder::new(Vec::new());
        add_directory_to_tar(&mut builder, base, base, &ignore_patterns)?;
        builder.into_inner().map_err(PackageError::Io)?
    };

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_data).map_err(PackageError::Io)?;
    let compressed = encoder.finish().map_err(PackageError::Io)?;

    Ok(compressed)
}

/// Recursively add files to the tar builder, skipping ignored paths.
fn add_directory_to_tar(
    builder: &mut tar::Builder<Vec<u8>>,
    base: &Path,
    current: &Path,
    ignore_patterns: &[String],
) -> Result<()> {
    let entries = std::fs::read_dir(current).map_err(PackageError::Io)?;

    for entry in entries {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let relative_str = relative.to_string_lossy();

        if should_ignore(&relative_str, ignore_patterns) {
            continue;
        }

        if path.is_dir() {
            add_directory_to_tar(builder, base, &path, ignore_patterns)?;
        } else if path.is_file() {
            builder
                .append_path_with_name(&path, relative)
                .map_err(PackageError::Io)?;
        }
    }

    Ok(())
}

/// Load ignore patterns from `.hudhudignore`, with sensible defaults.
pub fn load_ignore_patterns(base: &Path) -> Vec<String> {
    let ignore_file = base.join(".hudhudignore");
    let mut patterns = vec![
        ".git".to_string(),
        ".hudpackages".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".hudhudignore".to_string(),
    ];

    if let Ok(content) = std::fs::read_to_string(ignore_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                patterns.push(trimmed.to_string());
            }
        }
    }

    patterns
}

/// Check if a relative path should be ignored.
pub fn should_ignore(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if path == pattern.as_str()
            || path.starts_with(&format!("{}/", pattern))
            || path.contains(&format!("/{}/", pattern))
            || path.ends_with(&format!("/{}", pattern))
        {
            return true;
        }
    }
    false
}

/// Read the authentication token from `~/.hudhud/credentials`.
pub fn read_auth_token() -> Result<String> {
    let credentials_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".hudhud")
        .join("credentials");

    if !credentials_path.exists() {
        return Err(PackageError::Other(anyhow::anyhow!(
            "No credentials file found at {}. Run `hudp login` first.",
            credentials_path.display()
        )));
    }

    let content = std::fs::read_to_string(&credentials_path).map_err(PackageError::Io)?;

    let table: toml::Table = content
        .parse()
        .map_err(|_| PackageError::Other(anyhow::anyhow!("Invalid credentials file format")))?;

    table
        .get("token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            PackageError::Other(anyhow::anyhow!(
                "No 'token' field found in credentials file"
            ))
        })
}

#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    pub dry_run: bool,
    pub allow_dirty: bool,
}
