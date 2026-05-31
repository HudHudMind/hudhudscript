use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const INDEX_DUPLICATE_ID: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(110),
        long_code: "HHS_E_INDEX_DUPLICATE_ID",
        short_code: "E0110",
        title: "Document Id Already Exists In Index",
        short_description: "An attempt to insert a document failed because another document with the same id is already indexed.",
        long_description: "Vector indexes treat document ids as unique. Inserting a second document under an existing id is rejected to avoid silent overwrites and stale references.

Use `index.upsert(id, doc)` if you actually want to replace, or generate a fresh id if you intended a new document. If you are bulk-loading and ids are derived from content, you may have actual duplicates in your source data — deduplicate upstream.

Silent overwrites would corrupt RAG retrieval, so the index errs on the side of refusal.",
        hints: &["Call `index.upsert(id, doc)` instead of `insert` to replace", "Generate fresh ids for new documents", "Deduplicate source data before bulk-loading", "Use content-hash ids only when you want dedup"],
        example_bad: Some("index.insert(\"doc-1\", a)?;
index.insert(\"doc-1\", b)?; // duplicate id"),
        example_good: Some("index.insert(\"doc-1\", a)?;
index.upsert(\"doc-1\", b)?;"),
        see_also: &["IndexNotFound", "StoreNotFound", "CacheDuplicateContent"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const INDEX_EMBEDDING: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(111),
        long_code: "HHS_E_INDEX_EMBEDDING",
        short_code: "E0111",
        title: "Embedding Step Failed During Indexing",
        short_description: "The index could not embed a document because the embedding subsystem returned an error.",
        long_description: "Indexing a document requires producing its embedding first. This error wraps an `EmbeddingError` raised during that step — for example, the provider was down, the input was empty, or the dimension was wrong.

Fix the embedding-side problem (check the wrapped cause), then retry the indexing operation. For batch jobs, isolate the offending document so the rest of the batch can complete.

The wrapped error tells you whether the issue is transient (retry) or structural (config fix).",
        hints: &["Inspect the wrapped EmbeddingError for the root cause", "Skip and quarantine bad documents in batch jobs", "Retry transient embedding failures with backoff", "Pre-validate document text before sending to the index"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingProviderError", "EmbeddingInvalidDimensions", "IndexNotFound"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const INDEX_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(112),
        long_code: "HHS_E_INDEX_NOT_FOUND",
        short_code: "E0112",
        title: "Document Not Found In Index",
        short_description: "A lookup or delete by id failed because no such document is present in the index.",
        long_description: "This error fires when you reference an indexed document by id and it isn't there. Either it was never inserted, it was already deleted, or you are reading from a stale snapshot.

List the index contents to verify, and remember that deletes are visible immediately on the same handle but may take time to propagate to other readers in distributed setups. If the id came from a search result, ensure no concurrent writer removed it between the search and the read.

For optional reads, use `index.try_get(id)` which returns `None` instead of raising.",
        hints: &["Use `index.try_get(id)` for optional lookups", "Verify the id matches an existing document", "Check for races between deletes and reads", "Refresh the index handle after concurrent updates"],
        example_bad: None,
        example_good: None,
        see_also: &["IndexDuplicateId", "StoreNotFound", "MemoryNotFound"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };
