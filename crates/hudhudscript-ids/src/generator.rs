//! ID generation for governance structures.
//!
//! This module provides thread-safe, atomic counter-based ID generation for
//! constitutions, laws, rules, councils, swarms, and communities.

use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// ID Generator
// ============================================================================

/// Thread-safe ID generator using atomic counters.
///
/// This generator ensures unique IDs for all governance structure types:
/// - Constitution IDs: cons.N
/// - Law IDs: consN.lawM
/// - Rule IDs: rule.N
/// - Council IDs: council_N
/// - Swarm IDs: swarm_N
/// - Community IDs: community_N
///
/// All operations are thread-safe and guarantee uniqueness through atomic operations.
#[derive(Debug)]
pub struct IdGenerator {
    constitution_counter: AtomicU64,
    law_counter: AtomicU64,
    rule_counter: AtomicU64,
    council_counter: AtomicU64,
    swarm_counter: AtomicU64,
    community_counter: AtomicU64,
}

impl IdGenerator {
    /// Creates a new ID generator with all counters starting at 1.
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_constitution_id(), "cons.1");
    /// assert_eq!(generator.next_constitution_id(), "cons.2");
    /// ```
    pub fn new() -> Self {
        Self {
            constitution_counter: AtomicU64::new(1),
            law_counter: AtomicU64::new(1),
            rule_counter: AtomicU64::new(1),
            council_counter: AtomicU64::new(1),
            swarm_counter: AtomicU64::new(1),
            community_counter: AtomicU64::new(1),
        }
    }

    /// Creates a new ID generator with custom starting values.
    ///
    /// This is useful when resuming from a persisted state or when you need
    /// to avoid ID collisions with existing structures.
    ///
    /// # Arguments
    /// * `constitution_start` - Starting value for constitution counter
    /// * `law_start` - Starting value for law counter
    /// * `rule_start` - Starting value for rule counter
    /// * `council_start` - Starting value for council counter
    /// * `swarm_start` - Starting value for swarm counter
    /// * `community_start` - Starting value for community counter
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::with_start_values(100, 200, 300, 400, 500, 600);
    /// assert_eq!(generator.next_constitution_id(), "cons.100");
    /// assert_eq!(generator.next_rule_id(), "rule.300");
    /// ```
    pub fn with_start_values(
        constitution_start: u64,
        law_start: u64,
        rule_start: u64,
        council_start: u64,
        swarm_start: u64,
        community_start: u64,
    ) -> Self {
        Self {
            constitution_counter: AtomicU64::new(constitution_start),
            law_counter: AtomicU64::new(law_start),
            rule_counter: AtomicU64::new(rule_start),
            council_counter: AtomicU64::new(council_start),
            swarm_counter: AtomicU64::new(swarm_start),
            community_counter: AtomicU64::new(community_start),
        }
    }

    /// Generates the next constitution ID in the format "cons.N".
    ///
    /// This operation is thread-safe and guarantees uniqueness.
    ///
    /// # Returns
    /// A unique constitution ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_constitution_id(), "cons.1");
    /// assert_eq!(generator.next_constitution_id(), "cons.2");
    /// assert_eq!(generator.next_constitution_id(), "cons.3");
    /// ```
    pub fn next_constitution_id(&self) -> String {
        let id = self.constitution_counter.fetch_add(1, Ordering::SeqCst);
        format!("cons.{}", id)
    }

    /// Generates the next law ID for a given constitution in the format "consN.lawM".
    ///
    /// This operation is thread-safe and guarantees uniqueness across all laws.
    ///
    /// # Arguments
    /// * `constitution_id` - The parent constitution ID (e.g., "cons.1")
    ///
    /// # Returns
    /// A unique law ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_law_id("cons.1"), "cons1.law1");
    /// assert_eq!(generator.next_law_id("cons.1"), "cons1.law2");
    /// assert_eq!(generator.next_law_id("cons.2"), "cons2.law3");
    /// ```
    pub fn next_law_id(&self, constitution_id: &str) -> String {
        let id = self.law_counter.fetch_add(1, Ordering::SeqCst);
        // Extract the numeric part from constitution_id (e.g., "cons.1" -> "1")
        let cons_num = constitution_id.strip_prefix("cons.").unwrap_or("0");
        format!("cons{}.law{}", cons_num, id)
    }

    /// Generates the next rule ID in the format "rule.N".
    ///
    /// This operation is thread-safe and guarantees uniqueness.
    ///
    /// # Returns
    /// A unique rule ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_rule_id(), "rule.1");
    /// assert_eq!(generator.next_rule_id(), "rule.2");
    /// assert_eq!(generator.next_rule_id(), "rule.3");
    /// ```
    pub fn next_rule_id(&self) -> String {
        let id = self.rule_counter.fetch_add(1, Ordering::SeqCst);
        format!("rule.{}", id)
    }

    /// Generates the next council ID in the format "council_N".
    ///
    /// This operation is thread-safe and guarantees uniqueness.
    ///
    /// # Returns
    /// A unique council ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_council_id(), "council_1");
    /// assert_eq!(generator.next_council_id(), "council_2");
    /// ```
    pub fn next_council_id(&self) -> String {
        let id = self.council_counter.fetch_add(1, Ordering::SeqCst);
        format!("council_{}", id)
    }

    /// Generates the next swarm ID in the format "swarm_N".
    ///
    /// This operation is thread-safe and guarantees uniqueness.
    ///
    /// # Returns
    /// A unique swarm ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_swarm_id(), "swarm_1");
    /// assert_eq!(generator.next_swarm_id(), "swarm_2");
    /// ```
    pub fn next_swarm_id(&self) -> String {
        let id = self.swarm_counter.fetch_add(1, Ordering::SeqCst);
        format!("swarm_{}", id)
    }

    /// Generates the next community ID in the format "community_N".
    ///
    /// This operation is thread-safe and guarantees uniqueness.
    ///
    /// # Returns
    /// A unique community ID string
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_ids::generator::IdGenerator;
    ///
    /// let generator = IdGenerator::new();
    /// assert_eq!(generator.next_community_id(), "community_1");
    /// assert_eq!(generator.next_community_id(), "community_2");
    /// ```
    pub fn next_community_id(&self) -> String {
        let id = self.community_counter.fetch_add(1, Ordering::SeqCst);
        format!("community_{}", id)
    }

    /// Gets the current constitution counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_constitution_count(&self) -> u64 {
        self.constitution_counter.load(Ordering::SeqCst)
    }

    /// Gets the current law counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_law_count(&self) -> u64 {
        self.law_counter.load(Ordering::SeqCst)
    }

    /// Gets the current rule counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_rule_count(&self) -> u64 {
        self.rule_counter.load(Ordering::SeqCst)
    }

    /// Gets the current council counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_council_count(&self) -> u64 {
        self.council_counter.load(Ordering::SeqCst)
    }

    /// Gets the current swarm counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_swarm_count(&self) -> u64 {
        self.swarm_counter.load(Ordering::SeqCst)
    }

    /// Gets the current community counter value without incrementing.
    ///
    /// This is useful for persistence or debugging.
    ///
    /// # Returns
    /// The current counter value
    pub fn current_community_count(&self) -> u64 {
        self.community_counter.load(Ordering::SeqCst)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
