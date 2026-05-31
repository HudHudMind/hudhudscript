use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 3] = [
    ExceptionEntry {
        code: ExceptionCode(62),
        long_code: "HHS_E_CYBERNETICS_ACTUATION_FAILED",
        short_code: "E0062",
        title: "Cybernetic Loop Actuator Write Failed",
        short_description: "A control loop's actuator could not apply the commanded value to the downstream system.",
        long_description: "A cybernetic loop in `hudhudscript-cybernetics` consists of an observer that reads sensor state, a policy that decides on a correction, and an actuator that pushes that correction back to the controlled system. When the actuator side errors out — hardware fault, network drop to a remote endpoint, validation rejection by the target — the loop reports this variant tagged with the loop name and the underlying cause.

The loop is paused after the failure rather than continuing with stale state. Stale corrections in a feedback system can amplify faults, so the safe default is to surface the error and let supervising code decide on retry, fallback, or shutdown.

Inspect the wrapped cause to determine whether the actuator endpoint is reachable, whether your value range is acceptable, and whether the loop can be safely resumed.",
        hints: &["Check the wrapped cause for endpoint reachability or validation issues", "Do not auto-resume a feedback loop after actuation failure without analysis", "Verify the commanded value is within the actuator's accepted range", "Add a watchdog around the loop so failures escalate to a supervisor"],
        example_bad: None,
        example_good: None,
        see_also: &["CyberneticsObserverError", "CyberneticsPolicyError"],
        since_version: "0.4.5",
        category: ExceptionCategory::Cybernetics,
    },

    ExceptionEntry {
        code: ExceptionCode(63),
        long_code: "HHS_E_CYBERNETICS_OBSERVER_ERROR",
        short_code: "E0063",
        title: "Cybernetic Loop Observer Read Failed",
        short_description: "A control loop's observer could not read the current state of the system it monitors.",
        long_description: "Observers are the sensor side of a feedback loop. They poll, subscribe, or compute the value that the policy will react to. When that read fails — disconnected sensor, malformed sample, division by zero in a derived metric — the loop emits this variant naming both the loop and the underlying cause.

Without a fresh observation the policy has nothing legitimate to act on, so the loop pauses rather than feeding stale or fabricated data forward. This is by design: pretending the system is in its last-known state is a common path to runaway control failures.

Fix the observer's data source, then restart or step the loop. If the observer is intermittent by nature, wrap it with a smoothing or fallback layer before passing it into the loop core.",
        hints: &["Inspect the wrapped cause to find the sensor or computation at fault", "Avoid feeding stale samples — pause is safer than fabricated data", "Wrap intermittent observers in a smoothing/fallback layer", "Add observability around the observer to catch flapping early"],
        example_bad: None,
        example_good: None,
        see_also: &["CyberneticsActuationFailed", "CyberneticsPolicyError"],
        since_version: "0.4.5",
        category: ExceptionCategory::Cybernetics,
    },

    ExceptionEntry {
        code: ExceptionCode(64),
        long_code: "HHS_E_CYBERNETICS_POLICY_ERROR",
        short_code: "E0064",
        title: "Cybernetic Loop Policy Decision Failed",
        short_description: "A control loop's policy function raised an error while computing the next correction from observed state.",
        long_description: "The policy is the brain of a feedback loop. It accepts the latest observation and produces the value that the actuator will apply. Any failure inside the policy — script exception, divergence in a numerical solver, contract violation on its inputs — is wrapped in this variant with the loop name attached.

A policy error blocks the loop from proceeding because the alternative would mean re-running the previous correction, which can drive the system away from the setpoint. The loop pauses and waits for a supervisor decision.

Inspect the wrapped cause for the precise script-level or solver-level error. If the policy is data-driven (e.g. a model), check whether its expected input shape changed.",
        hints: &["Inspect the wrapped cause — it usually carries a script stack frame", "Check whether the observation shape changed under the policy's feet", "Validate that numerical solvers converge before deploying the loop", "Guard policies with input contracts so failures are localized"],
        example_bad: None,
        example_good: None,
        see_also: &["CyberneticsObserverError", "CyberneticsActuationFailed"],
        since_version: "0.4.5",
        category: ExceptionCategory::Cybernetics,
    }
];
