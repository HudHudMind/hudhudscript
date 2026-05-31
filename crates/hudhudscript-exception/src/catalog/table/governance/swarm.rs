use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const SWARM_AGENT_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(270),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_AGENT_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(271),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_DUPLICATE_AGENT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(272),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_INSUFFICIENT_SUCCESS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(273),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_NO_AGENTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(274),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_STATE_KEY_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(275),
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
        category: ExceptionCategory::Governance,
    };

pub const SWARM_TIMEOUT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(276),
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
        category: ExceptionCategory::Governance,
    };
