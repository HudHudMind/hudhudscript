use crate::{DependencySpec, PackageError, RegistryClient, Result};
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct DependencyResolver {
    registry: RegistryClient,
}

impl DependencyResolver {
    pub fn new(registry: RegistryClient) -> Self {
        Self { registry }
    }

    pub async fn resolve(
        &self,
        deps: &HashMap<String, DependencySpec>,
    ) -> Result<Vec<ResolvedDependency>> {
        let mut resolved: HashMap<String, ResolvedDependency> = HashMap::new();
        let mut in_progress: HashSet<String> = HashSet::new();

        for (name, spec) in deps {
            self.resolve_single(name, spec, &mut resolved, &mut in_progress)
                .await?;
        }

        topological_sort(&resolved)
    }

    /// Recursively resolve a single dependency and its transitive deps.
    fn resolve_single<'a>(
        &'a self,
        name: &'a str,
        spec: &'a DependencySpec,
        resolved: &'a mut HashMap<String, ResolvedDependency>,
        in_progress: &'a mut HashSet<String>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(self.resolve_single_inner(name, spec, resolved, in_progress))
    }

    async fn resolve_single_inner(
        &self,
        name: &str,
        spec: &DependencySpec,
        resolved: &mut HashMap<String, ResolvedDependency>,
        in_progress: &mut HashSet<String>,
    ) -> Result<()> {
        if resolved.contains_key(name) {
            return Ok(());
        }

        if in_progress.contains(name) {
            return Err(PackageError::DependencyResolution(format!(
                "Circular dependency detected involving '{}'",
                name
            )));
        }

        in_progress.insert(name.to_string());

        let constraint_str = spec.version();
        let version_req = parse_version_req(constraint_str)?;

        let metadata = self
            .registry
            .get_metadata(name)
            .await
            .map_err(|e| PackageError::PackageNotFound(format!("{}: {}", name, e)))?;

        let best_version =
            select_best_version(&metadata.versions, &version_req).ok_or_else(|| {
                PackageError::VersionNotFound(format!(
                    "No version of '{}' satisfies constraint '{}'",
                    name, constraint_str
                ))
            })?;

        let version_info = self
            .registry
            .get_version(name, &best_version)
            .await
            .map_err(|e| {
                PackageError::VersionNotFound(format!("{}@{}: {}", name, best_version, e))
            })?;

        let dep_names: Vec<String> = version_info.dependencies.keys().cloned().collect();

        resolved.insert(
            name.to_string(),
            ResolvedDependency {
                name: name.to_string(),
                version: best_version.clone(),
                dependencies: dep_names.clone(),
                resolved_path: None, // Will be set by download step
            },
        );

        for (dep_name, dep_version_str) in &version_info.dependencies {
            let transitive_spec = DependencySpec::Simple(dep_version_str.clone());
            self.resolve_single(dep_name, &transitive_spec, resolved, in_progress)
                .await?;
        }

        in_progress.remove(name);
        Ok(())
    }
}

/// Parse a version constraint string like "^1.0", "~2.3", ">=1.0.0, <2.0.0",
/// or "latest"/"*" (match anything).
pub fn parse_version_req(constraint: &str) -> Result<VersionReq> {
    let trimmed = constraint.trim();
    if trimmed == "latest" || trimmed == "*" {
        return Ok(VersionReq::STAR);
    }
    VersionReq::parse(trimmed).map_err(|e| {
        PackageError::InvalidVersion(format!("Invalid version constraint '{}': {}", trimmed, e))
    })
}

/// Select the highest version from `versions` that satisfies `req`.
pub fn select_best_version(versions: &[String], req: &VersionReq) -> Option<String> {
    let mut matching: Vec<Version> = versions
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| req.matches(v))
        .collect();
    matching.sort();
    matching.last().map(|v| v.to_string())
}

/// Topological sort via Kahn's algorithm.
pub fn topological_sort(
    graph: &HashMap<String, ResolvedDependency>,
) -> Result<Vec<ResolvedDependency>> {
    // Build reverse adjacency: for each package, track which packages depend on it
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();

    for name in graph.keys() {
        in_degree.entry(name.as_str()).or_insert(0);
        dependents.entry(name.as_str()).or_default();
    }

    // If A depends on B, A has in-degree +1 and B is a "provider" for A
    for dep in graph.values() {
        for child in &dep.dependencies {
            if graph.contains_key(child) {
                *in_degree.entry(dep.name.as_str()).or_insert(0) += 1;
                dependents
                    .entry(child.as_str())
                    .or_default()
                    .push(dep.name.as_str());
            }
        }
    }

    // Start with packages that have no dependencies (in-degree 0)
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect();

    let mut sorted: Vec<ResolvedDependency> = Vec::with_capacity(graph.len());

    while let Some(name) = queue.pop_front() {
        if let Some(dep) = graph.get(name) {
            sorted.push(dep.clone());
            // For each package that depends on this one, decrease its in-degree
            if let Some(deps) = dependents.get(name) {
                for dependent in deps {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }
    }

    if sorted.len() != graph.len() {
        return Err(PackageError::DependencyResolution(
            "Cycle detected during topological sort".to_string(),
        ));
    }

    Ok(sorted)
}

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    /// Path to the resolved source (local dir or .hudpkg archive).
    pub resolved_path: Option<std::path::PathBuf>,
}
