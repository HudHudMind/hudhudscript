use crate::goal::Goal;
use crate::observable::Observable;
use std::fmt;

/// The difference between the desired goal and the observed state — e(t).
///
/// `S` is the state type; the error is expressed in the same space.
#[derive(Debug, Clone)]
pub struct ControlError<S: Clone + fmt::Debug> {
    /// The goal that was not yet achieved.
    pub goal: Goal,
    /// The observation that was compared against the goal.
    pub observation: Observable<S>,
    /// A semantic description of the gap.
    pub description: String,
    /// Magnitude of the error on a normalised [0.0, 1.0] scale.
    /// 0.0 = goal fully achieved; 1.0 = maximally far from goal.
    pub magnitude: f64,
}

impl<S: Clone + fmt::Debug> ControlError<S> {
    /// Create a control error.
    pub fn new(goal: Goal, observation: Observable<S>, description: impl Into<String>) -> Self {
        Self {
            goal,
            observation,
            description: description.into(),
            magnitude: 1.0,
        }
    }

    /// Attach a normalised magnitude.
    pub fn with_magnitude(mut self, m: f64) -> Self {
        self.magnitude = m.clamp(0.0, 1.0);
        self
    }

    /// Returns `true` if the error magnitude is below the given threshold.
    pub fn within_tolerance(&self, tolerance: f64) -> bool {
        self.magnitude <= tolerance
    }
}

impl<S: Clone + fmt::Debug + fmt::Display> fmt::Display for ControlError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ControlError(goal={}, magnitude={:.3}, desc={})",
            self.goal, self.magnitude, self.description
        )
    }
}
