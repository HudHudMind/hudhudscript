use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 10] = [
    ExceptionEntry {
        code: ExceptionCode(209),
        long_code: "HHS_E_REGISTRY_CALL_FAILED",
        short_code: "E0209",
        title: "Tool Registry Call Failed",
        short_description: "A call to a registered tool returned an error or could not be dispatched at all.",
        long_description: "The tool registry in `hudhudscript-tools-schema` lets HudHudScript invoke tools registered by name. When a tool's handler runs but raises an error — or when dispatch fails for any reason after the tool was located — this variant wraps the cause.

Unlike `ToolNotFound`, the tool exists; the failure is in the call itself. The wrapped message typically comes from the tool implementation or from the transport layer between the script and the tool process.

Read the wrapped cause, decide whether to retry, and consider whether your script should validate inputs before invoking the tool to prevent obviously bad calls.",
        hints: &["Read the wrapped cause for the tool-side or transport-side reason", "Validate inputs before invoking tools to catch bad calls early", "Distinguish transient transport errors from logical tool errors", "Add retry/backoff only when the cause is clearly transient"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryToolNotFound", "RegistryDiscoveryFailed", "RegistryValidationFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(210),
        long_code: "HHS_E_REGISTRY_DISCOVERY_FAILED",
        short_code: "E0210",
        title: "Tool Discovery Failed",
        short_description: "The registry could not enumerate the tools exposed by a configured server during discovery.",
        long_description: "On startup or on demand, the registry asks each configured tool server to list its tools. When that listing fails — server unreachable, malformed response, version mismatch — this variant carries the cause back. No tools from that server are registered if discovery fails.

Discovery is independent per server, so other servers are unaffected. Scripts that rely on the missing tools will subsequently see `ToolNotFound`.

Check connectivity to the server, confirm version compatibility, and inspect the wrapped message for protocol-level details.",
        hints: &["Verify connectivity to the tool server", "Confirm protocol version compatibility on both ends", "Inspect the wrapped message for malformed-response details", "Other servers are unaffected — discovery failures are per server"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryServerNotFound", "RegistryCallFailed", "RegistryToolNotFound"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(211),
        long_code: "HHS_E_REGISTRY_DUPLICATE_TOOL",
        short_code: "E0211",
        title: "Duplicate Tool Registration",
        short_description: "Two tools tried to register under the same name, which the registry refuses to allow.",
        long_description: "Tool names must be unique within the registry so that script-side calls always have a single, unambiguous target. Attempting to register a name that already exists fails with this error and the second registration is rejected. The originally registered tool stays in place.

Duplicates often appear when the same server is added twice, or when two servers expose tools that happen to share a name. Namespacing or renaming on one side resolves the conflict.

Remove the duplicate registration, namespace one of the conflicting tools, or pick a different alias when adding the second server.",
        hints: &["Check whether a server is being added more than once", "Namespace tools with a server-specific prefix to avoid clashes", "Use an alias when registering a second server with overlapping names", "Audit the registry on startup to surface conflicts early"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryToolNotFound", "RegistryServerNotFound", "RegistryDiscoveryFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(212),
        long_code: "HHS_E_REGISTRY_SERVER_NOT_FOUND",
        short_code: "E0212",
        title: "Tool Server Not Found In Registry",
        short_description: "An operation referenced a tool server name that the registry has no record of.",
        long_description: "Some registry operations are scoped to a server (rediscover, disable, query its tools). When the supplied server name does not match any configured server, this error is returned. The registry is not modified.

This usually means the server name was misspelled, removed, or never configured at all. The fix is to add the server, correct the name, or remove the dependent code.

List the configured servers and compare the failing name against them. Server names are case-sensitive.",
        hints: &["List configured servers and compare against the failing name", "Server names are case-sensitive", "Confirm the server is actually loaded, not just declared", "Use a typed constant for server names to avoid typos"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryToolNotFound", "RegistryDiscoveryFailed", "RegistryDuplicateTool"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(213),
        long_code: "HHS_E_REGISTRY_TOOL_NOT_FOUND",
        short_code: "E0213",
        title: "Tool Not Found In Registry",
        short_description: "A script asked the registry for a tool by name and no matching tool is currently registered.",
        long_description: "This is the most common registry error. It fires when script code or another subsystem requests a tool by name and the registry has nothing under that name. Causes include typos, a server whose discovery failed, or a tool that was unregistered while the script was running.

The error includes the failing name so you can compare it against the live registry. The script must decide whether to fail loudly, fall back to a default, or wait and retry.

Verify the spelling, confirm the providing server completed discovery successfully, and check whether anything in your code path unregistered the tool unexpectedly.",
        hints: &["Confirm the tool name matches what the providing server exports", "Check whether discovery for the providing server succeeded", "Watch for code paths that unregister tools at runtime", "Fall back to a default only when the contract allows it"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryDiscoveryFailed", "RegistryServerNotFound", "RegistryCallFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(214),
        long_code: "HHS_E_REGISTRY_VALIDATION_FAILED",
        short_code: "E0214",
        title: "Tool Schema Validation Failed",
        short_description: "A tool's input or output failed validation against its declared JSON schema during a registry call.",
        long_description: "Every registered tool carries an input and output schema. The registry validates payloads on both directions: requests are checked before dispatching and responses are checked before returning. When either side fails, this error is raised with details from the validator.

Validation failures usually indicate a contract drift between the script and the tool, or a bug in the tool implementation that produces non-conforming output. The call is treated as unsuccessful even if the tool itself believed it succeeded.

Compare the offending payload against the tool's schema, fix the caller or the tool, and re-run. Treat output-side failures with extra care — they usually mean the tool needs a fix.",
        hints: &["Compare the failing payload against the tool's published schema", "Output-side failures usually indicate a tool implementation bug", "Pin the schema version when calling third-party tools", "Log the validator's full report when investigating"],
        example_bad: None,
        example_good: None,
        see_also: &["RegistryCallFailed", "ValidationTypeMismatch", "ValidationMissingRequired"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(217),
        long_code: "HHS_E_RESOURCE_DISCOVERY_FAILED",
        short_code: "E0217",
        title: "Resource Discovery Failed",
        short_description: "The resource layer could not enumerate available resources from a configured provider.",
        long_description: "`hudhudscript-resources` exposes a uniform interface over a variety of resource providers. Discovery is the step where each provider is asked for its catalog of resources. When that step fails — provider unreachable, authentication denied, malformed catalog response — this variant carries the cause.

A discovery failure means none of the resources from that provider are visible to the runtime, and subsequent lookups will return `ResourceNotFound`.

Inspect the wrapped cause, fix the provider configuration or connectivity, and re-run discovery. Other providers are unaffected.",
        hints: &["Inspect the wrapped cause for the provider-side reason", "Verify credentials and connectivity to the provider", "Other providers continue working independently", "Re-run discovery after fixing provider configuration"],
        example_bad: None,
        example_good: None,
        see_also: &["ResourceNotFound", "ResourceReadFailed", "ResourceInvalidUri"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(218),
        long_code: "HHS_E_RESOURCE_INVALID_URI",
        short_code: "E0218",
        title: "Invalid Resource URI",
        short_description: "A resource URI could not be parsed because it does not conform to the expected scheme and structure.",
        long_description: "Resources are addressed by URIs with a fixed grammar: scheme, authority, and path components, possibly followed by a fragment. When a string fails to parse as a valid resource URI — missing scheme, invalid characters, malformed percent-encoding — this error is returned with the offending input attached.

No lookup is performed for an invalid URI. The error fires purely at the parse step.

Fix the URI to follow the documented grammar. Beware of accidentally interpolating user input without escaping; it is the most common cause.",
        hints: &["Confirm the scheme matches one of the supported resource schemes", "Percent-encode any reserved characters in the path", "Avoid interpolating untrusted strings into URIs without escaping", "Validate URIs at construction time, not at use time"],
        example_bad: Some("res://my host/file"),
        example_good: Some("res://my-host/file"),
        see_also: &["ResourceNotFound", "ResourceReadFailed", "ResourceDiscoveryFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(219),
        long_code: "HHS_E_RESOURCE_NOT_FOUND",
        short_code: "E0219",
        title: "Resource Not Found",
        short_description: "The resource layer parsed the URI successfully but no provider has a resource registered at that location.",
        long_description: "After parsing the URI, the resource layer asks the appropriate provider for the named resource. When the provider returns nothing, this variant fires with the URI string for context. Causes include typos, deleted resources, or providers whose discovery is incomplete.

This error is structurally similar to a 404 from an HTTP server. It does not say the URI was wrong — only that nothing currently lives there.

Verify the resource exists at the expected location, confirm provider discovery completed, and consider whether your code should treat absence as an error or as an expected outcome.",
        hints: &["Confirm the resource exists at the expected URI", "Verify provider discovery completed before lookup", "Treat absence as expected when the contract allows it", "Cache negative lookups carefully to avoid stale misses"],
        example_bad: None,
        example_good: None,
        see_also: &["ResourceInvalidUri", "ResourceReadFailed", "ResourceDiscoveryFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    },

    ExceptionEntry {
        code: ExceptionCode(220),
        long_code: "HHS_E_RESOURCE_READ_FAILED",
        short_code: "E0220",
        title: "Resource Read Failed",
        short_description: "The resource was located but the underlying read operation failed before its contents could be returned.",
        long_description: "Once a resource is located, the provider streams or returns its contents. Any failure during that read — I/O error, decryption failure, transient network drop, decoder error — surfaces as this variant with the wrapped cause attached.

Unlike `ResourceNotFound`, the resource does exist; the read itself broke. Retry policy depends on the provider: filesystem reads are usually idempotent, while remote reads may incur side effects you should think about before retrying.

Inspect the wrapped cause and decide on retry, fallback, or surfacing the error to the user.",
        hints: &["Inspect the wrapped cause for the I/O- or decoder-level reason", "Filesystem reads are typically safe to retry; remote reads may not be", "Add timeouts around remote resource reads to bound failure cost", "Distinguish transient from permanent failures before retrying"],
        example_bad: None,
        example_good: None,
        see_also: &["ResourceNotFound", "ResourceInvalidUri", "ResourceDiscoveryFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Resource,
    }
];
