use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const OLLAMA_DESERIALIZE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(155),
        long_code: "HHS_E_OLLAMA_DESERIALIZE",
        short_code: "E0155",
        title: "Failed to deserialize Ollama response",
        short_description: "The Ollama API returned a body that does not match the expected JSON schema for this endpoint.",
        long_description: "The model manager called the local Ollama daemon and could not deserialize its response. Ollama's HTTP API is generally stable but occasionally adds or renames fields between versions, and you may be running a build of HudHudScript that expects a different shape than the daemon you have installed.

Reproduce the request with `curl http://localhost:11434/api/...` and inspect the body. If it has new or missing fields compared to what HudHudScript expects, upgrade either side until they match.

A second common cause is that the response is a streaming NDJSON body but the caller treated it as a single JSON object (or vice versa). The model manager picks the right mode per endpoint, so if you see this on a streaming endpoint, it usually points at a daemon-side error frame in the middle of the stream.",
        hints: &["Reproduce the request with `curl` and inspect the JSON body", "Check `ollama --version` against the version HudHudScript expects", "Streaming endpoints return NDJSON; non-streaming return a single object", "Upgrade Ollama or HudHudScript to align the schemas"],
        example_bad: None,
        example_good: None,
        see_also: &["OllamaHttp", "HfDeserialize", "PackageSerialization"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };

pub const OLLAMA_HTTP: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(156),
        long_code: "HHS_E_OLLAMA_HTTP",
        short_code: "E0156",
        title: "HTTP request to Ollama failed",
        short_description: "The HTTP client could not reach the Ollama daemon, or the daemon returned a non-2xx status.",
        long_description: "The model manager could not complete an HTTP call to Ollama (default `http://localhost:11434`). The wrapped error tells you whether the daemon was unreachable (connection refused — daemon not running), returned a 404 (model not pulled), 500 (daemon-side failure), or hit a timeout (long generation without keep-alive).

For connection refused, start Ollama with `ollama serve` or check that the systemd unit is up. For 404, run `ollama pull <model>` to install the model first. For timeouts, increase the request timeout in the model manager config or generate in smaller chunks.

If Ollama is bound to a non-default address, set the `OLLAMA_HOST` environment variable so the model manager uses the right URL.",
        hints: &["Start the daemon: `ollama serve`", "Pull missing models: `ollama pull <name>`", "Set OLLAMA_HOST if the daemon binds to a non-default address", "Increase timeouts for long generation requests"],
        example_bad: None,
        example_good: None,
        see_also: &["OllamaDeserialize", "HfHttp", "PackageNetwork"],
        since_version: "0.4.5",
        category: ExceptionCategory::Package,
    };
