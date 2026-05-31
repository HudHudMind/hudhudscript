use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_GOVERNANCE_VIOLATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(232),
        long_code: "HHS_E_RUNTIME_GOVERNANCE_VIOLATION",
        short_code: "E0232",
        title: "Action violates active constitution",
        short_description: "An action was blocked by the governance layer because it violates a rule in the currently enforced constitution.",
        long_description: "HudHudScript's governance system lets hosts attach a constitution — a declarative set of rules — to a running script. When a script attempts something the constitution forbids (for example, calling a denied tool, accessing a restricted resource, or exceeding a declared capability), the runtime aborts with this error, naming the constitution and the specific rule that was violated.

Fix by either changing the script to stay within the allowed actions, or by updating the constitution to grant the needed capability — the latter requires authorization and is not something scripts can do themselves.

This error is distinct from `SecurityViolation`: governance is user- or deployment-defined policy, whereas security violations are structural sandbox breaks.",
        hints: &["Read the error: it names the constitution and the violated rule", "Adjust the script to stay within allowed capabilities", "Request a constitution update through the appropriate governance channel", "Do not try to work around governance — it is enforced at every step"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_SECURITY_VIOLATION", "HHS_E_RUNTIME_EXECUTION_FAILED", "HHS_E_RUNTIME_RESOURCE_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };
