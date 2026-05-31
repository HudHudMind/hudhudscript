use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};
pub const HTTP_TOOL_INVALID_URL: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(106),
        long_code: "HHS_E_HTTP_TOOL_INVALID_URL",
        short_code: "E0106",
        title: "HTTP request URL is malformed",
        short_description: "The URL passed to the HTTP tool failed to parse as a valid absolute URL.",
        long_description: "The HTTP tool requires an absolute URL with scheme and host. This error means the parser rejected the input — common causes include missing scheme (`example.com/api` instead of `https://example.com/api`), unencoded spaces or non-ASCII characters in the path, an empty string, or a relative URL produced by string concatenation gone wrong.

Fix it by always supplying a fully qualified URL and by URL-encoding any user-supplied path or query components with `url::encode`. When building URLs from a base + path, use a join helper rather than naive string concatenation so that double slashes and missing slashes are normalized.

If the input legitimately is user-controlled, validate it before passing it to the tool so you can return a friendlier error.",
        hints: &["Always include the scheme: https:// or http://", "URL-encode user-supplied path and query components", "Use a join helper instead of string concatenation", "Validate user input before passing it to http::request"],
        example_bad: Some("http::get(\"example.com/api?q=hello world\");"),
        example_good: Some("http::get(\"https://example.com/api?q=\" + url::encode(\"hello world\"));"),
        see_also: &["HttpToolRequestFailed", "HttpToolParseError", "HttpToolTimeout"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const HTTP_TOOL_PARSE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(107),
        long_code: "HHS_E_HTTP_TOOL_PARSE_ERROR",
        short_code: "E0107",
        title: "HTTP response body parse failed",
        short_description: "The response was received but could not be decoded as the requested type (JSON, text, bytes, etc.).",
        long_description: "After a successful transport, the HTTP tool tries to decode the body according to the caller's expectation — usually JSON. This error fires when the body is not valid JSON, when the Content-Type promised JSON but the server returned HTML (a login page or an error page), or when text decoding fails because of an unexpected charset.

Fix it by inspecting the raw response body and Content-Type header before trusting the parse. For JSON APIs, check whether the failure response uses a different schema than the success path and branch on status code first.

A common pattern is a 200 OK page that is actually an HTML captive-portal interstitial — log the first 200 bytes of the body when this fires to spot it quickly.",
        hints: &["Branch on status code before parsing the body", "Log the first bytes of the body when parsing fails", "Check Content-Type — JSON parsers reject HTML error pages", "Use http::get_text() first if the schema is uncertain"],
        example_bad: Some("let data = http::get_json(\"https://api.example.com/x\");"),
        example_good: Some("let res = http::get(\"https://api.example.com/x\");
if res.status == 200 { let data = json::parse(res.body); }"),
        see_also: &["HttpToolRequestFailed", "HttpToolInvalidUrl", "OpenApiParseError"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const HTTP_TOOL_REQUEST_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(108),
        long_code: "HHS_E_HTTP_TOOL_REQUEST_FAILED",
        short_code: "E0108",
        title: "HTTP request transport failed",
        short_description: "The HTTP client failed to complete the request — DNS, TLS, connection refused, redirect loop, or non-2xx status surface here.",
        long_description: "This is the catch-all for transport-level HTTP failures. The wrapped message identifies the specific cause: DNS resolution failure, connection refused, TLS handshake error, too many redirects, or a non-success HTTP status when the call site requires 2xx.

Fix it by reading the wrapped error and reproducing with `curl -v` against the same URL. Verify DNS, firewalls, proxy configuration, and certificate trust independently. For 4xx/5xx responses, check the response body for the API's own error message.

For flaky upstreams, wrap the request in a retry-with-backoff loop, but only retry idempotent methods (GET, PUT, DELETE) — never blindly retry POST.",
        hints: &["Reproduce with `curl -v` against the same URL", "Check DNS, proxy env vars, and certificate trust", "Retry only idempotent methods on transient failures", "Inspect the response body for API-level error details"],
        example_bad: Some("http::post(\"https://api.example.com/charge\", body); // retried blindly"),
        example_good: Some("let res = retry(3, || http::get(\"https://api.example.com/health\"));"),
        see_also: &["HttpToolTimeout", "HttpToolInvalidUrl", "HttpToolParseError"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const HTTP_TOOL_TIMEOUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(109),
        long_code: "HHS_E_HTTP_TOOL_TIMEOUT",
        short_code: "E0109",
        title: "HTTP request exceeded timeout",
        short_description: "The HTTP client gave up waiting for the server to respond within the configured timeout window.",
        long_description: "The HTTP tool enforces both connect and total-request timeouts. This error means one of those budgets was exhausted — the server either never accepted the connection, accepted it but never sent headers, or stalled mid-body.

Fix it by first establishing whether the server is genuinely slow (test with `curl --max-time`) or whether the timeout is unrealistically tight. Raise the timeout for legitimately long operations, or move the call into a background job if it routinely takes minutes.

For idempotent requests, a retry-with-backoff after timeout is safe; for POSTs that mutate state, prefer an idempotency key so retries do not double-charge.",
        hints: &["Confirm with `curl --max-time` whether the server is slow", "Raise the timeout for genuinely long operations", "Use idempotency keys before retrying mutating POSTs", "Move multi-minute calls into a background job"],
        example_bad: Some("http::get(\"https://slow.example.com/big\", { timeout_secs: 1 });"),
        example_good: Some("http::get(\"https://slow.example.com/big\", { timeout_secs: 60 });"),
        see_also: &["HttpToolRequestFailed", "HttpToolInvalidUrl", "HttpToolParseError"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const OPEN_API_PARSE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(157),
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
        category: ErrorCategory::Tool,
    };

pub const OPEN_API_REGISTRY_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(158),
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
        category: ErrorCategory::Tool,
    };

pub const TOOL_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(295),
        long_code: "HHS_E_TOOL_EXECUTION_FAILED",
        short_code: "E0295",
        title: "Tool dispatch ran but failed",
        short_description: "A registered tool was located and invoked, but its execution returned a runtime failure.",
        long_description: "This is the generic outer wrapper for any tool that fails during its own work. The wrapped message comes from the tool implementation and is the authoritative description. Use this error as a router: read the wrapped detail, then jump to the more specific error code if one exists (HTTP, DB, Git, etc.).

Fix it by inspecting the wrapped cause and following its remediation. If the tool is custom, add structured error context inside the tool implementation so callers do not have to grep free-text.

For unreliable tools, wrap calls in `try`/`catch` and decide per-tool whether to retry, fall back, or surface to the operator.",
        hints: &["Read the wrapped cause — it identifies the real subsystem", "Add structured context in custom tool implementations", "Decide per-tool whether to retry or surface failures", "Jump to the specific error (HTTP/DB/Git) when one exists"],
        example_bad: Some("tool::call(\"flaky\", args); // no error handling"),
        example_good: Some("try { tool::call(\"flaky\", args); } catch (e) { log::warn(e); }"),
        see_also: &["ToolInvalidArguments", "ToolValidation", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const TOOL_INVALID_ARGUMENTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(296),
        long_code: "HHS_E_TOOL_INVALID_ARGUMENTS",
        short_code: "E0296",
        title: "Tool received invalid arguments",
        short_description: "Arguments to a tool call failed the tool's input schema check before any work was done.",
        long_description: "Each tool declares an input schema. This error fires when the supplied arguments fail that schema — missing required fields, wrong types, values outside enums, or extra unknown fields when the schema is closed.

Fix it by inspecting the tool's declared schema (`tool::describe(name)`) and aligning the call site. When generating tool calls from LLM output, validate the JSON locally before dispatch so that the model can be re-prompted on its own malformed output rather than failing the script.

This is the right error to raise from custom tools that want to reject bad input cleanly — keep the message specific to the offending field.",
        hints: &["Inspect the schema with tool::describe(name)", "Validate LLM-generated tool calls before dispatch", "Name the offending field in custom tool error messages", "Watch for closed schemas that reject extra fields"],
        example_bad: Some("tool::call(\"send_email\", { to: 42 }); // wrong type"),
        example_good: Some("tool::call(\"send_email\", { to: \"a@b.c\", subject: \"hi\" });"),
        see_also: &["ToolValidation", "ToolExecutionFailed", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const TOOL_SECURITY_VIOLATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(297),
        long_code: "HHS_E_TOOL_SECURITY_VIOLATION",
        short_code: "E0297",
        title: "Tool call blocked by security policy",
        short_description: "The runtime's security policy denied the tool call — for example a sandboxed script tried to access the network or filesystem.",
        long_description: "HudHudScript can run with a configurable security policy that restricts which tools a script can invoke and which resources those tools may touch. This error fires when a call is blocked by that policy: a sandboxed script reaches out to the network, a read-only context tries to mutate files, or a deny-listed tool is invoked.

Fix it by either rewriting the script to stay within the allowed surface, or by adjusting the policy in the host configuration if the access is legitimately required. Never silently widen a policy in production without auditing what it now allows.

For multi-tenant hosts, log every security violation with the script id and the requested tool — these logs are the audit trail for sandbox escapes.",
        hints: &["Read the policy to see which tools/resources are allowed", "Refactor scripts to stay inside the sandbox where possible", "Audit policy widenings — they expand the attack surface", "Log every violation with script id + requested tool"],
        example_bad: Some("// script in network-deny sandbox
http::get(\"https://example.com\");"),
        example_good: Some("// move external calls into a privileged orchestrator script"),
        see_also: &["ToolExecutionFailed", "ToolValidation", "ApprovalInvalidTransition"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const TOOL_VALIDATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(298),
        long_code: "HHS_E_TOOL_VALIDATION",
        short_code: "E0298",
        title: "Tool input failed semantic validation",
        short_description: "Arguments parsed against the schema but failed a stronger semantic check, such as cross-field constraints or referential integrity.",
        long_description: "Where `ToolInvalidArguments` covers shape/type problems, `ToolValidation` covers richer rules: a date range whose end precedes its start, a foreign key that does not resolve, mutually exclusive options that were both supplied, or a value that fails a regex constraint.

Fix it by reading the validation message — it should name the failing rule. For custom tools, prefer raising this error (rather than `ToolInvalidArguments`) for cross-field and lookup-based checks so callers can distinguish syntactic from semantic issues.

When building tool inputs from user data, run the same validation client-side first to give faster feedback before dispatch.",
        hints: &["Distinguish: ToolInvalidArguments = shape, ToolValidation = rules", "Name the failing rule in custom validation messages", "Mirror server-side validation client-side for faster feedback", "Reject mutually exclusive options early"],
        example_bad: Some("tool::call(\"book\", { start: \"2026-05-10\", end: \"2026-05-01\" });"),
        example_good: Some("tool::call(\"book\", { start: \"2026-05-01\", end: \"2026-05-10\" });"),
        see_also: &["ToolInvalidArguments", "ToolExecutionFailed", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const VCS_BRANCH_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(319),
        long_code: "HHS_E_VCS_BRANCH_ALREADY_EXISTS",
        short_code: "E0319",
        title: "Branch name is already in use",
        short_description: "A branch creation attempt failed because a branch with the same name already exists in the repository.",
        long_description: "Git refuses to create a branch over an existing one without an explicit force flag. This error wraps that refusal so scripts can react cleanly — for example by switching to the existing branch instead of failing, or by generating a unique suffix.

Fix it by deciding on a policy: skip if exists, force-overwrite (`-f`), or generate a fresh name (e.g. append a timestamp). Whatever the policy, make it explicit in the script so concurrent runs do not race.

This error often appears in automation that creates a feature branch per ticket; idempotent helpers should check `git rev-parse --verify <branch>` first.",
        hints: &["Decide explicit policy: skip / force / unique-name", "Check `git rev-parse --verify <branch>` for idempotency", "Append a timestamp or run id to keep names unique", "Avoid concurrent script runs racing on the same branch"],
        example_bad: Some("vcs::create_branch(\"feature/x\"); // already exists"),
        example_good: Some("if !vcs::branch_exists(\"feature/x\") { vcs::create_branch(\"feature/x\"); }"),
        see_also: &["VcsBranchNotFound", "VcsInvalidOperation", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const VCS_BRANCH_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(320),
        long_code: "HHS_E_VCS_BRANCH_NOT_FOUND",
        short_code: "E0320",
        title: "Branch does not exist in repository",
        short_description: "An operation tried to reference a branch that the repository does not have locally or on the configured remote.",
        long_description: "Git operations like checkout, delete, and merge require the target branch to exist. This error fires when the lookup returns nothing — typical causes are a typo, a branch that lives only on the remote and was never fetched, or a branch that has already been deleted by another script run.

Fix it by running `git fetch --all --prune` before the operation if the branch may live on a remote, and by listing branches (`git branch -a`) to verify spelling. For automation, check existence first with `git rev-parse --verify`.

When deleting branches, make the call idempotent: a missing branch is a successful end state, not an error.",
        hints: &["Run `git fetch --all --prune` before remote-only lookups", "Verify spelling with `git branch -a`", "Use `git rev-parse --verify` for existence checks", "Make delete idempotent — missing == success"],
        example_bad: Some("vcs::checkout(\"feature/typoo\");"),
        example_good: Some("vcs::fetch(); vcs::checkout(\"feature/typo\");"),
        see_also: &["VcsBranchAlreadyExists", "VcsInvalidOperation", "GitRepositoryNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const VCS_INVALID_OPERATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(321),
        long_code: "HHS_E_VCS_INVALID_OPERATION",
        short_code: "E0321",
        title: "VCS operation is not allowed in current state",
        short_description: "The repository is in a state that forbids the requested operation, e.g. merging on a detached HEAD or committing with no staged changes.",
        long_description: "Git enforces preconditions on many operations: you cannot commit without staged changes, cannot merge during a rebase, cannot switch branches with conflicting unstaged changes, and so on. This error wraps that family of refusals.

Fix it by inspecting the wrapped reason and the repo state (`git status`) to see what precondition was violated. Resolve the blocking state — finish or abort the in-progress operation, stash or commit dirty changes, attach HEAD to a branch — before retrying.

For scripts, snapshot `git status --porcelain` at the start and refuse to proceed if the tree is unexpectedly dirty.",
        hints: &["Read the wrapped reason — it names the violated precondition", "Inspect `git status` to see the repo's actual state", "Finish or abort in-progress merges/rebases before continuing", "Snapshot `git status --porcelain` at script entry"],
        example_bad: Some("vcs::commit(\"empty\"); // nothing staged"),
        example_good: Some("vcs::add([\"file.txt\"]);
vcs::commit(\"add file\");"),
        see_also: &["VcsMergeConflict", "VcsBranchNotFound", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const VCS_MERGE_CONFLICT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(322),
        long_code: "HHS_E_VCS_MERGE_CONFLICT",
        short_code: "E0322",
        title: "Merge produced conflicts requiring resolution",
        short_description: "A merge or rebase left conflict markers in the working tree; the operation cannot be completed automatically.",
        long_description: "This error fires when git reports unresolved conflicts after a merge, rebase, cherry-pick, or pull. The working tree contains files with `<<<<<<<`, `=======`, `>>>>>>>` markers and the index lists them as unmerged. No subsequent VCS operation will succeed until the conflicts are resolved or the merge is aborted.

Fix it by inspecting the conflicting files (`git status` lists them under \"Unmerged paths\"), editing them to a final state, then `git add`ing each and finishing with `git commit` (or `git rebase --continue`). To bail out entirely, use `git merge --abort` or `git rebase --abort`.

Automation should generally not try to auto-resolve semantic conflicts; instead, escalate to a human via an approval request and pause the script.",
        hints: &["Use `git status` to see Unmerged paths", "Resolve, `git add`, then `git commit` or `--continue`", "Use `git merge --abort` to bail out cleanly", "Escalate semantic conflicts to a human via approval::request()"],
        example_bad: Some("vcs::merge(\"feature\"); vcs::push(); // conflict ignored"),
        example_good: Some("if vcs::merge(\"feature\").has_conflicts {
    approval::request({ title: \"Resolve conflicts\" });
}"),
        see_also: &["VcsInvalidOperation", "GitCommandFailed", "ApprovalInvalidTransition"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const VCS_PARSE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(323),
        long_code: "HHS_E_VCS_PARSE_ERROR",
        short_code: "E0323",
        title: "Failed to parse VCS output",
        short_description: "The VCS layer could not parse output from an underlying command into its expected structured form.",
        long_description: "The high-level VCS layer parses porcelain or format-string output from git into structured records (branches, commits, statuses). This error means parsing failed — usually because of locale, an unrecognized git version, or an unexpected hook printing extra bytes onto stdout.

Fix it by forcing `LC_ALL=C` and `LANG=C` before VCS calls, by silencing or redirecting noisy hooks, and by reproducing the failing command manually to inspect the actual bytes. If the raw output looks well-formed, the parser may have a gap and is worth reporting.

This is the high-level cousin of `GitParseError` and shares the same remediation playbook.",
        hints: &["Force LC_ALL=C and LANG=C before VCS calls", "Silence or redirect hooks that print to stdout", "Reproduce the failing command manually to inspect bytes", "Report parser gaps with raw output and git version"],
        example_bad: Some("// noisy post-checkout hook prints banner to stdout"),
        example_good: Some("env::set(\"LC_ALL\", \"C\");
let log = vcs::log(10);"),
        see_also: &["GitParseError", "VcsInvalidOperation", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };
