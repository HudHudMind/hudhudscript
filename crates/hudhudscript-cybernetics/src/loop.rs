use crate::control_error::ControlError;
use crate::errors::CyberneticsError;
use crate::goal::Goal;
use crate::observable::Observable;
use crate::traits::{Actuator, FeedbackPolicy, Observer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Statistics accumulated over the lifetime of a control loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopStats {
    /// Number of ticks executed.
    pub ticks: u64,
    /// Number of ticks where the goal was within tolerance.
    pub converged_ticks: u64,
    /// Number of actuation failures.
    pub actuation_failures: u64,
    /// Total wall-clock time spent in ticks.
    #[serde(skip)]
    pub total_tick_time: Duration,
}

impl LoopStats {
    /// Convergence rate = converged_ticks / ticks.
    pub fn convergence_rate(&self) -> f64 {
        if self.ticks == 0 {
            0.0
        } else {
            self.converged_ticks as f64 / self.ticks as f64
        }
    }
}

impl fmt::Display for LoopStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LoopStats(ticks={}, convergence={:.1}%, failures={})",
            self.ticks,
            self.convergence_rate() * 100.0,
            self.actuation_failures
        )
    }
}

/// A cybernetic control loop — the primary unit of the COML paradigm.
///
/// `S` is the observed state type; `A` is the action type.
///
/// The loop ties together an `Observer`, a `FeedbackPolicy`, and an `Actuator`
/// into a single self-regulating unit that can be driven by calling `tick()`.
pub struct ControlLoop<S, A>
where
    S: Clone + fmt::Debug + Send + Sync + 'static,
    A: Clone + fmt::Debug + Send + Sync + 'static,
{
    /// Human-readable name for this loop.
    pub name: String,
    /// The goal this loop is trying to achieve.
    pub goal: Goal,
    /// Error tolerance — errors below this magnitude are treated as "converged".
    pub tolerance: f64,
    /// The observer (sensor).
    observer: Arc<dyn Observer<S>>,
    /// The feedback policy (controller).
    policy: Arc<dyn FeedbackPolicy<S, A>>,
    /// The actuator (effector).
    actuator: Arc<dyn Actuator<A>>,
    /// An error mapper that converts `(goal, observation) → ControlError`.
    #[allow(clippy::type_complexity)]
    error_fn: Arc<dyn Fn(&Goal, &Observable<S>) -> ControlError<S> + Send + Sync>,
    /// Accumulated statistics.
    stats: Arc<RwLock<LoopStats>>,
}

impl<S, A> fmt::Debug for ControlLoop<S, A>
where
    S: Clone + fmt::Debug + Send + Sync + 'static,
    A: Clone + fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlLoop")
            .field("name", &self.name)
            .field("goal", &self.goal)
            .field("tolerance", &self.tolerance)
            .finish()
    }
}

impl<S, A> ControlLoop<S, A>
where
    S: Clone + fmt::Debug + Send + Sync + 'static,
    A: Clone + fmt::Debug + Send + Sync + 'static,
{
    /// Construct a new control loop.
    pub fn new(
        name: impl Into<String>,
        goal: Goal,
        tolerance: f64,
        observer: Arc<dyn Observer<S>>,
        policy: Arc<dyn FeedbackPolicy<S, A>>,
        actuator: Arc<dyn Actuator<A>>,
        error_fn: impl Fn(&Goal, &Observable<S>) -> ControlError<S> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            goal,
            tolerance,
            observer,
            policy,
            actuator,
            error_fn: Arc::new(error_fn),
            stats: Arc::new(RwLock::new(LoopStats::default())),
        }
    }

    /// Execute one tick of the control loop.
    ///
    /// Returns `Ok(true)` if the loop has converged (error within tolerance),
    /// `Ok(false)` if an action was taken, or `Err` on actuation failure.
    pub async fn tick(&self) -> Result<bool, CyberneticsError> {
        let start = Instant::now();

        // 1. Observe current state.
        let obs = self.observer.observe().await;

        // 2. Compute error.
        let error = (self.error_fn)(&self.goal, &obs);

        // 3. Check convergence.
        if error.within_tolerance(self.tolerance) {
            let mut stats = self.stats.write().await;
            stats.ticks += 1;
            stats.converged_ticks += 1;
            stats.total_tick_time += start.elapsed();
            return Ok(true);
        }

        // 4. Compute control action.
        let action = self.policy.compute(&error).await;

        // 5. Actuate.
        let result = self.actuator.actuate(action).await;

        let mut stats = self.stats.write().await;
        stats.ticks += 1;
        stats.total_tick_time += start.elapsed();

        match result {
            Ok(actuation) if actuation.success => Ok(false),
            Ok(_) | Err(_) => {
                stats.actuation_failures += 1;
                Err(CyberneticsError::ActuationFailed {
                    loop_name: self.name.clone(),
                    reason: "actuation returned failure".to_string(),
                })
            }
        }
    }

    /// Return a snapshot of the accumulated loop statistics.
    pub async fn stats(&self) -> LoopStats {
        self.stats.read().await.clone()
    }
}
