use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const EMBEDDING_EMPTY_INPUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(75),
        long_code: "HHS_E_EMBEDDING_EMPTY_INPUT",
        short_code: "E0075",
        title: "Embedding Input Text Is Empty",
        short_description: "The embedding subsystem received an empty string and cannot produce a vector.",
        long_description: "Embedding models require non-empty text to produce a meaningful vector. Passing an empty string, a string of only whitespace, or a null document body will trigger this error.\n\nFilter empty inputs at the call site, or return a zero vector for empty documents if your use case permits. Batch pipelines should validate documents before sending them to the embedding stage.",
        hints: &["Skip empty documents instead of embedding them", "Return a zero vector for empty strings if semantics allow", "Add a `is_empty()` guard before the embedding call", "Validate document bodies at ingestion time"],
        example_bad: Some("let v = embed(\"\"); // empty input"),
        example_good: Some("let text = doc.body.trim();\nif text.is_empty() { return vec![0.0; DIM]; }\nlet v = embed(text);"),
        see_also: &["EmbeddingInvalidDimensions", "EmbeddingProviderError"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const EMBEDDING_INVALID_DIMENSIONS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(76),
        long_code: "HHS_E_EMBEDDING_INVALID_DIMENSIONS",
        short_code: "E0076",
        title: "Embedding Vector Has Wrong Dimensionality",
        short_description: "The embedding subsystem returned a vector whose length does not match the expected dimension.",
        long_description: "Every embedding model has a fixed output dimension (e.g. 768 for `text-embedding-3-small`, 1536 for `text-embedding-3-large`). This error means the returned vector length differs from what the index or store was configured for.\n\nUsually this happens after switching embedding models without rebuilding the index, or when a provider returns a truncated or padded vector. Rebuild the index with the new model, or add a dimension adapter layer.",
        hints: &["Rebuild the index after changing embedding models", "Add a dimension-mismatch adapter if you must support multiple models", "Verify the provider's dimension in its documentation", "Check that batch results aren't concatenated incorrectly"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingEmptyInput", "StoreDimensionMismatch"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const EMBEDDING_PROVIDER_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(77),
        long_code: "HHS_E_EMBEDDING_PROVIDER_ERROR",
        short_code: "E0077",
        title: "Embedding Provider Returned An Error",
        short_description: "The external embedding provider (Ollama, OpenAI, etc.) returned an error instead of a vector.",
        long_description: "This is a wrapper around a provider-specific failure — rate limit, authentication, model not found, or transient outage. The wrapped error contains the provider's raw response.\n\nInspect the wrapped error to determine whether to retry (transient) or fix config (permanent). For rate limits, back off and retry. For auth errors, rotate credentials. For model-not-found, verify the model name.",
        hints: &["Check the wrapped provider error for retryability", "Back off on rate-limit errors", "Rotate API keys on authentication failures", "Verify the model name in your config"],
        example_bad: None,
        example_good: None,
        see_also: &["ProviderApiError", "ProviderNetworkError"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const INDEX_DUPLICATE_ID: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(110),
        long_code: "HHS_E_INDEX_DUPLICATE_ID",
        short_code: "E0110",
        title: "Document Id Already Exists In Index",
        short_description: "An index insert was rejected because the document id is already present.",
        long_description: "Indexes enforce unique document ids within a collection. This error means you tried to insert a document with an id that already exists.\n\nUse `index.upsert(doc)` if you want to overwrite, or generate a fresh id if this is a new document. For immutable logs, append a sequence number or timestamp to the id.",
        hints: &["Use `index.upsert()` instead of `insert()` if overwrites are intended", "Generate a new UUID for every distinct document", "Append a timestamp to ids for append-only logs", "Check for duplicates before batch insertion"],
        example_bad: Some("index.insert(\"doc-1\", content1);\nindex.insert(\"doc-1\", content2); // duplicate id"),
        example_good: Some("index.upsert(\"doc-1\", content2); // overwrites"),
        see_also: &["IndexNotFound", "EmbeddingEmptyInput"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const INDEX_EMBEDDING: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(111),
        long_code: "HHS_E_INDEX_EMBEDDING",
        short_code: "E0111",
        title: "Embedding Step Failed During Indexing",
        short_description: "The index could not embed a document because the embedding subsystem returned an error.",
        long_description: "Indexing a document requires producing its embedding first. This error wraps an `EmbeddingError` raised during that step — for example, the provider was down, the input was empty, or the dimension was wrong.\n\nFix the embedding-side problem (check the wrapped cause), then retry the indexing operation. For batch jobs, isolate the offending document so the rest of the batch can complete.\n\nThe wrapped error tells you whether the issue is transient (retry) or structural (config fix).",
        hints: &["Check the wrapped embedding error for the root cause", "Retry transient provider errors", "Fix empty inputs before re-indexing", "Isolate bad documents in batch jobs"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingProviderError", "EmbeddingEmptyInput"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const INDEX_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(112),
        long_code: "HHS_E_INDEX_NOT_FOUND",
        short_code: "E0112",
        title: "Document Not Found In Index",
        short_description: "A lookup by id returned no document from the index.",
        long_description: "The index was queried for a specific document id that does not exist. This is the normal result of a failed lookup — not necessarily a bug.\n\nConfirm the id is correct, that the document was successfully indexed, and that you are querying the right collection. For eventually-consistent indexes, allow a short delay between insert and query.",
        hints: &["Verify the document id is correct", "Check that the document was indexed successfully", "Query the right collection", "Allow replication delay for distributed indexes"],
        example_bad: None,
        example_good: None,
        see_also: &["IndexDuplicateId", "ResourceNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const PERSISTENCE_IO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(188),
        long_code: "HHS_E_PERSISTENCE_IO",
        short_code: "E0188",
        title: "Snapshot Or Restore I/O Failure",
        short_description: "A snapshot or restore operation failed due to an underlying I/O error.",
        long_description: "The persistence layer writes snapshots to disk and restores them on restart. This error wraps an `std::io::Error` raised during either operation — disk full, permission denied, path not found, etc.\n\nCheck disk space, file permissions, and the snapshot directory path. For remote storage backends, verify network connectivity and credentials.",
        hints: &["Check disk space on the snapshot volume", "Verify file permissions for the snapshot directory", "Confirm the path exists and is writable", "Check network connectivity for remote backends"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceNotFound", "PersistenceSerialization"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const PERSISTENCE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(189),
        long_code: "HHS_E_PERSISTENCE_NOT_FOUND",
        short_code: "E0189",
        title: "No Snapshot Found For Agent",
        short_description: "A restore was requested for an agent that has no stored snapshot.",
        long_description: "This is a normal first-run condition — agents start without a snapshot and create one on shutdown. If you expected a snapshot, verify the agent id and the snapshot directory.\n\nFor new agents, ignore this and let the first shutdown create the snapshot. For migrated agents, copy the snapshot file into the expected directory.",
        hints: &["Ignore on first run — snapshot will be created on shutdown", "Verify the agent id matches the snapshot file name", "Check the snapshot directory path in config", "Copy snapshots manually for migrated agents"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceIo", "PersistenceSerialization"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const PERSISTENCE_SERIALIZATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(190),
        long_code: "HHS_E_PERSISTENCE_SERIALIZATION",
        short_code: "E0190",
        title: "Snapshot Serialization Or Parse Failure",
        short_description: "A snapshot could not be serialized or deserialized.",
        long_description: "Snapshots are serialized with a stable codec (bincode or JSON). This error means the encoder or decoder failed — schema mismatch, unsupported type, corrupted file, etc.\n\nIf this happens during restore, the snapshot file may be from an incompatible version. If during save, the agent state may contain non-serializable data.\n\nVersion your snapshot schema and provide migration paths between versions.",
        hints: &["Version snapshot schemas and migrate on load", "Avoid storing non-serializable types in agent state", "Validate snapshot files with a checksum", "Keep old snapshot versions for rollback"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceIo", "CacheDeserializationError"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const STORE_DIMENSION_MISMATCH: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(263),
        long_code: "HHS_E_STORE_DIMENSION_MISMATCH",
        short_code: "E0263",
        title: "Vector Store Dimension Mismatch",
        short_description: "A vector store received a vector whose dimension does not match the store's configured dimension.",
        long_description: "Vector stores are initialized with a fixed dimension. All inserts and queries must use vectors of that exact length. This error means a vector of the wrong length was passed.\n\nUsually this happens after switching embedding models without re-initializing the store. Rebuild the store with the new dimension, or add an adapter layer.",
        hints: &["Rebuild the store after changing embedding models", "Add a dimension adapter if supporting multiple models", "Validate vector length before insertion", "Check that the store was initialized with the correct dimension"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingInvalidDimensions", "StoreInvalidConfig"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const STORE_INVALID_CONFIG: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(264),
        long_code: "HHS_E_STORE_INVALID_CONFIG",
        short_code: "E0264",
        title: "Vector Store Configuration Is Invalid",
        short_description: "The vector store was initialized with an invalid configuration.",
        long_description: "Vector stores require valid config — dimension > 0, a supported distance metric, and a backend that is compiled in. This error means one or more of these constraints was violated.\n\nCheck the config for negative dimensions, unsupported metrics, or missing backend features. Use the default config if you are unsure.",
        hints: &["Ensure dimension is a positive integer", "Use a supported distance metric (cosine, euclidean, dot)", "Enable the required backend feature at build time", "Use the default config as a starting point"],
        example_bad: None,
        example_good: None,
        see_also: &["StoreDimensionMismatch", "StoreNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const STORE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(265),
        long_code: "HHS_E_STORE_NOT_FOUND",
        short_code: "E0265",
        title: "Store Entry Missing By Key",
        short_description: "A lookup in the vector store returned no entry for the requested key.",
        long_description: "This is the normal result of a failed lookup — the key was never inserted, or it was deleted. Confirm the key is correct and that the store was populated before querying.\n\nFor new stores, this is expected until the first insert. For cleared stores, all keys are gone.",
        hints: &["Verify the key is correct", "Check that the store was populated", "Handle missing keys gracefully in queries", "Confirm the store wasn't cleared"],
        example_bad: None,
        example_good: None,
        see_also: &["StorePersistError", "IndexNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const STORE_PERSIST_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(266),
        long_code: "HHS_E_STORE_PERSIST_ERROR",
        short_code: "E0266",
        title: "Vector Store Persist Operation Failed",
        short_description: "The vector store could not write its state to disk.",
        long_description: "Vector stores may persist to disk for durability. This error wraps an I/O error during that write — disk full, permission denied, path not found, etc.\n\nCheck disk space, permissions, and the persist directory. For remote backends, verify connectivity.",
        hints: &["Check disk space", "Verify permissions", "Confirm the persist directory exists", "Check network for remote backends"],
        example_bad: None,
        example_good: None,
        see_also: &["StoreNotFound", "PersistenceIo"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };
