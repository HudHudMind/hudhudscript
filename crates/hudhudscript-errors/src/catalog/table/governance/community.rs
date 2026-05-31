use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const COMMUNITY_COUNCIL_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(27),
        long_code: "HHS_E_COMMUNITY_COUNCIL_NOT_FOUND",
        short_code: "E0027",
        title: "Council not associated with this community",
        short_description: "Looked up a council inside a community that does not host it.",
        long_description: "A `Community` keeps an index of the councils that have been attached to it via `community.attach_council(...)`. Operations such as `community.council(name)`, `community.detach_council(name)`, or routing a proposal to a named council all consult this index. This error is raised when the requested council name has no entry in that index for the given community.

The most common cause is referring to a council before it has been attached, after it has been detached, or via a typo in the council's name. Council names are case-sensitive and must match exactly.

This error is local to a single community — a council with that name may exist elsewhere in the program. Use `GovernanceConstitutionNotFound`-style global lookups only when you need cross-community search.",
        hints: &["Verify the council was attached with `community.attach_council(council)` before lookup", "Council names are case-sensitive — check spelling and casing", "Use `community.has_council(name)` to test existence without raising", "List `community.councils()` while debugging to see what is actually attached"],
        example_bad: Some("let comm = Community::new(\"engineering\");
let decision = comm.council(\"design-review\").vote(proposal);"),
        example_good: Some("let comm = Community::new(\"engineering\");
comm.attach_council(Council::new(\"design-review\", members));
let decision = comm.council(\"design-review\").vote(proposal);"),
        see_also: &["HHS_E_COMMUNITY_DUPLICATE_COUNCIL", "HHS_E_COUNCIL_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_COUNCIL: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(28),
        long_code: "HHS_E_COMMUNITY_DUPLICATE_COUNCIL",
        short_code: "E0028",
        title: "Council already attached to community",
        short_description: "Attempted to attach a council whose name already exists in the community's council index.",
        long_description: "Within a single community, council names must be unique. When `community.attach_council(council)` is called with a council whose name collides with one already attached, this error is raised rather than silently replacing the existing entry. Names are the addressable handle that proposals and observers use to reach a council, so collisions would create routing ambiguity.

This is distinct from sharing a council across communities (which is allowed): the uniqueness constraint applies per-community, not globally.

If the goal is to replace an existing council, detach the old one first with `community.detach_council(name)` and then attach the new instance.",
        hints: &["Use a unique name when constructing each council", "Detach before re-attach: `comm.detach_council(name); comm.attach_council(new)`", "Use `community.has_council(name)` to guard the attach call", "If you need multiple councils with the same role, namespace them (e.g. `review-frontend`, `review-backend`)"],
        example_bad: Some("comm.attach_council(Council::new(\"review\", group_a));
comm.attach_council(Council::new(\"review\", group_b));"),
        example_good: Some("comm.attach_council(Council::new(\"review-frontend\", group_a));
comm.attach_council(Council::new(\"review-backend\", group_b));"),
        see_also: &["HHS_E_COMMUNITY_COUNCIL_NOT_FOUND", "HHS_E_COUNCIL_DUPLICATE_AGENT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_MEMBER: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(29),
        long_code: "HHS_E_COMMUNITY_DUPLICATE_MEMBER",
        short_code: "E0029",
        title: "Agent already a member of this community",
        short_description: "Attempted to add an agent to a community that already lists it as a member.",
        long_description: "Communities track membership as a set — each agent ID may appear at most once. This error fires when `community.add_member(id)` (or the constructor's seed list) is given an agent ID that is already present.

Uniqueness is enforced because membership drives broadcast fan-out, voting eligibility, and resource sharing. A duplicate would skew quorum math and double-count messages.

If you are bulk-loading members from an external source, deduplicate the list before passing it in. If you simply want add-or-ignore semantics, gate the call with `community.is_member(id)`.",
        hints: &["Check `community.is_member(id)` before calling `add_member`", "Deduplicate seed lists when constructing a community", "Constructors validate uniqueness — pass each ID exactly once", "Use a `Set` rather than a `List` to assemble the input"],
        example_bad: Some("let comm = Community::new(\"team\", [agent1, agent1, agent2]);"),
        example_good: Some("let comm = Community::new(\"team\", [agent1, agent2]);"),
        see_also: &["HHS_E_COMMUNITY_MEMBER_NOT_FOUND", "HHS_E_COUNCIL_DUPLICATE_AGENT"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_RESOURCE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(30),
        long_code: "HHS_E_COMMUNITY_DUPLICATE_RESOURCE",
        short_code: "E0030",
        title: "Resource already registered in community",
        short_description: "Tried to register a shared resource whose key already exists in the community.",
        long_description: "Communities can own shared resources (datasets, tools, channels) keyed by name. The resource registry enforces unique keys so that lookups by name are unambiguous. When `community.add_resource(key, resource)` is called with a key that is already taken, this error is raised.

This prevents accidental shadowing of an existing resource and catches name collisions early — particularly useful when several modules register resources during community bootstrap.

To replace an existing resource, remove it first with `community.remove_resource(key)` or use `community.replace_resource(key, new)` if available.",
        hints: &["Pick a more specific key — namespace by purpose (e.g. `db.users`, `db.orders`)", "Call `community.has_resource(key)` before adding", "Remove the existing resource first if replacement is intended", "Audit bootstrap order: two modules may both be registering the same key"],
        example_bad: Some("comm.add_resource(\"db\", primary_db);
comm.add_resource(\"db\", replica_db);"),
        example_good: Some("comm.add_resource(\"db.primary\", primary_db);
comm.add_resource(\"db.replica\", replica_db);"),
        see_also: &["HHS_E_COMMUNITY_RESOURCE_NOT_FOUND", "HHS_E_COMMUNITY_DUPLICATE_MEMBER"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_INVALID_CONFIGURATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(31),
        long_code: "HHS_E_COMMUNITY_INVALID_CONFIGURATION",
        short_code: "E0031",
        title: "Invalid community configuration",
        short_description: "The configuration supplied to a community failed structural or semantic validation.",
        long_description: "Community configuration includes fields such as the cultural profile (values, norms, communication style), membership policy, quorum thresholds, and default constitution. When any of these fields is missing, malformed, or inconsistent (for example, a quorum greater than the maximum membership), the community refuses to be constructed and raises this error.

The message string carries the specific field and reason. Treat this as a configuration bug rather than a runtime condition: the community will not function until the configuration is corrected.

Validation runs both at construction and on `community.update_config(...)`, so retroactive changes are checked too.",
        hints: &["Read the message — the offending field and reason are included", "Validate quorum settings against the seed member count", "Ensure the referenced default constitution exists before passing it", "Use `CommunityConfig::builder()` to catch missing required fields at compile-time"],
        example_bad: Some("let comm = Community::new(\"team\", CommunityConfig { quorum: 10, members: [a, b, c], ..default });"),
        example_good: Some("let comm = Community::new(\"team\", CommunityConfig { quorum: 2, members: [a, b, c], ..default });"),
        see_also: &["HHS_E_GOVERNANCE_INVALID_CONFIGURATION", "HHS_E_GOVERNANCE_FORMAT_VALIDATION"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_MEMBER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(32),
        long_code: "HHS_E_COMMUNITY_MEMBER_NOT_FOUND",
        short_code: "E0032",
        title: "Agent not a member of this community",
        short_description: "Operation referenced a member ID that the community has no record of.",
        long_description: "Member-targeted operations — `remove_member`, `assign_role`, `member_perspective`, `send_to_member` — all require the agent to be a current member of the community. This error fires when the supplied ID is not in the membership set.

The usual causes are: removing the same member twice, referring to an agent that was never added, race conditions where the member was concurrently removed, or typos in agent IDs (which are opaque strings and easy to corrupt).

This is the inverse of `CommunityDuplicateMember`: that one fires on add, this one fires on lookup or remove.",
        hints: &["Use `community.is_member(id)` before mutating operations", "Treat `remove_member` as idempotent in your wrapper layer if needed", "Log the agent ID — opaque IDs are easy to mistype", "Audit ordering: another task may have removed the agent first"],
        example_bad: Some("comm.remove_member(agent_id);
comm.remove_member(agent_id);  // second call fails"),
        example_good: Some("if comm.is_member(agent_id) {
  comm.remove_member(agent_id);
}"),
        see_also: &["HHS_E_COMMUNITY_DUPLICATE_MEMBER", "HHS_E_GOVERNANCE_AGENT_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COMMUNITY_RESOURCE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(33),
        long_code: "HHS_E_COMMUNITY_RESOURCE_NOT_FOUND",
        short_code: "E0033",
        title: "Resource key not registered in community",
        short_description: "Looked up a resource by key in a community that has no entry under that key.",
        long_description: "Each community owns a keyed registry of shared resources. When `community.resource(key)` or `community.remove_resource(key)` is called with a key that has never been added — or has already been removed — this error is raised.

Resources are not inherited between communities, so a key that exists in one community is invisible to another. Cross-community sharing must be done explicitly, either by registering the same resource in both communities or by passing a handle through a council message.

The error indicates either a bootstrap-order bug (the resource was looked up before it was registered) or a stale reference.",
        hints: &["Confirm the resource was added to *this* community, not a sibling", "Use `community.has_resource(key)` to test before access", "Inspect bootstrap order — registration must precede lookup", "Resource keys are case-sensitive strings"],
        example_bad: Some("let db = comm.resource(\"db.primary\");  // never registered"),
        example_good: Some("comm.add_resource(\"db.primary\", primary_db);
let db = comm.resource(\"db.primary\");"),
        see_also: &["HHS_E_COMMUNITY_DUPLICATE_RESOURCE", "HHS_E_GOVERNANCE_RESOURCE_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const CONSTITUTION_INVALID_VERSION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(42),
        long_code: "HHS_E_CONSTITUTION_INVALID_VERSION",
        short_code: "E0042",
        title: "Invalid constitution version string",
        short_description: "A constitution version did not parse as a valid semantic version.",
        long_description: "Constitutions in HudHudScript are versioned with a semver-like scheme (`MAJOR.MINOR.PATCH`). The version is used for ordering, dependency resolution, and rollback. When a version string is supplied that cannot be parsed — empty, missing fields, non-numeric components, or contains illegal separators — this error is raised.

Validation happens at constitution creation, on `set_version`, and when adding a dependency that names a version. The original input is included in the error message so you can pinpoint the bad source.

Fix the offending literal. If you are constructing the version programmatically, use the `Version::new(major, minor, patch)` builder rather than string formatting.",
        hints: &["Use `MAJOR.MINOR.PATCH` format — three numeric components", "Prefer `Version::new(1, 0, 0)` over string concatenation", "Avoid leading `v` prefixes — `1.0.0`, not `v1.0.0`", "Pre-release suffixes (e.g. `-rc1`) follow semver rules"],
        example_bad: Some("Constitution::new(\"core\", version: \"v1.0\");"),
        example_good: Some("Constitution::new(\"core\", version: \"1.0.0\");"),
        see_also: &["HHS_E_CONSTITUTION_NO_PREVIOUS_VERSION", "HHS_E_GOVERNANCE_FORMAT_VALIDATION"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const CONSTITUTION_LAW_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(43),
        long_code: "HHS_E_CONSTITUTION_LAW_NOT_FOUND",
        short_code: "E0043",
        title: "Law not present in constitution",
        short_description: "Tried to read, amend, or repeal a law by name in a constitution that does not contain it.",
        long_description: "A constitution is a named, ordered collection of laws. Each law has a unique identifier within its constitution. Operations like `constitution.law(id)`, `constitution.amend(id, new_text)`, and `constitution.repeal(id)` all require the law to exist; this error is raised when the lookup fails.

The most frequent causes are typos in the law ID, looking up a law in the wrong constitution version (laws can be added or removed across versions), or attempting to repeal a law that has already been repealed.

Use `constitution.has_law(id)` for existence checks and `constitution.laws()` to enumerate the current law set.",
        hints: &["List `constitution.laws()` to verify the ID exists", "Confirm you are inspecting the correct version", "Use `constitution.has_law(id)` to guard amend/repeal calls", "Law IDs are case-sensitive"],
        example_bad: Some("constitution.repeal(\"no-spam\");
constitution.repeal(\"no-spam\");  // already repealed"),
        example_good: Some("if constitution.has_law(\"no-spam\") {
  constitution.repeal(\"no-spam\");
}"),
        see_also: &["HHS_E_CONSTITUTION_NOT_FOUND", "HHS_E_GOVERNANCE_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const CONSTITUTION_NO_PREVIOUS_VERSION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(44),
        long_code: "HHS_E_CONSTITUTION_NO_PREVIOUS_VERSION",
        short_code: "E0044",
        title: "Constitution has no previous version to roll back to",
        short_description: "Requested the predecessor of a constitution that is at its initial version.",
        long_description: "Constitutions retain a chain of historical versions for rollback and diffing. `constitution.previous_version()` and `constitution.rollback()` walk this chain backwards. When the constitution is already at its first published version, there is no predecessor and this error is raised.

This is an expected outcome when iterating over history; treat it as a terminator rather than a failure if you are walking the chain. For one-off rollback calls, check `constitution.has_previous_version()` first.

Note that history is per-constitution. Two unrelated constitutions don't share lineage even if they have similar names.",
        hints: &["Use `constitution.has_previous_version()` before `rollback()`", "Treat this error as a stop condition when walking history", "The first published version has no parent — that is by design", "Check `constitution.version_history()` to inspect the full chain"],
        example_bad: Some("while true { constitution = constitution.previous_version(); }"),
        example_good: Some("while constitution.has_previous_version() {
  constitution = constitution.previous_version();
}"),
        see_also: &["HHS_E_CONSTITUTION_INVALID_VERSION", "HHS_E_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const CONSTITUTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(45),
        long_code: "HHS_E_CONSTITUTION_NOT_FOUND",
        short_code: "E0045",
        title: "Constitution not registered",
        short_description: "Looked up a constitution by name in a registry that has no such entry.",
        long_description: "Constitutions live in a named registry (per-runtime or per-governance scope). Operations that resolve a constitution by name — binding it to a council, citing it from a law, or loading it for inspection — raise this error when the name is unknown.

The usual causes are: forgetting to register the constitution before referencing it, name typos, or referencing a constitution defined in a different governance scope.

This error is the local form; `GovernanceConstitutionNotFound` is the same condition reported from the global governance facade and `CouncilConstitutionNotFound` is the council-binding-specific form.",
        hints: &["Register with `governance.add_constitution(c)` before referencing", "Confirm the registry scope — global vs. council-local", "Check spelling and casing — names are case-sensitive", "Use `governance.constitutions()` to list registered names"],
        example_bad: Some("council.bind_constitution(\"safety-rules\");  // not registered"),
        example_good: Some("governance.add_constitution(safety_rules);
council.bind_constitution(\"safety-rules\");"),
        see_also: &["HHS_E_GOVERNANCE_CONSTITUTION_NOT_FOUND", "HHS_E_COUNCIL_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_AGENT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(52),
        long_code: "HHS_E_COUNCIL_AGENT_NOT_FOUND",
        short_code: "E0052",
        title: "Agent not a member of this council",
        short_description: "Council operation referenced an agent that is not in the council's membership list.",
        long_description: "A council has a fixed roster of agents, each holding a specific role (Chair, Member, Observer). Operations like `council.assign_role(agent, role)`, `council.remove_member(agent)`, `council.cast_vote(agent, ballot)`, or querying an agent's role all require the agent to be a current member. When the agent ID is not in the roster, this error is raised.

Unlike communities (which carry loose membership), council membership is structural — votes, quorums, and chair succession all depend on it. A missing-agent error here usually indicates a bug in how proposals are routed or how members are added.

Verify membership with `council.is_member(agent)` before performing any per-agent operation.",
        hints: &["Use `council.is_member(agent_id)` before per-member operations", "List `council.members()` while debugging", "Confirm the agent was added with `council.add_member(...)` first", "Check whether another task removed the agent concurrently"],
        example_bad: Some("council.cast_vote(unknown_agent, Vote::Yes);"),
        example_good: Some("if council.is_member(agent) {
  council.cast_vote(agent, Vote::Yes);
}"),
        see_also: &["HHS_E_COUNCIL_DUPLICATE_AGENT", "HHS_E_COUNCIL_NO_MEMBERS"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_CONSTITUTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(53),
        long_code: "HHS_E_COUNCIL_CONSTITUTION_NOT_FOUND",
        short_code: "E0053",
        title: "Council's bound constitution is missing",
        short_description: "The constitution a council is bound to could not be resolved at lookup time.",
        long_description: "A council can be bound to a constitution that defines the rules its decisions must respect. When the council needs to consult that constitution — to validate a proposal, enforce a quorum rule, or check a law — it resolves the binding by name through the governance registry. This error fires when that resolution fails.

Common causes: the bound constitution was removed from the registry after binding, the binding refers to a name that was never registered, or the council was deserialized into a context where the constitution does not exist.

Rebind the council to a present constitution with `council.bind_constitution(name)` or re-register the constitution before the council is invoked.",
        hints: &["Verify the constitution is registered in the active governance scope", "Avoid removing constitutions while councils still reference them", "On deserialization, check that all required constitutions are loaded", "Use `governance.constitutions()` to list what is currently registered"],
        example_bad: Some("let council = Council::new(\"safety\", members).bind_constitution(\"missing\");"),
        example_good: Some("governance.add_constitution(safety_rules);
let council = Council::new(\"safety\", members).bind_constitution(\"safety-rules\");"),
        see_also: &["HHS_E_CONSTITUTION_NOT_FOUND", "HHS_E_GOVERNANCE_CONSTITUTION_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_DUPLICATE_AGENT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(54),
        long_code: "HHS_E_COUNCIL_DUPLICATE_AGENT",
        short_code: "E0054",
        title: "Agent already in council",
        short_description: "Tried to add an agent to a council that already includes it.",
        long_description: "Council membership is a set, not a multiset — each agent ID appears at most once. `council.add_member(agent, role)` raises this error if the agent is already in the roster, even if you intend to assign a different role.

To change a member's role, use `council.assign_role(agent, new_role)` instead of removing and re-adding. To replace a member entirely, remove them first.

The constraint exists because vote tallies, quorum, and chair succession rely on each agent counting exactly once. Allowing duplicates would silently inflate the apparent membership.",
        hints: &["Use `council.assign_role(agent, role)` to change roles in place", "Check `council.is_member(agent)` before adding", "When seeding a council, deduplicate the agent list", "To replace a member, call `remove_member` first"],
        example_bad: Some("council.add_member(alice, Role::Member);
council.add_member(alice, Role::Chair);"),
        example_good: Some("council.add_member(alice, Role::Member);
council.assign_role(alice, Role::Chair);"),
        see_also: &["HHS_E_COUNCIL_AGENT_NOT_FOUND", "HHS_E_COUNCIL_INVALID_ROLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };

pub const COUNCIL_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(55),
        long_code: "HHS_E_COUNCIL_EXECUTION_FAILED",
        short_code: "E0055",
        title: "Council decision execution failed",
        short_description: "The action chosen by a council vote raised an error during execution.",
        long_description: "After a council reaches a decision, its execute step runs the action attached to the winning option. If that action raises — whether from agent failure, network error, downstream guard, or panic in user code — the council reports `CouncilExecutionFailed`, wrapping the underlying cause in the message.

This is distinct from a vote failure. The vote succeeded; the *execution* of its outcome did not. The council's state advances to a failed state and the result is propagated to the caller of `council.run(proposal)`.

Inspect the inner cause to drive recovery: retryable errors may warrant a re-vote, while invariant violations should be surfaced to the operator.",
        hints: &["Inspect the wrapped cause — the message includes the inner error", "Wrap action bodies in defensive `try/catch` to attach context", "Decide whether re-running the decision or escalating is appropriate", "Log the proposal ID alongside the failure for traceability"],
        example_bad: Some("council.run(deploy_proposal);  // action panics on missing creds"),
        example_good: Some("try {
  council.run(deploy_proposal);
} catch (e: CouncilExecutionFailed) {
  log.error(\"deploy failed\", cause: e.cause);
}"),
        see_also: &["HHS_E_COUNCIL_TIMEOUT", "HHS_E_SWARM_AGENT_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Governance,
    };
