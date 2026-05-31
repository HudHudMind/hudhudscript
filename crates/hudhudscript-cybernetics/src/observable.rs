use std::fmt;
use std::time::{Duration, Instant};

/// A snapshot of the system state as seen by an `Observer`.
///
/// This is the sensor reading y(t) in control theory.
/// Note: not Deserializable because `Instant` has no Default; observations are
/// always created at runtime, never read from serialised form.
#[derive(Debug, Clone)]
pub struct Observable<S: Clone + fmt::Debug> {
    /// The observed state value.
    pub value: S,
    /// When the observation was taken.
    pub timestamp: Instant,
    /// Confidence in the reading [0.0, 1.0].
    pub confidence: f64,
}

impl<S: Clone + fmt::Debug> Observable<S> {
    /// Create a new observation with full confidence.
    pub fn new(value: S) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            confidence: 1.0,
        }
    }

    /// Attach a confidence level.
    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }

    /// Age of this observation.
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}
