use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const ROLE_INVALID_ROLE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(221),
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
        category: ExceptionCategory::Governance,
    };

pub const ROLE_PERMISSION_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(222),
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
        category: ExceptionCategory::Governance,
    };

pub const ROLE_ROLE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(223),
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
        category: ExceptionCategory::Governance,
    };
