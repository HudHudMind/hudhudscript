use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const CATALOG_IO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(24),
        long_code: "HHS_E_CATALOG_IO",
        short_code: "E0024",
        title: "Localization Catalog I/O Failure",
        short_description: "The localization subsystem failed to read or write a translation catalog file from disk.",
        long_description: "`hudhudscript-localization` loads message catalogs from JSON or YAML files at startup or on demand. Any underlying file-system error — missing path, permission denied, broken symlink, full disk during write — is wrapped in this variant so callers can distinguish I/O issues from parse errors.

The wrapped `std::io::Error` is preserved verbatim. Inspect it for the exact OS-level reason; the catalog manager itself is not at fault when this error fires.

Verify the path, check permissions, and make sure the deployment bundles the catalog files alongside the binary. In containerized environments, the catalog directory is a frequent victim of incomplete COPY directives.",
        hints: &["Confirm the catalog path exists and is readable by the process user", "Make sure deployments actually copy the catalog directory", "Distinguish read vs write failures from the wrapped io::Error kind", "Check disk space if the failure is on write"],
        example_bad: None,
        example_good: None,
        see_also: &["CatalogJson", "CatalogYaml", "ResourceReadFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Localization,
    };

pub const CATALOG_JSON: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(25),
        long_code: "HHS_E_CATALOG_JSON",
        short_code: "E0025",
        title: "Localization Catalog JSON Parse Error",
        short_description: "A JSON-formatted localization catalog could not be parsed because its contents are syntactically invalid.",
        long_description: "When the localization layer is configured to read catalogs in JSON, each file is parsed with `serde_json`. Any structural problem — trailing commas, unbalanced braces, smart quotes, BOM in the wrong place, encoding mismatch — surfaces as this variant with the underlying `serde_json::Error` attached.

The wrapped error includes the line and column of the failure, which is usually enough to find the offending token. The catalog file is not partially loaded; either it parses entirely or it is rejected.

Validate the file with a standalone JSON linter, fix the structural issue, and re-run. If the file came from a translation tool, check whether the tool emitted JSON5 or JSONC by mistake — strict JSON does not allow comments.",
        hints: &["Validate the file with `jq . catalog.json` for a clear error location", "Strict JSON has no comments and no trailing commas", "Watch for BOM bytes added by Windows editors", "Confirm the file is UTF-8 with no stray surrogates"],
        example_bad: Some("{
  \"hello\": \"Merhaba\",
}"),
        example_good: Some("{
  \"hello\": \"Merhaba\"
}"),
        see_also: &["CatalogYaml", "CatalogIo", "ResourceReadFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Localization,
    };

pub const CATALOG_YAML: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(26),
        long_code: "HHS_E_CATALOG_YAML",
        short_code: "E0026",
        title: "Localization Catalog YAML Parse Error",
        short_description: "A YAML-formatted localization catalog could not be parsed because its contents are syntactically invalid.",
        long_description: "When catalogs are stored in YAML, the localization layer uses a YAML parser whose error is wrapped here. YAML is whitespace-sensitive, so the most common causes are tab characters, inconsistent indentation, unquoted strings beginning with reserved characters, or accidentally mixed flow and block styles.

The wrapped error usually reports a line number and a brief description. As with JSON catalogs, the file is rejected as a whole; no partial entries leak into the runtime catalog.

Fix the indentation or quoting and re-run. When in doubt, normalize the file with a YAML formatter — they refuse to write structurally invalid output, which makes them a quick sanity check.",
        hints: &["Replace tab characters with spaces — YAML forbids tabs for indentation", "Quote any value beginning with `:`, `-`, `?`, `&`, `*`, `!`, `|`, or `>`", "Normalize the file with a YAML formatter to spot mistakes quickly", "Verify the file is UTF-8 without BOM"],
        example_bad: Some("hello:	Merhaba"),
        example_good: Some("hello: Merhaba"),
        see_also: &["CatalogJson", "CatalogIo", "ResourceReadFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Localization,
    };

pub static ENTRIES: &[ErrorEntry] = &[CATALOG_IO, CATALOG_JSON, CATALOG_YAML];
