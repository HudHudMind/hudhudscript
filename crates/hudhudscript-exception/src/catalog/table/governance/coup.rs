use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const COUP_AGENT_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(60),
        long_code: "HHS_E_COUP_AGENT_NOT_FOUND",
        short_code: "E0060",
        title: "Coup target agent not found",
        short_description: "A coup operation referenced an agent ID that does not exist in the governance system.",
        long_description: "A coup is an emergency override mechanism that temporarily suspends normal governance for a designated agent or group. Initiating, joining, or resolving a coup requires every referenced agent to exist in the runtime. This error fires when the agent ID cannot be resolved — either it was never registered, has been removed, or the ID is malformed.

Because coups are used for emergency response, the system errs on the side of strictness: it will refuse to operate on phantom agents rather than silently no-op. Inspect the failing ID and confirm the agent is still alive in the governance graph.

After a coup, agents may have been deregistered as part of the resolution; replaying old coup logs against a fresh runtime is a common cause of this error.",
        hints: &["Verify the agent exists with `governance.has_agent(id)`", "Avoid replaying historical coup actions against a fresh runtime", "Log the offending agent ID — it appears in the message", "Confirm the agent has not been removed by an earlier coup step"],
        example_bad: Some("Coup::initiate(against: ghost_agent);"),
        example_good: Some("if governance.has_agent(target) {
  Coup::initiate(against: target);
}"),
        see_also: &["HHS_E_GOVERNANCE_AGENT_NOT_FOUND", "HHS_E_COUP_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ExceptionCategory::Governance,
    };

pub const COUP_CONSTITUTION_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(61),
        long_code: "HHS_E_COUP_CONSTITUTION_NOT_FOUND",
        short_code: "E0061",
        title: "Coup-bound constitution not found",
        short_description: "A coup referenced a constitution that is not registered in the governance system.",
        long_description: "A coup can declare an emergency constitution that takes precedence over the normal one for the duration of the override. This error is raised when that emergency constitution name cannot be resolved against the active governance registry.

Because a coup with no rules at all is dangerous, the runtime refuses to start a coup whose declared constitution is missing. Either register the emergency constitution before initiating the coup, or omit the binding to fall back on the default emergency rules.

This is the coup-specific form of `ConstitutionNotFound`; both indicate the same root cause but at different layers of the governance stack.",
        hints: &["Register the emergency constitution before initiating the coup", "Ensure emergency constitutions are loaded at runtime startup", "Check the registry with `governance.constitutions()`", "Omit the binding to use the default emergency rules"],
        example_bad: Some("Coup::initiate(against: rogue, constitution: \"emergency\");"),
        example_good: Some("governance.add_constitution(emergency_rules);
Coup::initiate(against: rogue, constitution: \"emergency\");"),
        see_also: &["HHS_E_CONSTITUTION_NOT_FOUND", "HHS_E_COUP_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ExceptionCategory::Governance,
    };
