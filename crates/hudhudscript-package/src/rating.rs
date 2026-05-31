//! Rating and review system for marketplace packages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// A single rating / review left by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub user_id: String,
    pub package_name: String,
    /// Star rating, clamped to 1..=5.
    pub stars: u8,
    pub review_text: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Rating {
    /// Create a new rating, clamping `stars` to the valid 1..=5 range.
    pub fn new(
        user_id: impl Into<String>,
        package_name: impl Into<String>,
        stars: u8,
        review_text: Option<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            package_name: package_name.into(),
            stars: stars.clamp(1, 5),
            review_text,
            timestamp: Utc::now(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory rating store
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory store for package ratings.
///
/// **Status**: This is a local-only implementation suitable for testing,
/// development, and offline package workflows. It defines the rating protocol
/// (data shape, query API, aggregation logic) so that a remote registry
/// backend can be plugged in without changing call sites.
///
/// To upgrade to a remote registry: implement a `RegistryRatingStore` that
/// satisfies the same public API but persists via HTTP to the package
/// registry. See `hudhudscript-package/src/registry.rs` for the endpoint
/// definitions.
///
/// (v0.4.47.9 — Issue #813: clarified intent)
#[derive(Debug, Default)]
pub struct RatingStore {
    /// Ratings keyed by package name.
    ratings: HashMap<String, Vec<Rating>>,
}

impl RatingStore {
    /// Create an empty rating store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rating. If the same user has already rated the package, the old
    /// rating is replaced.
    pub fn add_rating(&mut self, rating: Rating) {
        let entry = self.ratings.entry(rating.package_name.clone()).or_default();

        // Replace existing rating from the same user.
        entry.retain(|r| r.user_id != rating.user_id);
        entry.push(rating);
    }

    /// Get all ratings for a package.
    pub fn get_ratings(&self, package_name: &str) -> Vec<&Rating> {
        self.ratings
            .get(package_name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Compute the average star rating for a package.
    /// Returns `None` when there are no ratings.
    pub fn average_rating(&self, package_name: &str) -> Option<f64> {
        let ratings = self.ratings.get(package_name)?;
        if ratings.is_empty() {
            return None;
        }
        let sum: u64 = ratings.iter().map(|r| r.stars as u64).sum();
        Some(sum as f64 / ratings.len() as f64)
    }

    /// Return the top-rated packages (by average stars), limited to `limit`.
    /// Packages with fewer ratings are ranked lower on ties.
    pub fn top_rated(&self, limit: usize) -> Vec<(String, f64, usize)> {
        let mut ranked: Vec<(String, f64, usize)> = self
            .ratings
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(name, ratings)| {
                let sum: u64 = ratings.iter().map(|r| r.stars as u64).sum();
                let avg = sum as f64 / ratings.len() as f64;
                (name.clone(), avg, ratings.len())
            })
            .collect();

        // Sort descending by average, then descending by count for ties.
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.cmp(&a.2))
        });

        ranked.truncate(limit);
        ranked
    }

    /// Return the total number of ratings across all packages.
    pub fn total_ratings(&self) -> usize {
        self.ratings.values().map(|v| v.len()).sum()
    }
}
