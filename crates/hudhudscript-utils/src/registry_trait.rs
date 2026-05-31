//! Unified Registry trait for HudHudScript (#820)
//!
//! This trait defines a common interface for all registry implementations
//! across the workspace (tool registry, package registry, resource registry,
//! native module registry, provider registry, actor registry, etc.)
//!
//! ## Design Principles
//!
//! 1. **Generic**: Works with any item type
//! 2. **Observable**: Registry state is always inspectable
//! 3. **Thread-safe**: Implementations should be `Send + Sync` where needed
//! 4. **Consistent**: register/unregister/get/list for all registries

/// Unified registry trait for synchronous registries.
///
/// All registry implementations in the workspace should implement this trait
/// to provide a consistent interface for registration, lookup, and listing.
///
/// # Type Parameters
///
/// * `K` - The key type (typically `String`)
/// * `V` - The value type (the registered item)
pub trait Registry<K, V> {
    /// Error type for registry operations.
    type Error;

    /// Register an item in the registry.
    ///
    /// If an item with the same key already exists, the behavior depends
    /// on the implementation (overwrite, error, or merge).
    fn register(&mut self, key: K, value: V) -> Result<(), Self::Error>;

    /// Unregister an item from the registry.
    ///
    /// Returns `Ok(Some(V))` if the item was found and removed,
    /// `Ok(None)` if the item was not found.
    fn unregister(&mut self, key: &K) -> Result<Option<V>, Self::Error>;

    /// Look up a registered item by key.
    fn get(&self, key: &K) -> Option<&V>;

    /// Check if an item is registered.
    fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// List all registered keys.
    fn keys(&self) -> Vec<&K>;

    /// Number of registered items.
    fn len(&self) -> usize;

    /// Whether the registry is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A registry that also supports searching/filtering.
pub trait SearchableRegistry<K, V>: Registry<K, V> {
    /// Search for items matching a query string.
    ///
    /// The implementation defines what "matching" means (name prefix,
    /// fuzzy match, tag filter, etc.)
    fn search(&self, query: &str) -> Vec<(&K, &V)>;
}
