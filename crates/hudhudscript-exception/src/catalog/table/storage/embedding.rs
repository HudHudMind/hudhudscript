use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const EMBEDDING_EMPTY_INPUT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(75),
        long_code: "HHS_E_EMBEDDING_EMPTY_INPUT",
        short_code: "E0075",
        title: "Embedding Input Text Is Empty",
        short_description: "An embedding was requested for an empty string, which most providers reject outright.",
        long_description: "Embedding models require at least one token to produce a vector. Passing an empty string or a string that contains only whitespace results in this error before the call is dispatched, sparing you a wasted API request and a downstream provider error.

Filter empty inputs at the source: skip them, replace with a placeholder, or treat them as a zero vector if your application semantics allow it. Chunked pipelines should drop empty chunks rather than fail the whole batch.

This is purely a defensive guard — there is no provider you can configure your way out of it.",
        hints: &["Skip empty strings before calling `embed()`", "Trim and check `input.is_empty()` upstream", "Drop empty chunks from batches rather than failing all", "Map empty inputs to a sentinel zero vector if useful"],
        example_bad: Some("let v = embed(\"\")?;"),
        example_good: Some("if !text.trim().is_empty() {
    let v = embed(text)?;
}"),
        see_also: &["EmbeddingInvalidDimensions", "EmbeddingProviderError"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const EMBEDDING_INVALID_DIMENSIONS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(76),
        long_code: "HHS_E_EMBEDDING_INVALID_DIMENSIONS",
        short_code: "E0076",
        title: "Embedding Vector Has Wrong Dimensionality",
        short_description: "An embedding vector did not have the dimensionality expected by the index or downstream model.",
        long_description: "Vector indexes are built for a specific dimension count (e.g., 1536 for OpenAI ada-002, 768 for many open models). If a vector with a different size is inserted or queried, this error fires.

The usual cause is mixing embeddings from two different models in the same index. Pick one model per index, and rebuild the index from scratch if you need to change models. Storing the model id alongside vectors makes this kind of mismatch easier to detect.

Some providers allow truncating to a smaller dimension — only do this when both writer and reader agree.",
        hints: &["Use one embedding model per index", "Store the model id alongside each vector", "Rebuild the index when you change embedding models", "Document the expected dimension in your config"],
        example_bad: None,
        example_good: None,
        see_also: &["StoreDimensionMismatch", "EmbeddingProviderError", "IndexEmbedding"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const EMBEDDING_PROVIDER_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(77),
        long_code: "HHS_E_EMBEDDING_PROVIDER_ERROR",
        short_code: "E0077",
        title: "Embedding Provider Returned An Error",
        short_description: "The upstream embedding provider failed the request — quota, network, or model load issue.",
        long_description: "Embedding generation is delegated to a provider (cloud API or local model server). This error wraps whatever the provider reported: rate limit, auth failure, model not loaded, network reset.

Read the wrapped message for the root cause. Add retry-with-backoff for transient errors, raise quotas for persistent rate limits, and ensure local models are loaded before serving traffic.

If the provider is flaky, configure a fallback embedding provider so RAG queries don't fail outright.",
        hints: &["Inspect the wrapped provider message for specifics", "Retry transient errors with exponential backoff", "Pre-load local embedding models at startup", "Configure a fallback embedding provider"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingEmptyInput", "EmbeddingInvalidDimensions", "ProviderApiError"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };
