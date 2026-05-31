use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const HTTP_TOOL_INVALID_URL: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(106),
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
        category: ExceptionCategory::Tool,
    };

pub const HTTP_TOOL_PARSE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(107),
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
        category: ExceptionCategory::Tool,
    };

pub const HTTP_TOOL_REQUEST_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(108),
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
        category: ExceptionCategory::Tool,
    };

pub const HTTP_TOOL_TIMEOUT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(109),
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
        category: ExceptionCategory::Tool,
    };
