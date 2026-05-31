//! Installer core — struct and constructors.

use crate::{PackageCache, SecurityChecker};

/// Package installer
#[derive(Clone)]
pub struct Installer {
    pub(crate) cache: PackageCache,
    pub(crate) security: SecurityChecker,
    pub(crate) registry: Option<crate::RegistryClient>,
    pub(crate) resolver: Option<crate::resolver::DependencyResolver>,
}

impl Installer {
    pub fn new(cache: PackageCache, security: SecurityChecker) -> Self {
        Self {
            cache,
            security,
            registry: None,
            resolver: None,
        }
    }

    /// Create an installer with registry and resolver support.
    pub fn with_registry(
        cache: PackageCache,
        security: SecurityChecker,
        registry: crate::RegistryClient,
        resolver: crate::resolver::DependencyResolver,
    ) -> Self {
        Self {
            cache,
            security,
            registry: Some(registry),
            resolver: Some(resolver),
        }
    }
}
