use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const PERMISSION_AGENT_NOT_REGISTERED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(186),
        long_code: "HHS_E_PERMISSION_AGENT_NOT_REGISTERED",
        short_code: "E0186",
        title: "Agent is not registered with the permission system",
        short_description: "A permission check was performed for an agent that has no entry in the orchestration ACL registry.",
        long_description: "Every agent participating in a permissioned orchestration must be registered with the permission system so it has an identity to authorize against. This error fires when a check is requested for an agent id that has no corresponding ACL record — either because registration was skipped, the agent was deregistered, or the id is misspelled.

Register each agent during spawn with `permissions.register_agent(id, role)` and deregister on shutdown. Treat the registration as part of the agent lifecycle so it cannot be forgotten. For dynamically spawned agents, use a factory that performs registration as part of construction.

If you intentionally want unauthenticated agents, place them in a sandbox network with a permissive default policy rather than bypassing registration.",
        hints: &["Register agents as part of their spawn lifecycle", "Use a factory that registers permissions on construction", "Place unauthenticated agents in a sandbox network instead of skipping registration"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_PERMISSION_DENIED"],
        since_version: "0.4.0",
        category: ExceptionCategory::Orchestration,
    };

pub const PERMISSION_DENIED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(187),
        long_code: "HHS_E_PERMISSION_DENIED",
        short_code: "E0187",
        title: "Agent is not permitted to perform this action",
        short_description: "The permission system rejected an action because the agent's role does not grant access to the requested resource.",
        long_description: "HudHudScript orchestration enforces ACLs on cross-agent calls, resource reads, and tool invocations. When an agent attempts an action that its role does not permit, the call is rejected with this error and the violation is logged for audit.

Fix this by either granting the agent the necessary permission (via role assignment or policy update), or by routing the action through an agent that already has the right. Do not catch and ignore this error blindly — permission failures are signals about either a policy gap or an attempted privilege escalation, both of which deserve attention.

Audit logs include `agent`, `resource`, and `action` fields so you can triage whether the policy needs to change or the calling code is misbehaving.",
        hints: &["Review the role assigned to the agent and the policy on the resource", "Route the action through an agent that already holds the permission", "Never silently swallow permission errors — log and alert on them"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_PERMISSION_AGENT_NOT_REGISTERED"],
        since_version: "0.4.0",
        category: ExceptionCategory::Orchestration,
    };
