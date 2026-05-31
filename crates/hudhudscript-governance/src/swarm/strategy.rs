//! Swarm coordination strategy — get and set strategy.

use crate::types::{CoordinationStrategy, Swarm};

impl Swarm {
    /// Get the current coordination strategy
    pub fn get_strategy(&self) -> CoordinationStrategy {
        self.coordination_strategy
    }

    /// Set a new coordination strategy
    pub fn set_strategy(&mut self, strategy: CoordinationStrategy) {
        self.coordination_strategy = strategy;
    }
}
