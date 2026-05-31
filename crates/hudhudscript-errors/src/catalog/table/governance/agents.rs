use super::*;

pub const PERSPECTIVE_FIELD_HIDDEN: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(191),
        long_code: "HHS_E_PERSPECTIVE_FIELD_HIDDEN",
        short_code: "E0191",
        title: "Field hidden by current perspective",
        short_description: "Read access to a field was denied because the current perspective filters it out.",
        long_description: "Perspectives are per-agent view filters applied to objects in the runtime. A perspective declares which fields of an object are visible to a particular agent. When an agent reads a field that its perspective does not include, the runtime raises this error rather than returning a default value.

This is a *visibility* error, not an *authorization* error: the field exists and may be writable by the same agent, but it is not visible from this viewpoint. Visibility models information asymmetry — for example, a council member may see vote totals while observers see only the final result.

Fix by adjusting the perspective definition, by reading from an agent whose perspective grants the field, or by routing the read through a privileged accessor.",
        hints: &["Inspect the perspective with `agent.perspective().visible_fields(obj)`", "Update the perspective definition to include the required field", "Route privileged reads through an explicit accessor", "Distinguish hidden (read) from write-denied (write) perspectives"],
        example_bad: Some("let totals = observer_view(council).vote_totals;"),
        example_good: Some("let totals = chair_view(council).vote_totals;"),
        see_also: &["HHS_E_PERSPECTIVE_WRITE_ACCESS_DENIED"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const PERSPECTIVE_WRITE_ACCESS_DENIED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(192),
        long_code: "HHS_E_PERSPECTIVE_WRITE_ACCESS_DENIED",
        short_code: "E0192",
        title: "Write access denied by perspective",
        short_description: "An agent attempted to write a field that its perspective marks read-only or hidden.",
        long_description: "In addition to controlling visibility, perspectives mark each visible field as read-only or writable for a given agent. This error is raised when an agent attempts to write a field its perspective does not authorize for writes — even if the field is readable.

The distinction matters because many governance scenarios require asymmetric access: members can read minutes but only the chair can amend them; observers can see proposals but cannot vote.

Fix by amending the perspective to grant write access where appropriate, or by routing the write through an agent whose perspective allows it (e.g., the chair).",
        hints: &["Check `perspective.writable_fields(obj)` to see what is writable", "Route privileged writes through the appropriate role", "Don't widen perspectives without considering the security implications", "Distinguish read-only-visible from completely-hidden fields"],
        example_bad: Some("observer_view(council).chair = new_chair;"),
        example_good: Some("chair_view(council).chair = new_chair;"),
        see_also: &["HHS_E_PERSPECTIVE_FIELD_HIDDEN"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const ROLE_INVALID_ROLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(221),
        long_code: "HHS_E_ROLE_INVALID_ROLE",
        short_code: "E0221",
        title: "Invalid role definition",
        short_description: "A role descriptor failed validation in the role registry.",
        long_description: "The role registry validates role descriptors when they are registered: the name must be non-empty and unique, the permission set must reference known permissions, and any inheritance must form a DAG. This error fires when one of those checks fails.

This is the role-registry-level form. `CouncilInvalidRole` and `GovernanceInvalidRole` are the consumer-side variants reported when a council or the governance facade receives an invalid role at use time. All three indicate the same family of bugs from different layers.

Fix the descriptor at the point of registration. Avoid building roles ad hoc — use the `Role::builder()` API which validates incrementally.",
        hints: &["Use `Role::builder()` to construct roles incrementally", "Ensure the role name is unique within the registry", "Reference only known permissions in the permission set", "Avoid cycles in role inheritance"],
        example_bad: Some("RoleRegistry::register(Role { name: \"\", permissions: [] });"),
        example_good: Some("RoleRegistry::register(Role::builder().name(\"Chair\").permission(\"vote\").build());"),
        see_also: &["HHS_E_ROLE_PERMISSION_NOT_FOUND", "HHS_E_COUNCIL_INVALID_ROLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const ROLE_PERMISSION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(222),
        long_code: "HHS_E_ROLE_PERMISSION_NOT_FOUND",
        short_code: "E0222",
        title: "Permission not found on role",
        short_description: "Looked up or required a permission that the role does not grant.",
        long_description: "A role grants a set of named permissions. Operations that consult the role for authorization — `role.has_permission(name)` in strict mode, or `role.require_permission(name)` — raise this error when the permission is not in the grant set.

This is distinct from `RoleInvalidRole`: the role itself is valid, but it lacks the specific permission requested. Use this error to drive authorization decisions: catch it to deny access, or fix the role definition if the missing permission is an oversight.

For agent-level checks, prefer `agent.role().has_permission(name)` over the throwing variant when you want a boolean.",
        hints: &["Use `role.has_permission(name)` for non-throwing checks", "Audit the role's grant set if a permission is unexpectedly missing", "Add the permission via `role.grant(name)` if appropriate", "Distinguish missing permissions from invalid role descriptors"],
        example_bad: Some("observer_role.require_permission(\"vote\");"),
        example_good: Some("if observer_role.has_permission(\"vote\") {
  cast_vote();
}"),
        see_also: &["HHS_E_ROLE_INVALID_ROLE", "HHS_E_ROLE_ROLE_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const ROLE_ROLE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(223),
        long_code: "HHS_E_ROLE_ROLE_NOT_FOUND",
        short_code: "E0223",
        title: "Role not found in registry",
        short_description: "Looked up a role by name in a registry that does not contain it.",
        long_description: "The role registry stores all role definitions for a governance scope. Looking up a role by name — for assignment, inspection, or permission checking — raises this error when the name is unknown.

The usual causes are: forgetting to register the role before referencing it, name typos, and referencing a role from a different governance scope. Role names are case-sensitive.

Register the role first, then look it up. List the current registry contents with `RoleRegistry::roles()` while debugging.",
        hints: &["Register roles at startup before any consumer references them", "Use `RoleRegistry::has(name)` for non-throwing checks", "List registered roles with `RoleRegistry::roles()`", "Names are case-sensitive"],
        example_bad: Some("let r = RoleRegistry::get(\"chair\");  // registered as \"Chair\""),
        example_good: Some("let r = RoleRegistry::get(\"Chair\");"),
        see_also: &["HHS_E_ROLE_INVALID_ROLE", "HHS_E_ROLE_PERMISSION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_AGENT_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(270),
        long_code: "HHS_E_SWARM_AGENT_FAILED",
        short_code: "E0270",
        title: "Swarm agent execution failed",
        short_description: "An individual agent within a swarm raised an error during its parallel execution.",
        long_description: "A swarm runs its members in parallel and collects their results. When an individual agent's execution raises, the swarm captures the failure and decides — based on its consensus policy — whether the overall run can still succeed. This error reports the per-agent failure; whether it propagates depends on the swarm's policy.

Under `first-wins` policies, a single failure may not abort the swarm if another agent succeeds in time. Under strict policies, any failure aborts. The error message wraps the agent's underlying cause and includes the failing agent's ID.

Use this error in concert with `SwarmInsufficientSuccess` to distinguish individual failures from quorum failures.",
        hints: &["Inspect the wrapped cause and the failing agent ID", "Choose a consensus policy that tolerates per-agent failures if appropriate", "Pair with `SwarmInsufficientSuccess` to detect quorum failures", "Add per-agent retries before treating failures as fatal"],
        example_bad: Some("swarm.run(input);  // one agent panics, no recovery configured"),
        example_good: Some("swarm.with_policy(Consensus::Majority).run(input);"),
        see_also: &["HHS_E_SWARM_INSUFFICIENT_SUCCESS", "HHS_E_SWARM_TIMEOUT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_AGENT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(271),
        long_code: "HHS_E_SWARM_AGENT_NOT_FOUND",
        short_code: "E0271",
        title: "Agent not in swarm",
        short_description: "Tried to address an agent by ID that is not a member of the swarm.",
        long_description: "A swarm's membership is fixed at construction (or extended via `swarm.add_agent`). Operations targeting a specific agent — `swarm.agent(id)`, `swarm.remove_agent(id)`, `swarm.result(id)` — raise this error when the ID is not in the swarm.

This is the per-swarm form of the missing-agent condition. The agent may exist elsewhere in the runtime; it is simply not part of *this* swarm.

Verify membership with `swarm.has_agent(id)` before targeted calls. If you need to enumerate members, use `swarm.agents()`.",
        hints: &["Use `swarm.has_agent(id)` to test membership", "Enumerate members with `swarm.agents()`", "Add the agent first with `swarm.add_agent(...)` if intended", "Confirm you are addressing the right swarm — multiple may exist"],
        example_bad: Some("swarm.result(stranger_id);"),
        example_good: Some("if swarm.has_agent(id) {
  swarm.result(id);
}"),
        see_also: &["HHS_E_SWARM_DUPLICATE_AGENT", "HHS_E_GOVERNANCE_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_DUPLICATE_AGENT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(272),
        long_code: "HHS_E_SWARM_DUPLICATE_AGENT",
        short_code: "E0272",
        title: "Agent already in swarm",
        short_description: "Tried to add an agent to a swarm that already includes it.",
        long_description: "Swarm membership is a set — each agent ID appears at most once. `swarm.add_agent(agent)` raises this error if the agent is already in the swarm. The constraint exists because consensus tallies (majority, average) and result indexing rely on each agent counting exactly once.

If you intend to replace an agent's configuration, remove it first with `swarm.remove_agent(id)` and re-add. If your input list may contain duplicates, deduplicate before construction.",
        hints: &["Deduplicate seed lists before constructing a swarm", "Use `swarm.has_agent(id)` before adding", "Remove first if you intend to replace an agent's configuration", "Use `Set` rather than `List` to assemble swarm membership"],
        example_bad: Some("swarm.add_agent(worker_a);
swarm.add_agent(worker_a);"),
        example_good: Some("if !swarm.has_agent(worker_a.id) {
  swarm.add_agent(worker_a);
}"),
        see_also: &["HHS_E_SWARM_AGENT_NOT_FOUND", "HHS_E_COUNCIL_DUPLICATE_AGENT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_INSUFFICIENT_SUCCESS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(273),
        long_code: "HHS_E_SWARM_INSUFFICIENT_SUCCESS",
        short_code: "E0273",
        title: "Swarm did not meet success quorum",
        short_description: "Fewer agents succeeded than the swarm's consensus policy required.",
        long_description: "Swarm consensus policies (majority, threshold, weighted) define a minimum number of successful agent results required for the swarm to declare overall success. When the run completes — either because all agents finished or because the timeout expired — and the success count falls short of that requirement, this error is raised.

The error message includes the required count and the actual count. This is distinct from `SwarmAgentFailed` (which reports an individual agent's failure) and `SwarmTimeout` (which reports a wall-clock failure): you may see this error after a mix of failures and successes that simply didn't add up.

Fix by lowering the threshold, increasing per-agent reliability, adding more agents to the swarm, or extending the timeout to give slow agents a chance to complete.",
        hints: &["Lower the success threshold if your task tolerates fewer responses", "Investigate which agents are failing and why", "Add more agents to absorb individual failures", "Extend the timeout if slow agents are being cut off"],
        example_bad: Some("swarm.with_policy(Consensus::Threshold(10)).run(input);  // only 8 agents in swarm"),
        example_good: Some("swarm.with_policy(Consensus::Threshold(5)).run(input);"),
        see_also: &["HHS_E_SWARM_AGENT_FAILED", "HHS_E_SWARM_TIMEOUT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_NO_AGENTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(274),
        long_code: "HHS_E_SWARM_NO_AGENTS",
        short_code: "E0274",
        title: "Swarm has no agents",
        short_description: "An operation requiring at least one member was performed on an empty swarm.",
        long_description: "A swarm with zero agents cannot run, cannot reach consensus, and cannot return a result. Calling `swarm.run(input)` or any consensus operation on an empty swarm raises this error.

The condition typically arises during construction — an empty seed list — or after bulk removal that emptied the swarm. Constructors that take a member list will reject empty input at build time when configured to require non-empty membership.

This is the swarm analogue of `CouncilNoMembers`.",
        hints: &["Seed swarms with at least one agent at construction", "Guard `swarm.run` with `swarm.agent_count() > 0`", "Avoid bulk removals that empty the swarm mid-flight", "Configure a `min_agents` invariant for safety"],
        example_bad: Some("let swarm = Swarm::new(\"workers\", []);
swarm.run(input);"),
        example_good: Some("let swarm = Swarm::new(\"workers\", [w1, w2, w3]);
swarm.run(input);"),
        see_also: &["HHS_E_COUNCIL_NO_MEMBERS", "HHS_E_SWARM_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_STATE_KEY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(275),
        long_code: "HHS_E_SWARM_STATE_KEY_NOT_FOUND",
        short_code: "E0275",
        title: "Swarm state key not found",
        short_description: "Looked up a key in the swarm's shared state map that has no entry.",
        long_description: "A swarm exposes a shared key/value state map for coordination among its agents. `swarm.state(key)` raises this error when the key has no associated value — either because nothing has ever been written under that key, because it was deleted, or because of a typo.

Unlike resource registries, the state map is intended for transient coordination data. Treat missing keys as a normal part of lifecycle: prefer `swarm.state_get(key)` (returning an option) over the throwing form when the absence is expected.

If you are seeing this error consistently for a key that should exist, audit your write/read ordering — another agent may be reading before the producer has written.",
        hints: &["Use `swarm.state_get(key)` for the option-returning variant", "Audit producer/consumer ordering — reads may precede writes", "Initialize required keys at swarm construction", "Keys are case-sensitive strings"],
        example_bad: Some("let cfg = swarm.state(\"config\");  // never written"),
        example_good: Some("swarm.state_set(\"config\", initial_config);
let cfg = swarm.state(\"config\");"),
        see_also: &["HHS_E_SWARM_AGENT_NOT_FOUND", "HHS_E_COMMUNITY_RESOURCE_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const SWARM_TIMEOUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(276),
        long_code: "HHS_E_SWARM_TIMEOUT",
        short_code: "E0276",
        title: "Swarm execution timed out",
        short_description: "The swarm did not complete within its configured time budget.",
        long_description: "Each swarm run has a wall-clock time budget. If agents collectively fail to produce enough results to satisfy the consensus policy before the budget expires, the swarm aborts and raises this error. In-flight agents are cancelled.

This is distinct from `SwarmInsufficientSuccess`, which is raised when the run completes but the success count is too low. A timeout means the run never had a chance to finish; insufficient success means it finished but disappointingly.

Fix by raising the timeout, parallelizing more aggressively, removing slow stragglers, or adopting a faster consensus policy (e.g., `first-wins`).",
        hints: &["Raise the timeout via `swarm.set_timeout(dur)`", "Use a `first-wins` consensus policy if any successful answer suffices", "Investigate slow agents — log per-agent latencies", "Distinguish from `SwarmInsufficientSuccess` — timeout means the run was cut short"],
        example_bad: Some("swarm.run_with_timeout(input, 100.millis);"),
        example_good: Some("swarm.run_with_timeout(input, 30.seconds);"),
        see_also: &["HHS_E_SWARM_INSUFFICIENT_SUCCESS", "HHS_E_COUNCIL_TIMEOUT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    COMMUNITY_COUNCIL_NOT_FOUND,
    COMMUNITY_DUPLICATE_COUNCIL,
    COMMUNITY_DUPLICATE_MEMBER,
    COMMUNITY_DUPLICATE_RESOURCE,
    COMMUNITY_INVALID_CONFIGURATION,
    COMMUNITY_MEMBER_NOT_FOUND,
    COMMUNITY_RESOURCE_NOT_FOUND,
    CONSTITUTION_INVALID_VERSION,
    CONSTITUTION_LAW_NOT_FOUND,
    CONSTITUTION_NO_PREVIOUS_VERSION,
    CONSTITUTION_NOT_FOUND,
    COUNCIL_AGENT_NOT_FOUND,
    COUNCIL_CONSTITUTION_NOT_FOUND,
    COUNCIL_DUPLICATE_AGENT,
    COUNCIL_EXECUTION_FAILED,
    COUNCIL_INVALID_ROLE,
    COUNCIL_NO_MEMBERS,
    COUNCIL_NOT_FOUND,
    COUNCIL_TIMEOUT,
    COUP_AGENT_NOT_FOUND,
    COUP_CONSTITUTION_NOT_FOUND,
    GOVERNANCE_AGENT_NOT_FOUND,
    GOVERNANCE_CACHE_ID_COLLISION,
    GOVERNANCE_CIRCULAR_DEPENDENCY,
    GOVERNANCE_CONSTITUTION_NOT_FOUND,
    GOVERNANCE_FORMAT_VALIDATION,
    GOVERNANCE_INVALID_CONFIGURATION,
    GOVERNANCE_INVALID_ROLE,
    GOVERNANCE_RESOURCE_NOT_FOUND,
    GOVERNANCE_SERIALIZATION_ERROR,
    PERSPECTIVE_FIELD_HIDDEN,
    PERSPECTIVE_WRITE_ACCESS_DENIED,
    ROLE_INVALID_ROLE,
    ROLE_PERMISSION_NOT_FOUND,
    ROLE_ROLE_NOT_FOUND,
    SWARM_AGENT_FAILED,
    SWARM_AGENT_NOT_FOUND,
    SWARM_DUPLICATE_AGENT,
    SWARM_INSUFFICIENT_SUCCESS,
    SWARM_NO_AGENTS,
    SWARM_STATE_KEY_NOT_FOUND,
    SWARM_TIMEOUT,
];
