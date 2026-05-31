use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const CONSTITUTION_INVALID_VERSION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(42),
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
        category: ExceptionCategory::Governance,
    };

pub const CONSTITUTION_LAW_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(43),
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
        category: ExceptionCategory::Governance,
    };

pub const CONSTITUTION_NO_PREVIOUS_VERSION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(44),
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
        category: ExceptionCategory::Governance,
    };

pub const CONSTITUTION_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(45),
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
        category: ExceptionCategory::Governance,
    };
