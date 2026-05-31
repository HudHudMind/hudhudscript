use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const HF_DESERIALIZE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(104),
        long_code: "HHS_E_HF_DESERIALIZE",
        short_code: "E0104",
        title: "Failed to deserialize HuggingFace response",
        short_description: "The HuggingFace API returned a response body that does not match the expected JSON schema for this endpoint.",
        long_description: "The model manager called the HuggingFace Hub API and got back JSON it could not deserialize into the expected struct. This usually happens when: the Hub changed its response format on a particular endpoint, you hit a rate-limit page that returned HTML instead of JSON, or you queried a private repo without an access token and got back an error envelope instead of the expected payload.

First verify you can fetch the same URL with `curl` and that the body looks like the JSON the loader expects. If the body is HTML or a `{\"error\": ...}` envelope, the underlying problem is authentication or rate-limiting, not deserialization.

If the HuggingFace API has genuinely shifted (which they do occasionally), upgrade HudHudScript or pin to a known-good revision while waiting for a fix.",
        hints: &["Reproduce the request with `curl` and inspect the body", "Set HF_TOKEN if you are accessing private or gated repos", "Check HuggingFace status if many requests fail at once", "Upgrade HudHudScript if the API schema has changed"],
        example_bad: None,
        example_good: None,
        see_also: &["HfHttp", "OllamaDeserialize", "PackageSerialization"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };

pub const HF_HTTP: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(105),
        long_code: "HHS_E_HF_HTTP",
        short_code: "E0105",
        title: "HTTP request to HuggingFace failed",
        short_description: "The HTTP client returned an error talking to the HuggingFace Hub: DNS, TLS, connection refused, timeout, or non-2xx status.",
        long_description: "The model manager could not complete an HTTP request to huggingface.co. The wrapped error is preserved verbatim so you can see whether it was DNS, TLS, a 401 (auth), a 403 (gated repo without acceptance), a 404 (typo'd repo id), a 429 (rate-limited), or a 5xx outage on the Hub side.

For 401/403, set `HF_TOKEN` to a valid access token and, for gated models, accept the license on the model page first. For 404, double-check the repo id (`org/name`, case-sensitive). For 429, back off and retry with exponential delay. For network-level errors, check your proxy settings and that `huggingface.co` is reachable.

The model manager retries transient failures internally a few times before surfacing this error, so by the time you see it the failure is persistent within that window.",
        hints: &["Set HF_TOKEN for private or gated repositories", "Verify the repo id is exactly `org/name` (case-sensitive)", "Check `https_proxy` if you are behind a corporate firewall", "On 429, back off and retry; the Hub rate-limits unauthenticated calls"],
        example_bad: None,
        example_good: None,
        see_also: &["HfDeserialize", "OllamaHttp", "PackageNetwork"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };
