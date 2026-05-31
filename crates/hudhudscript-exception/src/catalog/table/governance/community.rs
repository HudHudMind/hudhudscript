use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const COMMUNITY_COUNCIL_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(27),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_COUNCIL: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(28),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_MEMBER: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(29),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_DUPLICATE_RESOURCE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(30),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_INVALID_CONFIGURATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(31),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_MEMBER_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(32),
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
        category: ExceptionCategory::Governance,
    };

pub const COMMUNITY_RESOURCE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(33),
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
        category: ExceptionCategory::Governance,
    };
