use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const PERSPECTIVE_FIELD_HIDDEN: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(191),
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
        category: ExceptionCategory::Governance,
    };

pub const PERSPECTIVE_WRITE_ACCESS_DENIED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(192),
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
        category: ExceptionCategory::Governance,
    };
