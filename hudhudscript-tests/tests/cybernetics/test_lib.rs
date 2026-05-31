//! Public API tests for hudhudscript-cybernetics —
//! Goal, Observable, ControlError, ActuationResult, LoopStats,
//! CyberneticsError, BangBangPolicy, ControlLoop.

use hudhudscript_cybernetics::{
    ActuationResult, Actuator, BangBangPolicy, ControlError, ControlLoop, CyberneticsError,
    FeedbackPolicy, Goal, LoopStats, Observable, Observer,
};
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ── test fixtures ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct S {
    value: f64,
}

impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S({})", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum A {
    Up,
    Down,
    Noop,
}

struct ConstObs(S);

#[async_trait::async_trait]
impl Observer<S> for ConstObs {
    async fn observe(&self) -> Observable<S> {
        Observable::new(self.0.clone())
    }
}

struct CountingActuator {
    count: Arc<AtomicU32>,
    succeed: bool,
}

#[async_trait::async_trait]
impl Actuator<A> for CountingActuator {
    async fn actuate(&self, _a: A) -> Result<ActuationResult, CyberneticsError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        if self.succeed {
            Ok(ActuationResult::success("ok", Duration::from_millis(1)))
        } else {
            Ok(ActuationResult::failure("fail", Duration::from_millis(1)))
        }
    }
    fn name(&self) -> &str {
        "counting"
    }
}

struct AlwaysUpPolicy;

#[async_trait::async_trait]
impl FeedbackPolicy<S, A> for AlwaysUpPolicy {
    async fn compute(&self, _e: &ControlError<S>) -> A {
        A::Up
    }
    fn name(&self) -> &str {
        "always-up"
    }
}

fn make_loop_with_actuator(
    observed: f64,
    goal_val: f64,
    actuator: Arc<dyn Actuator<A>>,
) -> ControlLoop<S, A> {
    ControlLoop::new(
        "test-loop",
        Goal::new("reach", format!("{}", goal_val)),
        0.05,
        Arc::new(ConstObs(S { value: observed })),
        Arc::new(AlwaysUpPolicy),
        actuator,
        move |_g, obs| {
            let diff = (obs.value.value - goal_val).abs() / (goal_val.abs() + 1.0);
            ControlError::new(Goal::new("reach", ""), obs.clone(), "diff").with_magnitude(diff)
        },
    )
}

// ── Goal ──────────────────────────────────────────────────────────────────────

#[test]
fn goal_new_name_and_description() {
    let g = Goal::new("converge", "reach target");
    assert_eq!(g.name, "converge");
    assert_eq!(g.description, "reach target");
}

#[test]
fn goal_default_priority_is_zero() {
    let g = Goal::new("g", "desc");
    assert_eq!(g.priority, 0);
}

#[test]
fn goal_with_priority_sets_priority() {
    let g = Goal::new("g", "desc").with_priority(10);
    assert_eq!(g.priority, 10);
}

#[test]
fn goal_with_negative_priority() {
    let g = Goal::new("g", "desc").with_priority(-5);
    assert_eq!(g.priority, -5);
}

#[test]
fn goal_no_deadline_not_overdue() {
    let g = Goal::new("g", "desc");
    assert!(!g.is_overdue());
}

#[test]
fn goal_future_deadline_not_overdue() {
    let g = Goal::new("g", "desc").with_deadline(Instant::now() + Duration::from_secs(300));
    assert!(!g.is_overdue());
}

#[test]
fn goal_display_contains_name_and_priority() {
    let g = Goal::new("my-goal", "desc").with_priority(7);
    let s = format!("{}", g);
    assert!(s.contains("my-goal"));
    assert!(s.contains("priority=7"));
}

#[test]
fn goal_clone_preserves_fields() {
    let g = Goal::new("g", "desc").with_priority(3);
    let g2 = g.clone();
    assert_eq!(g2.name, "g");
    assert_eq!(g2.priority, 3);
}

// ── Observable ────────────────────────────────────────────────────────────────

#[test]
fn observable_new_full_confidence() {
    let obs = Observable::new(S { value: 42.0 });
    assert!((obs.confidence - 1.0).abs() < 1e-9);
}

#[test]
fn observable_new_stores_value() {
    let obs = Observable::new(S { value: 3.14 });
    assert!((obs.value.value - 3.14).abs() < 1e-9);
}

#[test]
fn observable_with_confidence_sets_confidence() {
    let obs = Observable::new(S { value: 1.0 }).with_confidence(0.75);
    assert!((obs.confidence - 0.75).abs() < 1e-5);
}

#[test]
fn observable_with_confidence_clamped_above_one() {
    let obs = Observable::new(S { value: 1.0 }).with_confidence(2.0);
    assert!((obs.confidence - 1.0).abs() < 1e-9);
}

#[test]
fn observable_with_confidence_clamped_below_zero() {
    let obs = Observable::new(S { value: 1.0 }).with_confidence(-0.5);
    assert!((obs.confidence - 0.0).abs() < 1e-9);
}

#[test]
fn observable_age_is_nonnegative() {
    let obs = Observable::new(S { value: 1.0 });
    assert!(obs.age().as_nanos() < 1_000_000_000); // less than 1 second
}

// ── ControlError ─────────────────────────────────────────────────────────────

#[test]
fn control_error_new_default_magnitude_one() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "test",
    );
    assert!((err.magnitude - 1.0).abs() < 1e-9);
}

#[test]
fn control_error_with_magnitude_sets_magnitude() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "test",
    )
    .with_magnitude(0.42);
    assert!((err.magnitude - 0.42).abs() < 1e-5);
}

#[test]
fn control_error_magnitude_clamped_above_one() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "test",
    )
    .with_magnitude(5.0);
    assert!((err.magnitude - 1.0).abs() < 1e-9);
}

#[test]
fn control_error_magnitude_clamped_below_zero() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "test",
    )
    .with_magnitude(-1.0);
    assert!((err.magnitude - 0.0).abs() < 1e-9);
}

#[test]
fn control_error_within_tolerance_when_below() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "small",
    )
    .with_magnitude(0.03);
    assert!(err.within_tolerance(0.05));
}

#[test]
fn control_error_not_within_tolerance_when_above() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "large",
    )
    .with_magnitude(0.1);
    assert!(!err.within_tolerance(0.05));
}

#[test]
fn control_error_within_tolerance_at_boundary() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "boundary",
    )
    .with_magnitude(0.05);
    assert!(err.within_tolerance(0.05));
}

#[test]
fn control_error_display_contains_goal_name() {
    let err = ControlError::new(
        Goal::new("my-goal", ""),
        Observable::new(S { value: 1.0 }),
        "desc",
    )
    .with_magnitude(0.5);
    let s = format!("{}", err);
    assert!(s.contains("my-goal"));
}

#[test]
fn control_error_display_contains_magnitude() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "desc",
    )
    .with_magnitude(0.5);
    let s = format!("{}", err);
    assert!(s.contains("0.500"));
}

#[test]
fn control_error_display_contains_description() {
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "error description",
    )
    .with_magnitude(0.1);
    let s = format!("{}", err);
    assert!(s.contains("error description"));
}

// ── ActuationResult ───────────────────────────────────────────────────────────

#[test]
fn actuation_result_success_is_success_true() {
    let r = ActuationResult::success("done", Duration::from_millis(10));
    assert!(r.success);
}

#[test]
fn actuation_result_success_stores_description() {
    let r = ActuationResult::success("action completed", Duration::from_millis(10));
    assert_eq!(r.description, "action completed");
}

#[test]
fn actuation_result_failure_is_success_false() {
    let r = ActuationResult::failure("boom", Duration::from_millis(5));
    assert!(!r.success);
}

#[test]
fn actuation_result_failure_stores_description() {
    let r = ActuationResult::failure("hardware error", Duration::from_millis(5));
    assert_eq!(r.description, "hardware error");
}

#[test]
fn actuation_result_stores_duration() {
    let d = Duration::from_millis(42);
    let r = ActuationResult::success("ok", d);
    assert_eq!(r.duration, d);
}

// ── LoopStats ─────────────────────────────────────────────────────────────────

#[test]
fn loop_stats_default_all_zero() {
    let stats = LoopStats::default();
    assert_eq!(stats.ticks, 0);
    assert_eq!(stats.converged_ticks, 0);
    assert_eq!(stats.actuation_failures, 0);
}

#[test]
fn loop_stats_convergence_rate_zero_ticks() {
    assert_eq!(LoopStats::default().convergence_rate(), 0.0);
}

#[test]
fn loop_stats_convergence_rate_all_converged() {
    let stats = LoopStats {
        ticks: 10,
        converged_ticks: 10,
        actuation_failures: 0,
        total_tick_time: Duration::ZERO,
    };
    assert!((stats.convergence_rate() - 1.0).abs() < 1e-9);
}

#[test]
fn loop_stats_convergence_rate_partial() {
    let stats = LoopStats {
        ticks: 10,
        converged_ticks: 7,
        actuation_failures: 0,
        total_tick_time: Duration::ZERO,
    };
    assert!((stats.convergence_rate() - 0.7).abs() < 1e-9);
}

#[test]
fn loop_stats_display_contains_ticks() {
    let stats = LoopStats {
        ticks: 10,
        converged_ticks: 7,
        actuation_failures: 2,
        total_tick_time: Duration::ZERO,
    };
    let s = format!("{}", stats);
    assert!(s.contains("ticks=10"));
}

#[test]
fn loop_stats_display_contains_convergence_percent() {
    let stats = LoopStats {
        ticks: 10,
        converged_ticks: 7,
        actuation_failures: 2,
        total_tick_time: Duration::ZERO,
    };
    let s = format!("{}", stats);
    assert!(s.contains("70.0%"));
}

#[test]
fn loop_stats_display_contains_failures() {
    let stats = LoopStats {
        ticks: 10,
        converged_ticks: 7,
        actuation_failures: 2,
        total_tick_time: Duration::ZERO,
    };
    let s = format!("{}", stats);
    assert!(s.contains("failures=2"));
}

#[test]
fn loop_stats_clone_preserves_fields() {
    let stats = LoopStats {
        ticks: 5,
        converged_ticks: 3,
        actuation_failures: 1,
        total_tick_time: Duration::ZERO,
    };
    let s2 = stats.clone();
    assert_eq!(s2.ticks, 5);
    assert_eq!(s2.converged_ticks, 3);
}

// ── CyberneticsError ──────────────────────────────────────────────────────────

#[test]
fn cybernetics_error_actuation_failed_display() {
    let e = CyberneticsError::ActuationFailed {
        loop_name: "loop1".to_string(),
        reason: "hardware failure".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("actuation failed"));
    assert!(s.contains("loop1"));
    assert!(s.contains("hardware failure"));
}

#[test]
fn cybernetics_error_observer_error_display() {
    let e = CyberneticsError::ObserverError {
        loop_name: "loop2".to_string(),
        reason: "sensor offline".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("observer error"));
    assert!(s.contains("loop2"));
    assert!(s.contains("sensor offline"));
}

#[test]
fn cybernetics_error_policy_error_display() {
    let e = CyberneticsError::PolicyError {
        loop_name: "loop3".to_string(),
        reason: "divide by zero".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("policy error"));
    assert!(s.contains("loop3"));
}

// ── BangBangPolicy ────────────────────────────────────────────────────────────

#[tokio::test]
async fn bang_bang_policy_high_error_returns_on_action() {
    let policy = BangBangPolicy::new("bb", 0.5, A::Up, A::Noop);
    let err = ControlError::new(
        Goal::new("g", ""),
        Observable::new(S { value: 1.0 }),
        "high",
    )
    .with_magnitude(0.8);
    let action = FeedbackPolicy::<S, A>::compute(&policy, &err).await;
    assert_eq!(action, A::Up);
}

#[tokio::test]
async fn bang_bang_policy_low_error_returns_off_action() {
    let policy = BangBangPolicy::new("bb", 0.5, A::Up, A::Noop);
    let err = ControlError::new(Goal::new("g", ""), Observable::new(S { value: 1.0 }), "low")
        .with_magnitude(0.2);
    let action = FeedbackPolicy::<S, A>::compute(&policy, &err).await;
    assert_eq!(action, A::Noop);
}

#[test]
fn bang_bang_policy_name() {
    let policy = BangBangPolicy::new("thermostat", 0.5, A::Up, A::Noop);
    assert_eq!(FeedbackPolicy::<S, A>::name(&policy), "thermostat");
}

// ── ControlLoop ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn converged_loop_tick_returns_true() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        10.0,
        10.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    let result = lp.tick().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn converged_loop_increments_ticks_and_converged_ticks() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        10.0,
        10.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    lp.tick().await.unwrap();
    let stats = lp.stats().await;
    assert_eq!(stats.ticks, 1);
    assert_eq!(stats.converged_ticks, 1);
}

#[tokio::test]
async fn non_converged_loop_tick_returns_false() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        0.0,
        100.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    let result = lp.tick().await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn non_converged_loop_increments_ticks_not_converged() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        0.0,
        100.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    lp.tick().await.unwrap();
    let stats = lp.stats().await;
    assert_eq!(stats.ticks, 1);
    assert_eq!(stats.converged_ticks, 0);
}

#[tokio::test]
async fn failed_actuation_increments_failure_counter() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        0.0,
        100.0,
        Arc::new(CountingActuator {
            count,
            succeed: false,
        }),
    );
    let result = lp.tick().await;
    assert!(result.is_err());
    let stats = lp.stats().await;
    assert_eq!(stats.actuation_failures, 1);
}

#[tokio::test]
async fn multiple_ticks_accumulate_stats() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        0.0,
        100.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    for _ in 0..3 {
        let _ = lp.tick().await;
    }
    let stats = lp.stats().await;
    assert_eq!(stats.ticks, 3);
}

#[test]
fn control_loop_debug_contains_name() {
    let count = Arc::new(AtomicU32::new(0));
    let lp = make_loop_with_actuator(
        0.0,
        10.0,
        Arc::new(CountingActuator {
            count,
            succeed: true,
        }),
    );
    let s = format!("{:?}", lp);
    assert!(s.contains("test-loop"));
}

// ── Comprehensive companion tests ────────────────────────────────────────────

#[test]
fn test_goal_full_state() {
    // Verify ALL Goal properties in a single test: name, description, priority, deadline, display.
    let deadline = Instant::now() + Duration::from_secs(600);
    let g = Goal::new("optimize-throughput", "maximize requests per second")
        .with_priority(5)
        .with_deadline(deadline);

    // name and description
    assert_eq!(g.name, "optimize-throughput");
    assert_eq!(g.description, "maximize requests per second");

    // priority
    assert_eq!(g.priority, 5);

    // deadline set and not overdue
    assert!(g.deadline.is_some());
    assert!(!g.is_overdue());

    // Display includes both name and priority
    let display = format!("{}", g);
    assert!(display.contains("optimize-throughput"));
    assert!(display.contains("priority=5"));

    // Clone preserves all fields
    let g2 = g.clone();
    assert_eq!(g2.name, g.name);
    assert_eq!(g2.description, g.description);
    assert_eq!(g2.priority, g.priority);
    assert!(g2.deadline.is_some());
    assert!(!g2.is_overdue());

    // Default (no deadline, no priority override)
    let g_default = Goal::new("simple", "basic goal");
    assert_eq!(g_default.priority, 0);
    assert!(g_default.deadline.is_none());
    assert!(!g_default.is_overdue());
}

#[test]
fn test_observable_full_state() {
    // Verify ALL Observable properties: value, confidence, timestamp/age.
    let before = Instant::now();
    let obs = Observable::new(S { value: 99.5 }).with_confidence(0.85);
    let after = Instant::now();

    // value stored correctly
    assert!((obs.value.value - 99.5).abs() < 1e-9);

    // confidence set and within range
    assert!((obs.confidence - 0.85).abs() < 1e-5);

    // timestamp is between before and after
    assert!(obs.timestamp >= before);
    assert!(obs.timestamp <= after);

    // age is non-negative and small (just created)
    assert!(obs.age() < Duration::from_secs(1));

    // Default confidence is 1.0
    let obs_default = Observable::new(S { value: 0.0 });
    assert!((obs_default.confidence - 1.0).abs() < 1e-9);

    // Confidence clamping: above 1.0
    let obs_high = Observable::new(S { value: 1.0 }).with_confidence(5.0);
    assert!((obs_high.confidence - 1.0).abs() < 1e-9);

    // Confidence clamping: below 0.0
    let obs_low = Observable::new(S { value: 1.0 }).with_confidence(-3.0);
    assert!((obs_low.confidence - 0.0).abs() < 1e-9);

    // Clone preserves value and confidence
    let obs_clone = obs.clone();
    assert!((obs_clone.value.value - 99.5).abs() < 1e-9);
    assert!((obs_clone.confidence - 0.85).abs() < 1e-5);
}

#[test]
fn test_actuation_result_comprehensive() {
    // Verify ALL ActuationResult fields for both success and failure paths.
    let dur_ok = Duration::from_millis(42);
    let r_ok = ActuationResult::success("valve opened", dur_ok);
    assert!(r_ok.success);
    assert_eq!(r_ok.description, "valve opened");
    assert_eq!(r_ok.duration, dur_ok);

    let dur_fail = Duration::from_millis(7);
    let r_fail = ActuationResult::failure("motor stalled", dur_fail);
    assert!(!r_fail.success);
    assert_eq!(r_fail.description, "motor stalled");
    assert_eq!(r_fail.duration, dur_fail);

    // Clone preserves all fields
    let r_clone = r_ok.clone();
    assert!(r_clone.success);
    assert_eq!(r_clone.description, "valve opened");
    assert_eq!(r_clone.duration, dur_ok);

    // Zero-duration edge case
    let r_zero = ActuationResult::success("instant", Duration::ZERO);
    assert!(r_zero.success);
    assert_eq!(r_zero.duration, Duration::ZERO);
    assert_eq!(r_zero.description, "instant");
}

#[test]
fn test_control_error_full_state() {
    // Verify ALL ControlError fields: goal, observation, description, magnitude, tolerance, display.
    let goal = Goal::new("stabilize", "keep value at 50").with_priority(3);
    let obs = Observable::new(S { value: 45.0 }).with_confidence(0.9);
    let err = ControlError::new(goal, obs, "deviation from target").with_magnitude(0.35);

    // goal preserved inside error
    assert_eq!(err.goal.name, "stabilize");
    assert_eq!(err.goal.description, "keep value at 50");
    assert_eq!(err.goal.priority, 3);

    // observation preserved
    assert!((err.observation.value.value - 45.0).abs() < 1e-9);
    assert!((err.observation.confidence - 0.9).abs() < 1e-5);

    // description and magnitude
    assert_eq!(err.description, "deviation from target");
    assert!((err.magnitude - 0.35).abs() < 1e-5);

    // tolerance checks
    assert!(err.within_tolerance(0.35)); // at boundary
    assert!(err.within_tolerance(0.5)); // above
    assert!(!err.within_tolerance(0.3)); // below

    // Display includes goal name, magnitude, and description
    let s = format!("{}", err);
    assert!(s.contains("stabilize"));
    assert!(s.contains("0.350"));
    assert!(s.contains("deviation from target"));
}

#[tokio::test]
async fn test_feedback_loop_integration() {
    // Full feedback loop cycle: observer -> error -> policy -> actuator -> stats.
    // Tests the complete cycle with multiple ticks mixing converged and non-converged states.

    // --- Phase 1: non-converged loop, successful actuator ---
    let count = Arc::new(AtomicU32::new(0));
    let actuator = Arc::new(CountingActuator {
        count: count.clone(),
        succeed: true,
    });
    let lp = make_loop_with_actuator(0.0, 100.0, actuator);

    // First tick: large error, should actuate and return false (not converged)
    let result = lp.tick().await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // Second tick: same state, actuate again
    let result2 = lp.tick().await;
    assert!(result2.is_ok());
    assert!(!result2.unwrap());
    assert_eq!(count.load(Ordering::SeqCst), 2);

    // Stats after 2 non-converged ticks
    let stats = lp.stats().await;
    assert_eq!(stats.ticks, 2);
    assert_eq!(stats.converged_ticks, 0);
    assert_eq!(stats.actuation_failures, 0);
    assert!(stats.total_tick_time > Duration::ZERO);
    assert!((stats.convergence_rate() - 0.0).abs() < 1e-9);

    // --- Phase 2: converged loop ---
    let count2 = Arc::new(AtomicU32::new(0));
    let actuator2 = Arc::new(CountingActuator {
        count: count2.clone(),
        succeed: true,
    });
    let lp2 = make_loop_with_actuator(10.0, 10.0, actuator2);

    let result3 = lp2.tick().await;
    assert!(result3.is_ok());
    assert!(result3.unwrap()); // converged
                               // Actuator should NOT be called when converged
    assert_eq!(count2.load(Ordering::SeqCst), 0);

    let stats2 = lp2.stats().await;
    assert_eq!(stats2.ticks, 1);
    assert_eq!(stats2.converged_ticks, 1);
    assert!((stats2.convergence_rate() - 1.0).abs() < 1e-9);

    // --- Phase 3: failed actuator ---
    let count3 = Arc::new(AtomicU32::new(0));
    let actuator3 = Arc::new(CountingActuator {
        count: count3.clone(),
        succeed: false,
    });
    let lp3 = make_loop_with_actuator(0.0, 100.0, actuator3);

    let result4 = lp3.tick().await;
    assert!(result4.is_err());
    // Actuator was still called
    assert_eq!(count3.load(Ordering::SeqCst), 1);

    let stats3 = lp3.stats().await;
    assert_eq!(stats3.ticks, 1);
    assert_eq!(stats3.converged_ticks, 0);
    assert_eq!(stats3.actuation_failures, 1);

    // Verify the error type
    if let Err(e) = lp3.tick().await {
        let msg = format!("{}", e);
        assert!(msg.contains("test-loop"));
        assert!(msg.contains("actuation failed"));
    }
}
