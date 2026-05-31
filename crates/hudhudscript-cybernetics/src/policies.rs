use crate::control_error::ControlError;
use crate::traits::FeedbackPolicy;
use std::fmt;

/// A simple bang-bang (on/off) feedback policy.
///
/// If the error magnitude exceeds the threshold, emit the `on_action`; otherwise
/// emit the `off_action`.  This is the simplest non-trivial feedback policy —
/// analogous to a thermostat.
pub struct BangBangPolicy<A: Clone + fmt::Debug + Send + Sync + 'static> {
    name: String,
    threshold: f64,
    on_action: A,
    off_action: A,
}

impl<A: Clone + fmt::Debug + Send + Sync + 'static> BangBangPolicy<A> {
    /// Create a bang-bang policy.
    pub fn new(name: impl Into<String>, threshold: f64, on_action: A, off_action: A) -> Self {
        Self {
            name: name.into(),
            threshold,
            on_action,
            off_action,
        }
    }
}

#[async_trait::async_trait]
impl<S, A> FeedbackPolicy<S, A> for BangBangPolicy<A>
where
    S: Clone + fmt::Debug + Send + Sync,
    A: Clone + fmt::Debug + Send + Sync,
{
    async fn compute(&self, error: &ControlError<S>) -> A {
        if error.magnitude > self.threshold {
            self.on_action.clone()
        } else {
            self.off_action.clone()
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
