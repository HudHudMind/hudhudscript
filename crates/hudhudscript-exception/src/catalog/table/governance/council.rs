use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const COUNCIL_AGENT_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(52),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_CONSTITUTION_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(53),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_DUPLICATE_AGENT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(54),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_EXECUTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(55),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_INVALID_ROLE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(56),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_NO_MEMBERS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(57),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(58),
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
        category: ExceptionCategory::Governance,
    };

pub const COUNCIL_TIMEOUT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(59),
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
        category: ExceptionCategory::Governance,
    };
