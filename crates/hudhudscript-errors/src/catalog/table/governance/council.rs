use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const COUNCIL_INVALID_ROLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(56),
        long_code: "HHS_E_COUNCIL_INVALID_ROLE",
        short_code: "E0056",
        title: "Invalid role for council operation",
        short_description: "An operation was attempted with a role value the council does not recognize or permit.",
        long_description: "Councils accept a fixed set of roles (Chair, Member, Observer, plus any roles registered via `RoleRegistry`). This error is raised when an operation passes an unknown role name, when a role assignment violates a council invariant (for example, attempting to demote the only Chair when the council requires one), or when a member tries an action their role is not authorized to perform.

The error message identifies which role and which constraint was violated. Refer to the council's role policy to see the valid set.

For permission checks specifically, this error overlaps with `RolePermissionNotFound`; the council variant fires when the *role itself* is invalid, while the role variant fires when the role is valid but lacks the requested permission.",
        hints: &["List valid roles with `council.role_registry().roles()`", "Don't demote the last Chair without first promoting another member", "Use the registered role enum rather than arbitrary strings", "Check `RoleInvalidRole` for the role-registry-level form"],
        example_bad: Some("council.assign_role(alice, \"OverlordSupreme\");"),
        example_good: Some("council.assign_role(alice, Role::Chair);"),
        see_also: &["HHS_E_ROLE_INVALID_ROLE", "HHS_E_GOVERNANCE_INVALID_ROLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_NO_MEMBERS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(57),
        long_code: "HHS_E_COUNCIL_NO_MEMBERS",
        short_code: "E0057",
        title: "Council has no members",
        short_description: "An operation requiring at least one member was performed on an empty council.",
        long_description: "A council with zero members cannot vote, cannot establish quorum, and has no Chair to drive proceedings. Calling `council.run(proposal)`, `council.cast_vote(...)`, or any quorum-checking operation on an empty council raises this error.

The condition typically arises during construction — forgetting to seed members — or after bulk removal that emptied the roster. Council constructors that take a member list will reject empty input at build time when configured to require non-empty membership.

For swarms, the analogous error is `SwarmNoAgents`.",
        hints: &["Seed councils with at least one member at construction", "Guard `council.run` with `council.member_count() > 0`", "Avoid bulk removals that empty the roster mid-flight", "Consider a `min_members` invariant in the council config"],
        example_bad: Some("let council = Council::new(\"empty\", []);
council.run(proposal);"),
        example_good: Some("let council = Council::new(\"safety\", [alice, bob, carol]);
council.run(proposal);"),
        see_also: &["HHS_E_SWARM_NO_AGENTS", "HHS_E_COUNCIL_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(58),
        long_code: "HHS_E_COUNCIL_NOT_FOUND",
        short_code: "E0058",
        title: "Council not found in registry",
        short_description: "Looked up a council by name in a registry that has no entry under that name.",
        long_description: "The orchestration runtime keeps a registry of councils that have been declared in the program. Operations like `governance.council(name)`, `runtime.run_council(name, proposal)`, and cross-community routing rely on this registry. This error is raised when the name does not resolve.

Unlike `CommunityCouncilNotFound`, which is scoped to a specific community, this error is reported by the global orchestration registry. A council may be present in a community while still being absent from the global registry if it was never published.

Fix by registering the council before lookup, or by switching to a community-scoped lookup if that is the intended scope.",
        hints: &["Register the council with `governance.register_council(council)` before lookup", "Distinguish global vs. community-scoped lookups", "Council names are case-sensitive", "Use `governance.councils()` to list registered names"],
        example_bad: Some("let result = governance.council(\"reviw-board\").run(proposal);"),
        example_good: Some("governance.register_council(review_board);
let result = governance.council(\"review-board\").run(proposal);"),
        see_also: &["HHS_E_COMMUNITY_COUNCIL_NOT_FOUND", "HHS_E_COUNCIL_NO_MEMBERS"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_TIMEOUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(59),
        long_code: "HHS_E_COUNCIL_TIMEOUT",
        short_code: "E0059",
        title: "Council decision timed out",
        short_description: "A council failed to reach a decision within its configured time budget.",
        long_description: "Each council run has a time budget — either inherited from the council's configuration or supplied per-call via `council.run_with_timeout(proposal, dur)`. If voting members do not respond, deliberation does not converge, or downstream waits stall the run beyond that budget, this error is raised.

A timeout finalizes the council in the failed state; in-flight votes are abandoned. The proposal is *not* automatically retried — that is the caller's choice. For long-deliberation councils, raise the timeout or restructure the decision into smaller proposals.

Timeouts are wall-clock and not paused for agent backoff; treat them as a circuit breaker rather than a precise scheduling tool.",
        hints: &["Increase the timeout via `council.set_timeout(dur)` for slow members", "Investigate which member or step is stalling — log per-vote latencies", "Break large proposals into smaller, faster decisions", "Treat timeout as terminal — explicitly retry if appropriate"],
        example_bad: Some("council.run_with_timeout(huge_proposal, 1.seconds);"),
        example_good: Some("council.run_with_timeout(huge_proposal, 30.seconds);"),
        see_also: &["HHS_E_SWARM_TIMEOUT", "HHS_E_COUNCIL_EXECUTION_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUP_AGENT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(60),
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
        category: ErrorCategory::Governance,
    };

pub const COUP_CONSTITUTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(61),
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
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_AGENT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(93),
        long_code: "HHS_E_GOVERNANCE_AGENT_NOT_FOUND",
        short_code: "E0093",
        title: "Agent not registered with governance",
        short_description: "A governance API call referenced an agent ID that the governance system does not know about.",
        long_description: "The governance facade keeps a registry of every agent participating in councils, swarms, communities, and coups. Calls that resolve an agent by ID — assigning a role, querying perspective, or routing a directive — raise this error when the ID is absent.

This is the top-level form of the missing-agent condition; specialized variants exist on `Council`, `Swarm`, `Community`, and `Coup`. They report the same logical fact at the layer that detected it. When you see this top-level form, the missing agent was never registered or has been globally deregistered.

Register agents with `governance.register_agent(agent)` before referencing them, and avoid removing agents that other governance entities still hold references to.",
        hints: &["Register agents with `governance.register_agent(agent)` at startup", "Use `governance.has_agent(id)` for non-throwing checks", "Coordinate removals — councils and swarms may still reference the agent", "Inspect the ID in the message for typos"],
        example_bad: Some("governance.assign_role(unregistered_id, Role::Chair);"),
        example_good: Some("governance.register_agent(alice);
governance.assign_role(alice.id, Role::Chair);"),
        see_also: &["HHS_E_COUNCIL_AGENT_NOT_FOUND", "HHS_E_COMMUNITY_MEMBER_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_CACHE_ID_COLLISION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(94),
        long_code: "HHS_E_GOVERNANCE_CACHE_ID_COLLISION",
        short_code: "E0094",
        title: "Governance cache ID collision",
        short_description: "Two distinct governance objects hashed to the same internal cache key.",
        long_description: "The governance subsystem maintains an internal cache keyed by stable IDs derived from object identity. A collision indicates that two logically distinct objects produced the same key — typically because of a malformed ID generator, manual ID assignment that bypassed the allocator, or deserialization from an inconsistent source.

This is an internal-consistency error and should be treated as a bug. The cache cannot disambiguate the colliding entries, so behavior beyond this point is undefined and the operation is aborted.

If you are constructing IDs by hand, switch to the allocator. If you are loading a snapshot, verify that the snapshot was produced by a compatible runtime version.",
        hints: &["Don't construct governance IDs manually — use the allocator", "Verify snapshot/runtime version compatibility on load", "File a bug report if this occurs without manual ID manipulation", "Check for accidental object cloning that reused an ID"],
        example_bad: Some("let a = Agent { id: AgentId(42), .. };
let b = Agent { id: AgentId(42), .. };  // collision"),
        example_good: Some("let a = governance.new_agent();
let b = governance.new_agent();"),
        see_also: &["HHS_E_GOVERNANCE_SERIALIZATION_ERROR", "HHS_E_GOVERNANCE_INVALID_CONFIGURATION"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_CIRCULAR_DEPENDENCY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(95),
        long_code: "HHS_E_GOVERNANCE_CIRCULAR_DEPENDENCY",
        short_code: "E0095",
        title: "Circular dependency in governance graph",
        short_description: "A constitution, council, or community dependency chain forms a cycle.",
        long_description: "Governance entities may declare dependencies on each other: a constitution can extend another, a council can defer to a parent council, a community can inherit policy from a base community. The dependency graph must be a DAG. When adding an edge that would close a cycle, this error is raised and the edge is rejected.

Cycles are forbidden because resolution algorithms (rule lookup, version chasing, decision delegation) walk the dependency graph and a cycle would cause unbounded recursion or non-deterministic outcomes.

The error message names the entity whose addition closed the cycle. Refactor by introducing a shared base instead of mutual dependence, or by inverting the direction of one edge.",
        hints: &["Introduce a shared base instead of two entities depending on each other", "Visualize the dependency graph with `governance.dependency_graph()`", "Invert one edge to break the cycle", "Avoid late-binding `extend` calls that close loops"],
        example_bad: Some("a.extend(b);
b.extend(a);  // cycle"),
        example_good: Some("let base = Constitution::new(\"shared\");
a.extend(base);
b.extend(base);"),
        see_also: &["HHS_E_GOVERNANCE_INVALID_CONFIGURATION", "HHS_E_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_CONSTITUTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(96),
        long_code: "HHS_E_GOVERNANCE_CONSTITUTION_NOT_FOUND",
        short_code: "E0096",
        title: "Constitution not found in governance registry",
        short_description: "A governance call referenced a constitution name that is not in the global registry.",
        long_description: "The governance facade owns the canonical constitution registry. Any call that resolves a constitution by name through the top-level governance API — typically when wiring councils, communities, or coups — raises this error if the name is unknown.

This is the global counterpart to `ConstitutionNotFound`, `CouncilConstitutionNotFound`, and `CoupConstitutionNotFound`. They all describe the same missing-name condition reported from different layers; this one originates at the registry itself.

Register the constitution before referencing it. If you are loading constitutions from disk or a remote store, ensure the load completes before any consumer runs.",
        hints: &["Register with `governance.add_constitution(c)` before referencing", "Wait for async constitution loaders to complete during startup", "List registered names with `governance.constitutions()`", "Names are case-sensitive"],
        example_bad: Some("governance.bind_council(\"safety\", constitution: \"missing\");"),
        example_good: Some("governance.add_constitution(safety_rules);
governance.bind_council(\"safety\", constitution: \"safety-rules\");"),
        see_also: &["HHS_E_CONSTITUTION_NOT_FOUND", "HHS_E_COUNCIL_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_FORMAT_VALIDATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(97),
        long_code: "HHS_E_GOVERNANCE_FORMAT_VALIDATION",
        short_code: "E0097",
        title: "Governance format validation failed",
        short_description: "A governance object failed structural format validation during parse or load.",
        long_description: "When governance objects (constitutions, councils, communities, role definitions) are loaded from a serialized form — JSON, on-disk snapshots, network payloads — the runtime validates their structure before admitting them. This error is raised when a required field is missing, a field has the wrong type, or the document violates a structural invariant.

The error message contains the field path and the specific reason. Treat this as a data-correctness issue; the offending document must be fixed at its source.

Validation errors here are distinct from semantic errors like `GovernanceInvalidConfiguration`: this one fires before the object even reaches the configuration layer.",
        hints: &["Read the field path in the message — it points at the bad field", "Validate documents against the schema before loading at runtime", "Check producer/consumer version compatibility", "Use the typed builders rather than raw JSON when authoring objects"],
        example_bad: Some("governance.load_constitution_json(\"{ \\\"name\\\": \\\"x\\\" }\");  // missing version"),
        example_good: Some("governance.load_constitution_json(\"{ \\\"name\\\": \\\"x\\\", \\\"version\\\": \\\"1.0.0\\\", \\\"laws\\\": [] }\");"),
        see_also: &["HHS_E_GOVERNANCE_INVALID_CONFIGURATION", "HHS_E_GOVERNANCE_SERIALIZATION_ERROR"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_INVALID_CONFIGURATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(98),
        long_code: "HHS_E_GOVERNANCE_INVALID_CONFIGURATION",
        short_code: "E0098",
        title: "Invalid governance configuration",
        short_description: "Governance configuration failed semantic validation after parsing.",
        long_description: "After a governance object passes structural format checks, it is validated against semantic rules — for example, that quorums do not exceed membership, that role names are unique, that constitutional dependencies form a DAG, and that referenced agents exist. This error is raised when one of those rules is violated.

The message identifies the offending field and the rule it violated. Unlike `GovernanceFormatValidation`, the document is structurally well-formed; the error is in its meaning.

Fix the configuration and reload. For programmatic construction, prefer the typed builders, which catch most invalid combinations at compile-time.",
        hints: &["Read the message — it specifies the violated rule", "Use typed builders to catch errors earlier", "Validate cross-references (agents, constitutions) at load time", "Avoid ad-hoc edits to governance documents — use migration scripts"],
        example_bad: Some("Council::config(quorum: 5, members: [alice, bob]);  // quorum > members"),
        example_good: Some("Council::config(quorum: 2, members: [alice, bob]);"),
        see_also: &["HHS_E_GOVERNANCE_FORMAT_VALIDATION", "HHS_E_COMMUNITY_INVALID_CONFIGURATION"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_INVALID_ROLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(99),
        long_code: "HHS_E_GOVERNANCE_INVALID_ROLE",
        short_code: "E0099",
        title: "Invalid role at governance layer",
        short_description: "A role name or descriptor was rejected by the governance registry.",
        long_description: "The governance facade validates role names against the registered role registry. When a role descriptor refers to a name that has not been registered, or when its permission set is malformed, this error is raised.

This is the top-level form of the role error family. `RoleInvalidRole` and `CouncilInvalidRole` report the same condition from the role-registry and council layers respectively. The governance variant is the one you will see when binding roles to entities through the top-level governance API.

Register the role before binding it, or correct the descriptor's permission set.",
        hints: &["Register custom roles with `governance.role_registry().register(role)`", "List valid roles via `governance.role_registry().roles()`", "Check spelling — role names are case-sensitive", "Validate role descriptors against the registry schema"],
        example_bad: Some("governance.assign_role(alice, \"Wizard\");  // not registered"),
        example_good: Some("governance.role_registry().register(Role::new(\"Wizard\", permissions));
governance.assign_role(alice, \"Wizard\");"),
        see_also: &["HHS_E_ROLE_INVALID_ROLE", "HHS_E_COUNCIL_INVALID_ROLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_RESOURCE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(100),
        long_code: "HHS_E_GOVERNANCE_RESOURCE_NOT_FOUND",
        short_code: "E0100",
        title: "Governance resource not found",
        short_description: "A globally scoped governance resource lookup did not match any registered key.",
        long_description: "Governance maintains a registry of shared resources that span councils and communities — for example, audit logs, broadcast channels, and policy stores. When a top-level governance call resolves a resource by key and finds nothing, this error is raised.

This is the global counterpart to `CommunityResourceNotFound`. The community variant is scoped to a single community's local resource map, while this one is reported from the global governance registry.

Register the resource through `governance.register_resource(key, resource)` before referencing it, and avoid removing resources that other entities still depend on.",
        hints: &["Register resources at startup with `governance.register_resource(...)`", "Use `governance.has_resource(key)` to check existence", "Distinguish global vs. community-scoped resource maps", "Avoid removing resources while consumers still hold references"],
        example_bad: Some("let log = governance.resource(\"audit-log\");  // never registered"),
        example_good: Some("governance.register_resource(\"audit-log\", audit_log);
let log = governance.resource(\"audit-log\");"),
        see_also: &["HHS_E_COMMUNITY_RESOURCE_NOT_FOUND", "HHS_E_GOVERNANCE_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const GOVERNANCE_SERIALIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(101),
        long_code: "HHS_E_GOVERNANCE_SERIALIZATION_ERROR",
        short_code: "E0101",
        title: "Governance serialization or deserialization failed",
        short_description: "Encoding or decoding a governance object failed at the serialization layer.",
        long_description: "Governance objects can be serialized for snapshotting, IPC, and persistence. This error is raised when the underlying serializer or deserializer fails — for example, due to a corrupted byte stream, an incompatible schema version, or a Rust-side encoding error wrapped from `serde`.

The wrapped cause is included in the message. Treat this as a data-integrity issue: the object cannot be safely produced or consumed in its current form.

For version mismatches, run a migration. For corruption, restore from a known-good snapshot. For encoding bugs in custom types, fix the implementation rather than working around it.",
        hints: &["Inspect the wrapped cause for the underlying serde error", "Run schema migration if producer and consumer versions differ", "Restore from a known-good snapshot if data is corrupted", "Verify custom `Serialize`/`Deserialize` impls round-trip"],
        example_bad: Some("governance.load_snapshot(corrupted_bytes);"),
        example_good: Some("let bytes = governance.snapshot();
governance.load_snapshot(bytes);"),
        see_also: &["HHS_E_GOVERNANCE_FORMAT_VALIDATION", "HHS_E_GOVERNANCE_CACHE_ID_COLLISION"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };
