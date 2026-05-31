use crate::actuation::ActuationResult;
use crate::control_error::ControlError;
use crate::observable::Observable;
use std::fmt;

/// An observer measures the current state of the system (the sensor).
///
/// Implementors wrap whatever telemetry or introspection mechanism is
/// appropriate for the agent network being monitored.
#[async_trait::async_trait]
pub trait Observer<S: Clone + fmt::Debug + Send + Sync>: Send + Sync {
    /// Take a snapshot of the current state.
    async fn observe(&self) -> Observable<S>;
}

/// A feedback policy maps a control error to an action.
///
/// This is the controller's transfer function.  Different policy types (PID,
/// bang-bang, model-predictive, …) are expressed as implementations of this
/// trait.
#[async_trait::async_trait]
pub trait FeedbackPolicy<S, A>: Send + Sync
where
    S: Clone + fmt::Debug + Send + Sync,
    A: Clone + fmt::Debug + Send + Sync,
{
    /// Compute the control action given the current error.
    async fn compute(&self, error: &ControlError<S>) -> A;

    /// Name of this policy (used in logs).
    fn name(&self) -> &str;
}

/// An actuator applies a control action to the system.
///
/// In agent orchestration the actuator might:
/// - Spawn or kill agents.
/// - Adjust token budgets.
/// - Reroute data flows in the network.
/// - Trigger a workflow.
#[async_trait::async_trait]
pub trait Actuator<A: Clone + fmt::Debug + Send + Sync>: Send + Sync {
    /// Apply the action to the system.
    async fn actuate(&self, action: A) -> Result<ActuationResult, crate::errors::CyberneticsError>;

    /// Name of this actuator (used in logs).
    fn name(&self) -> &str;
}
