use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const CONVERSATION_EMPTY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(46),
        long_code: "HHS_E_CONVERSATION_EMPTY",
        short_code: "E0046",
        title: "Conversation Has No Messages",
        short_description: "An operation was attempted on a conversation that contains zero messages.",
        long_description: "Many AI tools require at least one message in a conversation — for example, to generate a reply or to compute embeddings. This error fires when you call such a function on a freshly created or fully cleared conversation.

Initialize the conversation with a system prompt or first user turn before invoking the operation. If the conversation was loaded from disk and unexpectedly empty, check the persistence layer for truncation.

This is almost always a programming error in the calling code rather than a runtime fault.",
        hints: &["Add at least one message before calling `conversation.send()`", "Use a system prompt as the first turn", "Verify deserialized conversations actually loaded messages", "Check for accidental `conversation.clear()` calls"],
        example_bad: Some("let c = Conversation::new();
c.send(\"hi\")?; // empty, no system prompt"),
        example_good: Some("let c = Conversation::new();
c.add_system(\"You are a helpful assistant\");
c.send(\"hi\")?;"),
        see_also: &["ConversationIo", "ConversationSerialization", "MemoryNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const CONVERSATION_IO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(47),
        long_code: "HHS_E_CONVERSATION_IO",
        short_code: "E0047",
        title: "Conversation Persistence I/O Failure",
        short_description: "Reading or writing the on-disk conversation store failed at the OS level.",
        long_description: "Conversations can be persisted to disk so multi-turn history survives across runs. This error wraps the underlying OS error (permission denied, disk full, file not found, etc.) raised when the store tried to read or write its files.

Check the directory exists, the process has read/write permission, and there is free disk space. If you are running multiple instances, make sure they are not contending on the same file without locking.

The wrapped OS error in the message is the authoritative diagnosis — start there.",
        hints: &["Verify the conversation directory exists and is writable", "Check disk space with `df -h`", "Avoid sharing the same store across processes without locks", "Inspect the wrapped OS error for the root cause"],
        example_bad: None,
        example_good: None,
        see_also: &["ConversationSerialization", "MemoryBackend", "PersistenceIo"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const CONVERSATION_SERIALIZATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(48),
        long_code: "HHS_E_CONVERSATION_SERIALIZATION",
        short_code: "E0048",
        title: "Conversation Failed To Serialize Or Parse",
        short_description: "A conversation could not be encoded for saving or decoded after loading.",
        long_description: "Conversations are stored as JSON. This error means the encoder or decoder rejected the value — typically because the on-disk format doesn't match the current schema (after an upgrade), or because a message contains a non-serializable extension field.

Migrate old conversations using `conversation.migrate()` after upgrades, and avoid stuffing arbitrary runtime objects into message metadata. If decoding fails on a single corrupt file, move it aside rather than deleting and reload the rest of the store.

Keep backups before major version bumps so you can roll back if migration breaks unexpectedly.",
        hints: &["Run `conversation.migrate()` after upgrades", "Don't store closures or raw handles in message metadata", "Quarantine corrupt files instead of deleting them", "Back up the conversation directory before upgrades"],
        example_bad: None,
        example_good: None,
        see_also: &["ConversationIo", "ConversationEmpty", "MemorySerialization"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const FALLBACK_ALL_PROVIDERS_EXHAUSTED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(80),
        long_code: "HHS_E_FALLBACK_ALL_PROVIDERS_EXHAUSTED",
        short_code: "E0080",
        title: "Every Fallback Provider Failed In Sequence",
        short_description: "All providers in the fallback chain were tried and all of them failed; the last error is reported.",
        long_description: "A fallback chain tries providers in priority order until one succeeds. This error means none of them did. The last provider's error is included so you can see why the final attempt failed, but earlier providers may have failed for unrelated reasons.

Check the chain configuration, provider health, and credentials. Enable per-attempt logging to see which providers failed and why. Don't catch this and silently return placeholder content — surface it so users know the AI subsystem is fully degraded.

Consider adding a free local provider (Ollama) at the bottom of the chain so you always have a last resort.",
        hints: &["Enable `fallback.log_attempts = true` to see each failure", "Add a local Ollama provider as a last-resort fallback", "Verify all providers' credentials are still valid", "Check the wrapped last-error for the most recent cause"],
        example_bad: None,
        example_good: None,
        see_also: &["FallbackEmptyChain", "ProviderNotConfigured", "ProviderApiError"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const FALLBACK_EMPTY_CHAIN: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(81),
        long_code: "HHS_E_FALLBACK_EMPTY_CHAIN",
        short_code: "E0081",
        title: "Fallback Chain Has No Providers Configured",
        short_description: "A fallback dispatcher was invoked with zero providers in its chain.",
        long_description: "A fallback chain must contain at least one provider to be useful. This error fires when the chain is empty at the moment of dispatch — usually because configuration loading failed silently or all providers were filtered out by health checks.

Load your providers config and verify at least one entry is present and healthy. If you build the chain dynamically, assert non-empty before dispatch and surface the misconfiguration loudly at startup.

Failing fast at boot is much better than this error appearing in production traffic.",
        hints: &["Add at least one provider to `fallback.providers`", "Assert non-empty chain at startup, not at first call", "Don't let health checks remove the entire chain silently", "Log the resolved chain after configuration loads"],
        example_bad: Some("let chain = FallbackChain::new();
chain.send(req)?; // empty"),
        example_good: Some("let chain = FallbackChain::from_config(&config)?;
assert!(!chain.is_empty());"),
        see_also: &["FallbackAllProvidersExhausted", "ProviderNotConfigured"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const MEMORY_BACKEND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(124),
        long_code: "HHS_E_MEMORY_BACKEND",
        short_code: "E0124",
        title: "Long-Term Memory Backend Failed",
        short_description: "The underlying storage backend for long-term memory raised an error.",
        long_description: "AI tools can persist long-term memory to a backend (sled, sqlite, Redis, file). When that backend fails — disk full, lock contention, network partition — this error wraps the underlying cause.

Treat the wrapped message as authoritative. For transient backends (network) retry with backoff; for hard backends (disk full) free space and restart writes. If you suspect lock contention, reduce write concurrency or switch to a backend with better concurrent semantics.

Monitor backend health independently of the AI tools so you catch failures before they cascade into agent errors.",
        hints: &["Read the wrapped backend message for the real cause", "Free disk space if the backend is local", "Reduce concurrency if you see lock contention", "Add backend health checks to your monitoring"],
        example_bad: None,
        example_good: None,
        see_also: &["MemoryNotFound", "MemorySerialization", "StorePersistError"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const MEMORY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(125),
        long_code: "HHS_E_MEMORY_NOT_FOUND",
        short_code: "E0125",
        title: "Memory Entry Missing By Key",
        short_description: "A long-term memory lookup by key returned no entry.",
        long_description: "Long-term memory entries are addressed by key. This error fires when you read a key that has never been written, was deleted, or was evicted by a TTL or capacity policy.

Use `memory.try_get(key)` for optional reads, or check existence with `memory.contains(key)` before fetching. If you expected the entry to be there, verify TTL settings and check whether another writer cleared it.

Do not synthesize fake memory in response to this error — that hides bugs in your write path.",
        hints: &["Use `memory.try_get(key)` for optional reads", "Check `memory.contains(key)` before fetching", "Verify TTL settings aren't expiring entries early", "Audit other writers that might delete entries"],
        example_bad: None,
        example_good: None,
        see_also: &["MemoryBackend", "MemorySerialization", "ConversationEmpty"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const MEMORY_SERIALIZATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(126),
        long_code: "HHS_E_MEMORY_SERIALIZATION",
        short_code: "E0126",
        title: "Memory Entry Serialization Or Parse Failure",
        short_description: "A memory value could not be encoded for storage or decoded on retrieval.",
        long_description: "Memory entries are serialized to bytes before storage. This error means the encoder or decoder rejected the value — typically a schema drift after an upgrade, or a non-serializable field.

Migrate old entries with `memory.migrate()` after upgrades, and avoid storing closures or raw handles. For decode failures on a subset of entries, quarantine the bad ones rather than wiping the entire store.

Version your memory schema explicitly so future migrations are predictable.",
        hints: &["Run `memory.migrate()` after upgrades", "Don't store closures in memory entries", "Quarantine corrupt entries instead of deleting all", "Version your memory schema explicitly"],
        example_bad: None,
        example_good: None,
        see_also: &["MemoryBackend", "MemoryNotFound", "ConversationSerialization"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_API_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(197),
        long_code: "HHS_E_PROVIDER_API_ERROR",
        short_code: "E0197",
        title: "LLM Provider API Returned An Error",
        short_description: "The upstream LLM provider responded with an API-level error such as 4xx or 5xx.",
        long_description: "This error wraps whatever the provider returned: invalid request, model not available, server error, content filter, or auth failure. The wrapped message contains the provider's own error string and is your primary diagnostic.

Auth and request errors require config or code fixes. Server errors are usually transient — retry with backoff. Content filter errors require prompt rewriting. For persistent failures, fail over to another provider via a fallback chain.

Log the request id from the wrapped message so you can correlate with the provider's own dashboards.",
        hints: &["Read the wrapped provider message for the exact reason", "Retry 5xx errors with exponential backoff", "Rewrite prompts that trip content filters", "Use a fallback chain for persistent provider outages"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderNetworkError", "ProviderNotConfigured", "FallbackAllProvidersExhausted"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_BUDGET_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(198),
        long_code: "HHS_E_PROVIDER_BUDGET_EXCEEDED",
        short_code: "E0198",
        title: "Per-Request Token Budget Exceeded",
        short_description: "A single LLM request was blocked because its requested token count exceeds the per-call limit.",
        long_description: "Each provider call has a per-request token budget guard. If your request (prompt + max output) would exceed the configured limit, dispatch is refused and this error reports both the requested amount and the cap.

Shorten the prompt, lower `max_tokens`, or increase the per-request limit if it is unrealistically tight. For workflows that genuinely need long context, use a model with a larger context window rather than fighting the guard.

This is distinct from daily/monthly budgets — those track cumulative spend over time.",
        hints: &["Lower `max_tokens` on the request", "Trim system prompt and conversation history", "Increase the per-request token cap if intentional", "Switch to a longer-context model when needed"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderDailyBudgetExceeded", "ProviderMonthlyBudgetExceeded", "CostBudgetExceeded"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_DAILY_BUDGET_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(199),
        long_code: "HHS_E_PROVIDER_DAILY_BUDGET_EXCEEDED",
        short_code: "E0199",
        title: "Daily Token Or Spend Budget Exceeded",
        short_description: "Cumulative usage today has reached the daily budget cap, blocking further provider calls.",
        long_description: "The runtime tracks per-day usage per provider. Once today's total reaches the configured daily limit, further calls are blocked until midnight (in the configured timezone) or until you raise the cap.

Raise the daily cap, wait for the next day, or route traffic to a different provider whose daily budget still has headroom. Investigate whether the spike is from a runaway loop before simply raising the cap.

Daily budgets are the most common backstop against unexpected agent behaviour — keep them in place even after raising them.",
        hints: &["Raise `provider.daily_limit` if the spend is legitimate", "Investigate runaway loops before lifting the cap", "Route to a backup provider via fallback chain", "Check the configured timezone for the daily reset"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderMonthlyBudgetExceeded", "ProviderBudgetExceeded", "CostBudgetExceeded"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_INVALID_CONFIG: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(200),
        long_code: "HHS_E_PROVIDER_INVALID_CONFIG",
        short_code: "E0200",
        title: "Provider Configuration Is Invalid",
        short_description: "A provider was constructed with a configuration that failed validation.",
        long_description: "Provider configs have required fields (api key, base url, model) and value ranges. This error fires at construction or first use when those constraints are violated — missing api key, malformed url, unknown model, negative timeout, etc.

Load config with the validating constructor and surface errors at startup, not at first request. The wrapped message names the offending field.

Keep secrets out of code. Use environment variables or a secret manager so config parses consistently in every environment.",
        hints: &["Validate config at startup, not at first call", "Read the wrapped message for the offending field", "Load secrets from env vars or a secret manager", "Use `Provider::from_env()` helpers when available"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderNotConfigured", "ProviderNotFound", "TokenomicsConfigError"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_MONTHLY_BUDGET_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(201),
        long_code: "HHS_E_PROVIDER_MONTHLY_BUDGET_EXCEEDED",
        short_code: "E0201",
        title: "Monthly Token Or Spend Budget Exceeded",
        short_description: "Cumulative usage this month has reached the monthly budget cap, blocking further provider calls.",
        long_description: "Like the daily budget, the monthly budget caps total usage over a calendar month. When reached, this error blocks new calls until the next month begins or you raise the cap.

Monthly caps catch slow leaks that daily caps miss. Raise carefully and only after you understand why the spend is high. Route to a different provider for the rest of the month if you need continuity.

Dashboard the monthly burn-down so you see the trajectory days before hitting the cap.",
        hints: &["Raise `provider.monthly_limit` only after understanding the cause", "Route traffic to a backup provider for the rest of the month", "Dashboard burn-down to predict cap hits", "Investigate slow leaks before they hit the cap"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderDailyBudgetExceeded", "ProviderBudgetExceeded", "TokenomicsInsufficientBudget"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_NETWORK_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(202),
        long_code: "HHS_E_PROVIDER_NETWORK_ERROR",
        short_code: "E0202",
        title: "Network Failure Talking To Provider",
        short_description: "The HTTP transport to the LLM provider failed before a response could be received.",
        long_description: "This error covers DNS failures, connection refused, TLS handshake errors, timeouts, and similar transport-level issues. The provider may be perfectly healthy — the failure is between you and them.

Retry with exponential backoff for transient network errors. If you see this consistently, check DNS, firewall rules, proxy settings, and your local clock (TLS is sensitive to clock skew). For private endpoints, verify the route and credentials.

Distinguish this from `ProviderApiError`: this one means we never got a response, that one means we got an error response.",
        hints: &["Retry with exponential backoff for transient errors", "Check DNS, firewall, and proxy configuration", "Verify system clock — TLS rejects skewed clocks", "Configure a generous but bounded request timeout"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderApiError", "FallbackAllProvidersExhausted"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_NOT_CONFIGURED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(203),
        long_code: "HHS_E_PROVIDER_NOT_CONFIGURED",
        short_code: "E0203",
        title: "Provider Referenced But Not Configured",
        short_description: "A call referenced a provider name that has no configuration loaded.",
        long_description: "Providers must be registered in config before they can be used. This error fires when a call names a provider that the registry has never seen — typo, missing config file, or provider was disabled.

Add the provider to your config, or correct the name at the call site. If providers are loaded from environment variables, double-check that the relevant env vars are set in the current shell.

For multi-tenant setups, ensure tenant-specific provider configs are merged into the registry before serving traffic.",
        hints: &["Add the provider to `providers.toml`", "Correct any typos in the provider name", "Verify env vars for env-driven configs", "Check provider isn't filtered out by feature flags"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderNotFound", "ProviderInvalidConfig", "FallbackEmptyChain"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(204),
        long_code: "HHS_E_PROVIDER_NOT_FOUND",
        short_code: "E0204",
        title: "Provider Lookup Returned Nothing",
        short_description: "The provider registry has no entry under the requested name at lookup time.",
        long_description: "This is the registry-side mirror of `ProviderNotConfigured`: the registry was queried for a provider id and returned nothing. Causes overlap — typo, missing config, runtime removal — but this error fires at the registry boundary rather than the dispatch boundary.

List registered providers with `registry.list()` to see what is actually present. If providers are loaded lazily, ensure the loader has run before the lookup.

If you hot-reload provider configs, hold a read lock during the lookup so a concurrent reload can't remove the provider mid-call.",
        hints: &["Call `registry.list()` to see registered providers", "Ensure lazy loaders run before first lookup", "Hold a read lock during lookup if hot-reloading", "Match the lookup name to the registry exactly"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderNotConfigured", "ProviderInvalidConfig"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_OPTIMIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(205),
        long_code: "HHS_E_PROVIDER_OPTIMIZATION_ERROR",
        short_code: "E0205",
        title: "Token Optimization Pass Failed",
        short_description: "The pre-dispatch token optimization step failed to compress or trim a request.",
        long_description: "Before sending requests, the runtime can run optional optimization passes (deduplication, summarization, eviction of old turns). This error fires when one of those passes errors out — for example, the summarizer model failed, or the trimmer couldn't reach the target size.

Disable optimization for the request to bypass the issue, or fix the underlying summarizer/trimmer config. If optimization fails frequently, the budget you are trying to fit into is probably unrealistically tight.

This error doesn't reflect a problem with the original request — it's the optimizer that failed.",
        hints: &["Bypass optimization for the affected request", "Loosen the target token budget", "Fix the summarizer model config", "Inspect the wrapped error for which pass failed"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderBudgetExceeded", "ProviderApiError", "TokenomicsPredictionFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub const PROVIDER_SERIALIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(206),
        long_code: "HHS_E_PROVIDER_SERIALIZATION_ERROR",
        short_code: "E0206",
        title: "Provider Request Or Response Serialization Failed",
        short_description: "Encoding the request body or decoding the response body for an LLM provider failed.",
        long_description: "This error fires when the request/response codec for a provider rejects a value — usually because the provider's API contract changed and our adapter is out of date, or because a custom message extension is not encodable.

Update the runtime to a version with the latest provider adapters. For custom extensions, ensure they implement `Serialize`/`Deserialize` correctly. Log the wrapped message; the field name in it usually points right at the problem.

If the API contract changed upstream, file a bug — adapters need a patch to keep up.",
        hints: &["Upgrade to a runtime with current provider adapters", "Verify custom extensions implement serde traits", "Inspect the wrapped message for the offending field", "File a bug if the upstream API contract changed"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderApiError", "ConversationSerialization"],
        since_version: "0.4.0",
        category: ErrorCategory::Ai,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    CONVERSATION_EMPTY,
    CONVERSATION_IO,
    CONVERSATION_SERIALIZATION,
    FALLBACK_ALL_PROVIDERS_EXHAUSTED,
    FALLBACK_EMPTY_CHAIN,
    MEMORY_BACKEND,
    MEMORY_NOT_FOUND,
    MEMORY_SERIALIZATION,
    PROVIDER_API_ERROR,
    PROVIDER_BUDGET_EXCEEDED,
    PROVIDER_DAILY_BUDGET_EXCEEDED,
    PROVIDER_INVALID_CONFIG,
    PROVIDER_MONTHLY_BUDGET_EXCEEDED,
    PROVIDER_NETWORK_ERROR,
    PROVIDER_NOT_CONFIGURED,
    PROVIDER_NOT_FOUND,
    PROVIDER_OPTIMIZATION_ERROR,
    PROVIDER_SERIALIZATION_ERROR,
];
