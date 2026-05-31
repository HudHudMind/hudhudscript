//! Constitution management module
//!
//! This module provides functions for creating, modifying, and managing constitutions
//! with versioning support, including version queries and rollback capabilities.

use crate::types::{Constitution, ConstitutionId, Law, LawId};
use chrono::Utc;
use std::collections::HashMap;

/// Result type for constitution operations
pub type ConstitutionResult<T> = Result<T, ConstitutionError>;

/// Errors that can occur during constitution operations
#[derive(Debug, Clone, PartialEq)]
pub enum ConstitutionError {
    /// Constitution not found
    NotFound(ConstitutionId),
    /// Invalid version number
    InvalidVersion(u32),
    /// Law not found in constitution
    LawNotFound(LawId),
    /// No previous version available for rollback
    NoPreviousVersion,
}

impl ConstitutionError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ConstitutionError::NotFound(..) => hudhudscript_errors::ErrorCode::ConstitutionNotFound,
            ConstitutionError::InvalidVersion(..) => {
                hudhudscript_errors::ErrorCode::ConstitutionInvalidVersion
            }
            ConstitutionError::LawNotFound(..) => {
                hudhudscript_errors::ErrorCode::ConstitutionLawNotFound
            }
            ConstitutionError::NoPreviousVersion => {
                hudhudscript_errors::ErrorCode::ConstitutionNoPreviousVersion
            }
        }
    }
}

impl std::fmt::Display for ConstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ConstitutionError::NotFound(id) => write!(f, "Constitution not found: {}", id),
            ConstitutionError::InvalidVersion(v) => write!(f, "Invalid version number: {}", v),
            ConstitutionError::LawNotFound(id) => write!(f, "Law not found: {}", id),
            ConstitutionError::NoPreviousVersion => write!(f, "No previous version available"),
        }
    }
}

impl std::error::Error for ConstitutionError {}

/// Constitution version history entry
#[derive(Debug, Clone, PartialEq)]
pub struct ConstitutionVersion {
    pub constitution: Constitution,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Constitution manager with version history
#[derive(Debug, Clone)]
pub struct ConstitutionManager {
    /// Current constitution
    current: Constitution,
    /// Version history (version number -> constitution snapshot)
    history: HashMap<u32, ConstitutionVersion>,
}

impl ConstitutionManager {
    /// Create a new constitution with version 1
    ///
    /// # Arguments
    /// * `id` - Unique constitution identifier (e.g., "cons.1")
    /// * `name` - Human-readable constitution name
    /// * `description` - Optional description
    ///
    /// # Returns
    /// * New `ConstitutionManager` with version 1
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_governance::constitution::ConstitutionManager;
    ///
    /// let manager = ConstitutionManager::new(
    ///     "cons.1".to_string(),
    ///     "Test Constitution".to_string(),
    ///     Some("A test constitution".to_string()),
    /// );
    ///
    /// assert_eq!(manager.current_version(), 1);
    /// ```
    pub fn new(id: ConstitutionId, name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        let constitution = Constitution {
            id,
            name,
            description,
            laws: HashMap::new(),
            created_at: now,
            version: 1,
        };

        let mut history = HashMap::new();
        history.insert(
            1,
            ConstitutionVersion {
                constitution: constitution.clone(),
                modified_at: now,
            },
        );

        Self {
            current: constitution,
            history,
        }
    }

    /// Get the current constitution
    pub fn current(&self) -> &Constitution {
        &self.current
    }

    /// Get the current version number
    pub fn current_version(&self) -> u32 {
        self.current.version
    }

    /// Get a specific version of the constitution
    ///
    /// # Arguments
    /// * `version` - Version number to retrieve
    ///
    /// # Returns
    /// * `Some(&Constitution)` if version exists, `None` otherwise
    pub fn get_version(&self, version: u32) -> Option<&Constitution> {
        self.history.get(&version).map(|v| &v.constitution)
    }

    /// Get all available version numbers
    pub fn available_versions(&self) -> Vec<u32> {
        let mut versions: Vec<u32> = self.history.keys().copied().collect();
        versions.sort();
        versions
    }

    /// Modify the constitution (increments version)
    ///
    /// This creates a new version of the constitution with the provided modifications.
    ///
    /// # Arguments
    /// * `modifier` - Function that modifies the constitution
    ///
    /// # Returns
    /// * New version number
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_governance::constitution::ConstitutionManager;
    ///
    /// let mut manager = ConstitutionManager::new(
    ///     "cons.1".to_string(),
    ///     "Test Constitution".to_string(),
    ///     None,
    /// );
    ///
    /// let new_version = manager.modify(|constitution| {
    ///     constitution.name = "Updated Constitution".to_string();
    /// });
    ///
    /// assert_eq!(new_version, 2);
    /// assert_eq!(manager.current().name, "Updated Constitution");
    /// ```
    pub fn modify<F>(&mut self, modifier: F) -> u32
    where
        F: FnOnce(&mut Constitution),
    {
        // Clone current constitution
        let mut new_constitution = self.current.clone();

        // Increment version
        new_constitution.version += 1;

        // Apply modifications
        modifier(&mut new_constitution);

        // Store in history
        let now = Utc::now();
        self.history.insert(
            new_constitution.version,
            ConstitutionVersion {
                constitution: new_constitution.clone(),
                modified_at: now,
            },
        );

        // Update current
        self.current = new_constitution;

        self.current.version
    }

    /// Add a law to the constitution
    ///
    /// This is a convenience method that modifies the constitution to add a law.
    ///
    /// # Arguments
    /// * `law` - Law to add
    ///
    /// # Returns
    /// * New version number
    pub fn add_law(&mut self, law: Law) -> u32 {
        let law_id = law.id.clone();
        self.modify(|constitution| {
            constitution.laws.insert(law_id, law);
        })
    }

    /// Remove a law from the constitution
    ///
    /// # Arguments
    /// * `law_id` - ID of the law to remove
    ///
    /// # Returns
    /// * `Ok(version)` with new version number if law was found and removed
    /// * `Err(ConstitutionError::LawNotFound)` if law doesn't exist
    pub fn remove_law(&mut self, law_id: &LawId) -> ConstitutionResult<u32> {
        if !self.current.laws.contains_key(law_id) {
            return Err(ConstitutionError::LawNotFound(law_id.clone()));
        }

        let law_id = law_id.clone();
        let version = self.modify(|constitution| {
            constitution.laws.remove(&law_id);
        });

        Ok(version)
    }

    /// Update an existing law
    ///
    /// # Arguments
    /// * `law_id` - ID of the law to update
    /// * `updater` - Function that modifies the law
    ///
    /// # Returns
    /// * `Ok(version)` with new version number if law was found and updated
    /// * `Err(ConstitutionError::LawNotFound)` if law doesn't exist
    pub fn update_law<F>(&mut self, law_id: &LawId, updater: F) -> ConstitutionResult<u32>
    where
        F: FnOnce(&mut Law),
    {
        if !self.current.laws.contains_key(law_id) {
            return Err(ConstitutionError::LawNotFound(law_id.clone()));
        }

        let law_id = law_id.clone();
        let version = self.modify(|constitution| {
            if let Some(law) = constitution.laws.get_mut(&law_id) {
                updater(law);
            }
        });

        Ok(version)
    }

    /// Rollback to a previous version
    ///
    /// # Arguments
    /// * `version` - Version number to rollback to
    ///
    /// # Returns
    /// * `Ok(())` if rollback successful
    /// * `Err(ConstitutionError::InvalidVersion)` if version doesn't exist
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_governance::constitution::ConstitutionManager;
    ///
    /// let mut manager = ConstitutionManager::new(
    ///     "cons.1".to_string(),
    ///     "Test Constitution".to_string(),
    ///     None,
    /// );
    ///
    /// // Make some modifications
    /// manager.modify(|c| c.name = "Version 2".to_string());
    /// manager.modify(|c| c.name = "Version 3".to_string());
    ///
    /// assert_eq!(manager.current_version(), 3);
    ///
    /// // Rollback to version 1
    /// manager.rollback(1).unwrap();
    /// assert_eq!(manager.current_version(), 1);
    /// assert_eq!(manager.current().name, "Test Constitution");
    /// ```
    pub fn rollback(&mut self, version: u32) -> ConstitutionResult<()> {
        let version_data = self
            .history
            .get(&version)
            .ok_or(ConstitutionError::InvalidVersion(version))?;

        self.current = version_data.constitution.clone();
        Ok(())
    }

    /// Rollback to the previous version
    ///
    /// # Returns
    /// * `Ok(version)` with the version number rolled back to
    /// * `Err(ConstitutionError::NoPreviousVersion)` if already at version 1
    pub fn rollback_previous(&mut self) -> ConstitutionResult<u32> {
        let current_version = self.current.version;
        if current_version <= 1 {
            return Err(ConstitutionError::NoPreviousVersion);
        }

        let previous_version = current_version - 1;
        self.rollback(previous_version)?;
        Ok(previous_version)
    }

    /// Get the modification timestamp for a specific version
    pub fn get_version_timestamp(&self, version: u32) -> Option<chrono::DateTime<chrono::Utc>> {
        self.history.get(&version).map(|v| v.modified_at)
    }
}
