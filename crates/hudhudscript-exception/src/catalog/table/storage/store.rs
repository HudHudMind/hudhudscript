use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const STORE_DIMENSION_MISMATCH: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(263),
        long_code: "HHS_E_STORE_DIMENSION_MISMATCH",
        short_code: "E0263",
        title: "Vector Store Dimension Mismatch",
        short_description: "An insert or query vector did not match the dimension the store was created with.",
        long_description: "A vector store is created with a fixed dimension count. Inserting or querying with a vector of a different size is rejected to prevent silent corruption of nearest-neighbour calculations.

Verify that the embedding model used to produce the vector matches the model the store was built for. Mixing models within one store is unsupported — rebuild the store from scratch when you change models.

Record the embedding model id alongside vectors so you can audit which writer produced which entries.",
        hints: &["Use one embedding model per vector store", "Rebuild the store when changing models", "Record the model id with each vector", "Document the expected dimension in store config"],
        example_bad: None,
        example_good: None,
        see_also: &["EmbeddingInvalidDimensions", "StoreInvalidConfig", "IndexEmbedding"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const STORE_INVALID_CONFIG: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(264),
        long_code: "HHS_E_STORE_INVALID_CONFIG",
        short_code: "E0264",
        title: "Vector Store Configuration Is Invalid",
        short_description: "A vector store was constructed with a configuration that failed validation.",
        long_description: "Store configs have required fields (path, dimension, distance metric) and value ranges. This error fires at construction when those constraints fail — missing path, zero dimension, unknown metric.

Fix the named field and reconstruct. Catching this at startup is much better than catching it at first query, so always construct stores during initialization rather than lazily.

The wrapped message names the offending field.",
        hints: &["Read the wrapped message for the offending field", "Construct stores at startup, not lazily", "Validate dimensions against your embedding model", "Pick a distance metric appropriate for your vectors"],
        example_bad: None,
        example_good: None,
        see_also: &["StoreDimensionMismatch", "StoreNotFound", "StorePersistError"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const STORE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(265),
        long_code: "HHS_E_STORE_NOT_FOUND",
        short_code: "E0265",
        title: "Store Entry Missing By Key",
        short_description: "A vector store lookup by id returned no entry.",
        long_description: "Vector stores are also key-addressed for their metadata. This error fires when you ask for a specific entry by id and it isn't present — never written, deleted, or evicted.

Use `store.try_get(id)` for optional reads, and verify the id matches what was used at write time. If you suspect concurrent deletes, hold a read lock during the lookup.

For RAG retrieval, a missing entry on a hit is sometimes a sign that the index and metadata store are out of sync — rebuild from source if it persists.",
        hints: &["Use `store.try_get(id)` for optional reads", "Verify the id matches the write-side id", "Rebuild if index and metadata desync", "Check for races between deletes and reads"],
        example_bad: None,
        example_good: None,
        see_also: &["IndexNotFound", "StorePersistError", "MemoryNotFound"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const STORE_PERSIST_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(266),
        long_code: "HHS_E_STORE_PERSIST_ERROR",
        short_code: "E0266",
        title: "Vector Store Persist Operation Failed",
        short_description: "Saving the vector store to its persistent backing file failed.",
        long_description: "Vector stores can flush their state to disk for durability. This error wraps an underlying I/O or codec failure during that flush — disk full, permission denied, file lock contention, or schema mismatch.

The wrapped message identifies the cause. Free disk space, fix permissions, or migrate after schema upgrades. Do not ignore persist errors — in-memory state will be lost on next restart.

For hot stores, schedule periodic snapshots and monitor that the snapshot files are actually being written.",
        hints: &["Inspect the wrapped error for the root cause", "Free disk space on the backing volume", "Migrate schemas after runtime upgrades", "Monitor snapshot file timestamps"],
        example_bad: None,
        example_good: None,
        see_also: &["StoreInvalidConfig", "PersistenceIo", "TokenomicsStorageError"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };
