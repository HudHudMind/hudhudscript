use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 9] = [
    ExceptionEntry {
        code: ExceptionCode(310),
        long_code: "HHS_E_VALIDATION_CUSTOM",
        short_code: "E0310",
        title: "Custom Validation Rule Failed",
        short_description: "A user-defined validation rule rejected a value with a custom error message.",
        long_description: "Beyond the built-in validators, `hudhudscript-validation` allows scripts to attach arbitrary predicates with custom messages. When such a predicate returns false, this variant wraps the supplied message and reports it as a normal validation failure.

The message is whatever the rule author wrote, so its quality depends on them. Treat the wrapped text as authoritative for the cause, but verify it actually identifies the field at fault.

Fix the input to satisfy the rule, or revisit the rule itself if its message is unclear.",
        hints: &["The wrapped message comes from the rule author — read it carefully", "Improve custom rules to mention the field name and constraint", "Distinguish custom rule failures from built-in validators when triaging", "Test custom rules with both valid and invalid inputs"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationInvalidFormat", "ValidationOutOfRange", "ValidationPatternMismatch"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(311),
        long_code: "HHS_E_VALIDATION_INVALID_FORMAT",
        short_code: "E0311",
        title: "Value Has Invalid Format",
        short_description: "A value did not match the structural format expected for its declared type, such as date, UUID, or email.",
        long_description: "Format validators check that a string conforms to a high-level shape — e-mail, ISO date, UUID, URL, and similar — without going through full pattern matching. When the input fails the format check, this variant fires with the format name attached.

The check is binary: either the input is well-formed for the declared format or it is not. There is no partial credit.

Normalize the input to match the format and re-submit. If you control the schema, reconsider whether the format is the right validator or whether you actually want a more permissive pattern.",
        hints: &["Confirm which format the field declares (date, uuid, email, etc.)", "Normalize whitespace and case before validating where appropriate", "Prefer format validators over hand-rolled regex when one fits", "Reject inputs at the boundary, not deep inside business logic"],
        example_bad: Some("{ \"email\": \"not-an-email\" }"),
        example_good: Some("{ \"email\": \"user@example.com\" }"),
        see_also: &["ValidationPatternMismatch", "ValidationTypeMismatch", "ValidationCustom"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(312),
        long_code: "HHS_E_VALIDATION_INVALID_LENGTH",
        short_code: "E0312",
        title: "Value Has Invalid Length",
        short_description: "A string or array value's length did not satisfy the schema's minimum or maximum length constraint.",
        long_description: "Length validators enforce a minimum and/or maximum number of elements (or characters, for strings). The diagnostic includes both the expected bound and the actual length so you can see immediately why the input was rejected.

Note that for strings, length is typically measured in Unicode code points, not bytes. A multibyte character counts as one. Mismatches between byte length and code-point length are a common source of confusion.

Fix the input to fall within the allowed range, or relax the bound if it does not match real-world usage.",
        hints: &["Note whether length is measured in characters or bytes", "Multibyte characters count as one code point", "Display real-world length distributions when picking bounds", "Pad or truncate at the boundary, not deep inside business logic"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationOutOfRange", "ValidationPatternMismatch", "ValidationCustom"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(313),
        long_code: "HHS_E_VALIDATION_MISSING_REQUIRED",
        short_code: "E0313",
        title: "Required Schema Field Missing",
        short_description: "A field marked as required by the tools-schema validator was absent from the validated payload.",
        long_description: "This variant comes from the tools-schema flavor of the validator and fires when a payload omits a field that the schema declared as required. The diagnostic includes the field name so you can locate it in the schema and the input.

This is distinct from `ValidationRequiredFieldMissing` from the standalone validation crate — they have separate code points because they come from different validator implementations. Functionally they cover the same case.

Add the field with an appropriate value, or remove the requirement from the schema if it is genuinely optional.",
        hints: &["Locate the field in the schema and add it to the payload", "Distinguish from `ValidationRequiredFieldMissing` (different validator)", "Remove the requirement only if the field is genuinely optional", "Use schema-driven test fixtures to catch this in CI"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationRequiredFieldMissing", "ValidationTypeMismatch", "ValidationUnknownType"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(314),
        long_code: "HHS_E_VALIDATION_OUT_OF_RANGE",
        short_code: "E0314",
        title: "Numeric Value Out Of Range",
        short_description: "A numeric value fell outside the inclusive minimum and maximum range declared by the schema.",
        long_description: "Range validators reject numbers that lie below the configured minimum or above the configured maximum. The diagnostic includes both bounds and the offending value so the gap is obvious.

Floating-point ranges should be treated with the usual care — equality on floats is fragile, and numbers very close to a bound can fail in surprising ways. If the bound is critical, prefer integer or fixed-point representations.

Clamp the value at the boundary, reject the input upstream, or relax the bound if it does not match real-world data.",
        hints: &["Clamp values at the boundary rather than mid-pipeline", "Prefer integer or fixed-point types when bounds are critical", "Beware of float equality near the bound", "Display realistic value distributions when picking bounds"],
        example_bad: Some("{ \"age\": -1 }"),
        example_good: Some("{ \"age\": 0 }"),
        see_also: &["ValidationInvalidLength", "ValidationCustom", "ValidationTypeMismatch"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(315),
        long_code: "HHS_E_VALIDATION_PATTERN_MISMATCH",
        short_code: "E0315",
        title: "Value Does Not Match Pattern",
        short_description: "A string value failed to match the regular expression pattern declared in the schema.",
        long_description: "Pattern validators apply a regular expression to a string field. When the input does not match, this error is raised with the failing pattern attached so you can see exactly what was expected.

Patterns are powerful but easy to get wrong. Prefer dedicated format validators (`InvalidFormat`) when one exists for the shape you want — they are more readable and harder to misuse than a hand-rolled regex.

Fix the input to match, or revise the pattern if it is more strict than the real-world data.",
        hints: &["Prefer format validators when one exists for the shape you want", "Anchor patterns with `^` and `$` to avoid partial-match surprises", "Test patterns against real production samples, not just synthetic ones", "Document the pattern's intent next to the schema declaration"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationInvalidFormat", "ValidationCustom", "ValidationInvalidLength"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(316),
        long_code: "HHS_E_VALIDATION_REQUIRED_FIELD_MISSING",
        short_code: "E0316",
        title: "Required Field Missing From Input",
        short_description: "The standalone validation crate detected that a payload is missing a field declared as required.",
        long_description: "This variant is produced by `hudhudscript-validation` (as opposed to the tools-schema variant). It fires whenever a required field is absent from the input being validated. The field name is included in the diagnostic.

The duplicate naming with `ValidationMissingRequired` is intentional: the two validators come from different crates and produce distinct error codes so callers can route them separately. Their semantic meanings overlap.

Add the missing field, or revise the schema if the field is genuinely optional.",
        hints: &["Add the missing field to the input payload", "Distinguish from `ValidationMissingRequired` (tools-schema validator)", "Use schema-driven test fixtures to catch this before runtime", "Make required fields explicit in API documentation"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationMissingRequired", "ValidationTypeMismatch", "ValidationCustom"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(317),
        long_code: "HHS_E_VALIDATION_TYPE_MISMATCH",
        short_code: "E0317",
        title: "Schema Type Mismatch",
        short_description: "A field's value did not match the type declared in the tools-schema, e.g. a string where a number was expected.",
        long_description: "The tools-schema validator inspects each field and confirms its runtime type matches the declared schema type. When they differ, this variant fires with both the expected and actual types in the message.

Type mismatches commonly come from JSON values that look right but are not — quoted numbers, boolean-like strings, null in non-nullable fields. The validator does not coerce; it reports the mismatch and stops.

Fix the producer to emit the correct type. Coercion at the boundary, if appropriate, should be explicit and applied before validation rather than inside the validator.",
        hints: &["Watch for quoted numbers and boolean-like strings in JSON", "Apply explicit coercion before validation, not inside it", "Make schema types match the producer's output exactly", "Use typed serializers on the producer side to prevent drift"],
        example_bad: Some("{ \"count\": \"3\" }"),
        example_good: Some("{ \"count\": 3 }"),
        see_also: &["ValidationUnknownType", "ValidationMissingRequired", "ValidationInvalidFormat"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    },

    ExceptionEntry {
        code: ExceptionCode(318),
        long_code: "HHS_E_VALIDATION_UNKNOWN_TYPE",
        short_code: "E0318",
        title: "Unknown Schema Type",
        short_description: "A schema referenced a type name that the validator does not recognize and cannot evaluate.",
        long_description: "Schemas declare their fields by type name. When the validator encounters a type name that is not in its registry — usually because of a typo, a removed extension, or a schema written against a newer validator — this error is raised with the offending name attached.

The schema cannot be used until the unknown type is resolved. No fields are validated against an unrecognized type; the validator refuses to guess.

Fix the type name, register the missing type, or upgrade the validator to a version that knows it.",
        hints: &["Confirm the type name against the validator's supported list", "Check for typos or stale references in the schema", "Upgrade the validator if the schema targets a newer version", "Register custom types explicitly before use"],
        example_bad: None,
        example_good: None,
        see_also: &["ValidationTypeMismatch", "ValidationMissingRequired", "ValidationInvalidFormat"],
        since_version: "0.4.0",
        category: ExceptionCategory::Validation,
    }
];
