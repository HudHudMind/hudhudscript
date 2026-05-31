//! Update notification — detect available updates for installed packages.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::registry::RegistryClient;

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// An installed package tracked locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub install_date: DateTime<Utc>,
}

/// Information about an available update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub package_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub changelog: Option<String>,
}

impl UpdateInfo {
    /// Whether this is a major version bump.
    pub fn is_major_update(&self) -> bool {
        if let (Ok(current), Ok(latest)) = (
            Version::parse(&self.current_version),
            Version::parse(&self.latest_version),
        ) {
            latest.major > current.major
        } else {
            false
        }
    }

    /// Whether this is a minor (but not major) version bump.
    pub fn is_minor_update(&self) -> bool {
        if let (Ok(current), Ok(latest)) = (
            Version::parse(&self.current_version),
            Version::parse(&self.latest_version),
        ) {
            latest.major == current.major && latest.minor > current.minor
        } else {
            false
        }
    }

    /// Whether this is a patch-only version bump.
    pub fn is_patch_update(&self) -> bool {
        if let (Ok(current), Ok(latest)) = (
            Version::parse(&self.current_version),
            Version::parse(&self.latest_version),
        ) {
            latest.major == current.major
                && latest.minor == current.minor
                && latest.patch > current.patch
        } else {
            false
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Update checker
// ─────────────────────────────────────────────────────────────────────────────

/// Checks the registry for available updates to installed packages.
#[derive(Debug, Clone)]
pub struct UpdateChecker {
    registry: RegistryClient,
}

impl UpdateChecker {
    /// Create a new update checker backed by the given registry client.
    pub fn new(registry: RegistryClient) -> Self {
        Self { registry }
    }

    /// Check all installed packages for updates.
    ///
    /// Returns an `UpdateInfo` for each package whose latest registry version
    /// is newer than the installed version.  Packages that cannot be looked up
    /// (e.g. network errors) are silently skipped.
    pub async fn check_updates(&self, installed: &[InstalledPackage]) -> Vec<UpdateInfo> {
        let mut updates = Vec::new();

        for pkg in installed {
            if let Some(info) = self.check_single(pkg).await {
                updates.push(info);
            }
        }

        updates
    }

    /// Check a single installed package against the registry.
    async fn check_single(&self, pkg: &InstalledPackage) -> Option<UpdateInfo> {
        let metadata = self.registry.get_metadata(&pkg.name).await.ok()?;

        let current = Version::parse(&pkg.version).ok()?;
        let latest = Version::parse(&metadata.latest_version).ok()?;

        if latest > current {
            Some(UpdateInfo {
                package_name: pkg.name.clone(),
                current_version: pkg.version.clone(),
                latest_version: metadata.latest_version,
                changelog: None,
            })
        } else {
            None
        }
    }
}

/// Compare two version strings. Returns `true` when `latest` is strictly
/// newer than `current`.
pub fn is_newer(current: &str, latest: &str) -> bool {
    match (Version::parse(current), Version::parse(latest)) {
        (Ok(c), Ok(l)) => l.cmp_precedence(&c) == std::cmp::Ordering::Greater,
        _ => false,
    }
}
