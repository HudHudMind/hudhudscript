use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const OPEN_API_PARSE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(157),
        long_code: "HHS_E_OPEN_API_PARSE_ERROR",
        short_code: "E0157",
        title: "OpenAPI document failed to parse",
        short_description: "The OpenAPI / Swagger spec could not be parsed — invalid JSON/YAML, wrong version, or schema violations.",
        long_description: "The OpenAPI tool loads a spec and converts each operation into a callable. This error fires before any operation is exposed: the document itself is malformed, the OpenAPI version is unsupported, required top-level fields are missing, or `$ref` pointers cannot be resolved.

Fix it by validating the spec with an external linter such as `redocly lint` or `openapi-spec-validator`. Resolve `$ref`s in a separate pass and check that referenced files are reachable from the loader's working directory.

For specs you do not control, pin a known-good version and report the upstream issue. Truncated downloads are also a common cause — verify the byte length matches the source.",
        hints: &["Lint the spec with redocly or openapi-spec-validator", "Check that all $ref targets resolve", "Verify the OpenAPI version is supported (3.0+ typically)", "Re-download the spec — partial downloads parse oddly"],
        example_bad: Some("openapi::load(\"./broken.yaml\");"),
        example_good: Some("openapi::load(\"./petstore-v3.yaml\");"),
        see_also: &["OpenApiRegistryError", "HttpToolParseError", "ToolValidation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const OPEN_API_REGISTRY_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(158),
        long_code: "HHS_E_OPEN_API_REGISTRY_ERROR",
        short_code: "E0158",
        title: "OpenAPI tool registration failed",
        short_description: "The parsed OpenAPI spec could not be registered as callable tools — usually a name clash or unsupported operation shape.",
        long_description: "After parsing succeeds, each OpenAPI operation is registered with the tool registry under a derived name (commonly `operationId`). This error fires when registration fails: duplicate names with already-loaded tools, missing `operationId`, unsupported parameter styles, or request body schemas the binder cannot represent.

Fix it by giving every operation a unique, identifier-friendly `operationId` in the spec, and by avoiding clashes with built-in tool names. If the spec uses exotic features (deeply nested oneOf, callbacks, links) consider preprocessing it into a simpler shape.

Namespacing the import (e.g. `openapi::load_namespaced(\"petstore\", spec)`) is the cleanest way to avoid name collisions across multiple imported APIs.",
        hints: &["Give every operation a unique operationId", "Namespace imports to avoid clashes between specs", "Avoid clashing with built-in tool names", "Preprocess specs that use callbacks/links/deep oneOf"],
        example_bad: Some("openapi::load(\"a.yaml\");
openapi::load(\"b.yaml\"); // both define `getUser`"),
        example_good: Some("openapi::load_namespaced(\"a\", \"a.yaml\");
openapi::load_namespaced(\"b\", \"b.yaml\");"),
        see_also: &["OpenApiParseError", "ToolInvalidArguments", "ToolValidation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };
