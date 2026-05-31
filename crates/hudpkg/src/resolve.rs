use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{DependencyValue, LocalPackageManifest, MANIFEST, PACKAGES_SOURCE_DIR};

/// A resolved dependency with its source path and version.
#[derive(Debug, Clone)]
pub(crate) struct Resolved {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source_path: PathBuf,
    pub(crate) dependencies: Vec<String>,
}

/// Resolve all dependencies from the local `packages/` directory.
/// Handles transitive dependencies via BFS.
pub(crate) fn resolve_all(
    deps: &BTreeMap<String, DependencyValue>,
) -> Result<BTreeMap<String, Resolved>, String> {
    let mut resolved: BTreeMap<String, Resolved> = BTreeMap::new();
    let mut queue: Vec<(String, DependencyValue)> =
        deps.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    while let Some((name, dep_val)) = queue.pop() {
        if resolved.contains_key(&name) {
            continue;
        }

        let r = resolve_single(&name, &dep_val)?;

        for transitive_name in &r.dependencies {
            if !resolved.contains_key(transitive_name) {
                queue.push((
                    transitive_name.clone(),
                    DependencyValue::Simple("*".to_string()),
                ));
            }
        }

        resolved.insert(name, r);
    }

    // Check for circular dependencies (simple cycle detection)
    for (name, r) in &resolved {
        for dep in &r.dependencies {
            if let Some(dep_resolved) = resolved.get(dep) {
                if dep_resolved.dependencies.contains(name) {
                    return Err(format!(
                        "Circular dependency detected: {} <-> {}",
                        name, dep
                    ));
                }
            }
        }
    }

    Ok(resolved)
}

/// Resolve a single package from the local filesystem.
///
/// Search order:
/// 1. If `path` override is given in the dependency spec, use that directory.
/// 2. Look in `packages/<name>/` for a directory with `hudhud.toml`.
/// 3. Look in `packages/` for directories matching `<name>-<version>/`.
pub(crate) fn resolve_single(name: &str, dep: &DependencyValue) -> Result<Resolved, String> {
    let version_constraint = dep.version_str();
    let version_req = parse_version_req(version_constraint)?;

    // 1. Path override
    if let Some(path_str) = dep.path_override() {
        let path = PathBuf::from(path_str);
        return resolve_from_path(name, &path, &version_req);
    }

    let packages_dir = PathBuf::from(PACKAGES_SOURCE_DIR);
    if !packages_dir.is_dir() {
        return Err(format!(
            "Package '{}' not found: no '{}' directory exists. \
             Create a packages/ directory with your local packages.",
            name, PACKAGES_SOURCE_DIR
        ));
    }

    // 2. Direct directory: packages/<name>/
    let direct_path = packages_dir.join(name);
    if direct_path.is_dir() {
        return resolve_from_path(name, &direct_path, &version_req);
    }

    // 3. Versioned directories: packages/<name>-<version>/
    let mut candidates: Vec<(Version, PathBuf)> = Vec::new();
    let entries = fs::read_dir(&packages_dir)
        .map_err(|e| format!("Failed to read packages directory: {}", e))?;

    let prefix = format!("{}-", name);
    for entry in entries.flatten() {
        let entry_name = entry.file_name().to_string_lossy().to_string();
        if entry_name.starts_with(&prefix) && entry.path().is_dir() {
            let version_str = &entry_name[prefix.len()..];
            if let Ok(ver) = Version::parse(version_str) {
                if version_req.matches(&ver) {
                    candidates.push((ver, entry.path()));
                }
            }
        }
    }

    if candidates.is_empty() {
        return Err(format!(
            "Package '{}' (version {}) not found in {}. \
             Expected a directory at packages/{} or packages/{}-<version>.",
            name, version_constraint, PACKAGES_SOURCE_DIR, name, name
        ));
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    let (_, best_path) = candidates.last().unwrap();
    resolve_from_path(name, best_path, &version_req)
}

/// Read package metadata from a directory path and validate version constraint.
pub(crate) fn resolve_from_path(
    name: &str,
    path: &Path,
    version_req: &VersionReq,
) -> Result<Resolved, String> {
    if !path.is_dir() {
        return Err(format!(
            "Package '{}': path '{}' is not a directory",
            name,
            path.display()
        ));
    }

    let manifest_path = path.join(MANIFEST);
    let (version, transitive_deps) = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            format!(
                "Failed to read {} for package '{}': {}",
                manifest_path.display(),
                name,
                e
            )
        })?;
        let pkg_manifest: LocalPackageManifest = toml::from_str(&content).map_err(|e| {
            format!(
                "Failed to parse {} for package '{}': {}",
                manifest_path.display(),
                name,
                e
            )
        })?;

        let deps: Vec<String> = pkg_manifest.dependencies.keys().cloned().collect();
        (pkg_manifest.package.version, deps)
    } else {
        ("0.0.0".to_string(), vec![])
    };

    if let Ok(ver) = Version::parse(&version) {
        if !version_req.matches(&ver) {
            return Err(format!(
                "Package '{}' at {} has version {} which does not satisfy constraint '{}'",
                name,
                path.display(),
                version,
                version_req
            ));
        }
    }

    Ok(Resolved {
        name: name.to_string(),
        version,
        source_path: path.to_path_buf(),
        dependencies: transitive_deps,
    })
}

/// Parse a version constraint string.
/// Accepts: "^1.0", "~2.3", ">=1.0.0, <2.0.0", "latest", "*", "0.1"
pub(crate) fn parse_version_req(constraint: &str) -> Result<VersionReq, String> {
    let trimmed = constraint.trim();
    if trimmed.is_empty() || trimmed == "latest" || trimmed == "*" {
        return Ok(VersionReq::STAR);
    }
    VersionReq::parse(trimmed)
        .map_err(|e| format!("Invalid version constraint '{}': {}", trimmed, e))
}
