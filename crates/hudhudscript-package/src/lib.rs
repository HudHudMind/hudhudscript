// ═══════════════════════════════════════════════════════════════════════════
// HudHudScript Package Management System
// ═══════════════════════════════════════════════════════════════════════════

pub mod builder;
pub mod cache;
pub mod config;
pub mod error;
pub mod installer;
pub mod marketplace;
pub mod publisher;
pub mod rating;
pub mod registry;
pub mod resolver;
pub mod runner;
pub mod security;
pub mod signature;
pub mod update;

pub use builder::{BuildResult, PackageBuilder};
pub use cache::PackageCache;
pub use config::{
    AiProviderConfig, DependencySpec, HudhudConfig, McpServerSpec, NativeDependencySpec,
    PackageConfig, PackageType,
};
pub use error::{PackageError, Result};
pub use installer::{InstallOptions, Installer};
pub use marketplace::{
    default_page, default_per_page, Category, Marketplace, MarketplaceEntry, SearchQuery,
    SearchResult, SortBy,
};
pub use publisher::{
    create_tarball, load_ignore_patterns, read_auth_token, should_ignore, PublishOptions, Publisher,
};
pub use rating::{Rating, RatingStore};
pub use registry::{
    PackageMetadata, PackageStats, PublishRequest, PublishResponse, Registry, RegistryClient,
    VersionInfo,
};
pub use resolver::{
    parse_version_req, select_best_version, topological_sort, DependencyResolver,
    ResolvedDependency,
};
pub use runner::{
    collect_modules, find_entry_point, get_exported_modules, is_application_package, PackageRunner,
};
pub use security::calculate_checksum;
pub use security::{Advisory, Checksum, SecurityChecker};
pub use signature::{
    compute_signature, enforce_policy, generate_keypair, verify_signature, PackageSignature,
    SignatureAlgorithm, SignaturePolicy,
};
pub use update::{InstalledPackage, UpdateChecker, UpdateInfo};

/// Package manager version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default registry URL
pub const DEFAULT_REGISTRY: &str = "https://registry.hudhudscript.org";

/// Default directory for installed packages (local to the project)
pub const PACKAGES_DIR: &str = ".hudpackages";

/// Package manager instance
pub struct PackageManager {
    config: HudhudConfig,
    registry: RegistryClient,
    installer: Installer,
    publisher: Publisher,
    resolver: DependencyResolver,
    cache: PackageCache,
    security: SecurityChecker,
}

impl PackageManager {
    /// Create new package manager
    pub fn new(config: HudhudConfig) -> Result<Self> {
        let registry = RegistryClient::new(&config.registry_url)?;
        let cache = PackageCache::new(&config.cache_dir)?;
        let security = SecurityChecker::new();
        let installer = Installer::new(cache.clone(), security.clone());
        let publisher = Publisher::new(registry.clone(), security.clone());
        let resolver = DependencyResolver::new(registry.clone());

        Ok(Self {
            config,
            registry,
            installer,
            publisher,
            resolver,
            cache,
            security,
        })
    }

    /// Initialize new project
    pub async fn init(&self, name: &str, path: &str) -> Result<()> {
        self.installer.init_project(name, path, false).await
    }

    /// Initialize new project with --library flag
    pub async fn init_library(&self, name: &str, path: &str) -> Result<()> {
        self.installer.init_project(name, path, true).await
    }

    /// Install all dependencies
    pub async fn install(&self) -> Result<()> {
        let deps = self.config.dependencies.clone();
        let resolved = self.resolver.resolve(&deps).await?;
        self.installer.install_all(&resolved).await
    }

    /// Add new dependency
    pub async fn add(&self, package: &str, version: Option<&str>) -> Result<()> {
        self.installer.add_package(package, version).await
    }

    /// Remove dependency
    pub async fn remove(&self, package: &str) -> Result<()> {
        self.installer.remove_package(package).await
    }

    /// Update dependencies
    pub async fn update(&self) -> Result<()> {
        self.installer.update_all().await
    }

    /// Search packages
    pub async fn search(&self, query: &str) -> Result<Vec<PackageMetadata>> {
        self.registry
            .search(query)
            .await
            .map_err(PackageError::Other)
    }

    /// Get package info
    pub async fn info(&self, package: &str) -> Result<PackageMetadata> {
        self.registry
            .get_metadata(package)
            .await
            .map_err(PackageError::Other)
    }

    /// List installed packages
    pub fn list(&self) -> Result<Vec<String>> {
        self.cache.list_installed()
    }

    /// Publish package
    pub async fn publish(&self, options: PublishOptions) -> Result<()> {
        self.publisher.publish(options).await
    }

    /// Audit dependencies for vulnerabilities
    pub async fn audit(&self) -> Result<Vec<String>> {
        self.security
            .audit_dependencies(&self.config.dependencies)
            .await
    }
}
