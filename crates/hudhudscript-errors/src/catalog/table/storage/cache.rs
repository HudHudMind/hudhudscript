use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const CACHE_CONSTITUTION_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(16),
        long_code: "HHS_E_CACHE_CONSTITUTION_NOT_FOUND",
        short_code: "E0016",
        title: "Constitution Entry Missing From Cache",
        short_description: "The cache could not find a constitution document with the requested identifier.",
        long_description: "Constitutions are top-level governance documents in the HudHudScript cache hierarchy (constitution > law > rule). This error fires when a lookup is performed for a constitution id that has never been inserted, was evicted, or was deleted by another writer.\n\nVerify the id you are passing is the canonical hash, not a human alias, and check that the cache has been warmed from its persistent backing store on startup. If you rely on a remote cache, also confirm replication has finished before issuing reads.\n\nThis commonly happens during cold boots, after `cache.clear()`, or when a node joins a cluster before catching up on snapshots.",
        hints: &["Call `cache.warm()` during startup to preload constitutions", "Use `cache.contains(id)` before `get` to avoid throwing", "Confirm the id is the content hash, not a display name", "Check that eviction policy isn't dropping rarely accessed roots"],
        example_bad: Some("let c = cache.get_constitution(\"my-cool-rules\"); // human alias, not an id"),
        example_good: Some("let id = cache.resolve_alias(\"my-cool-rules\")?;\nlet c = cache.get_constitution(id)?;"),
        see_also: &["CacheLawNotFound", "CacheRuleNotFound", "CacheIdCollision"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_DESERIALIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(17),
        long_code: "HHS_E_CACHE_DESERIALIZATION_ERROR",
        short_code: "E0017",
        title: "Cache Entry Failed To Deserialize",
        short_description: "A cached blob could not be decoded back into its expected typed value.",
        long_description: "The cache stores values as serialized bytes (bincode/JSON depending on the entry kind) and decodes them on read. This error means the bytes were retrieved successfully but the decoder rejected them — usually because the schema changed between writer and reader versions, or the entry was written with a different codec.\n\nMigrate the cache when you bump on-disk types, or version your value types so old entries can be detected and dropped. Never share a single cache directory between two HudHudScript versions with incompatible types.\n\nIf you see this only for a subset of keys, those entries are likely from a previous schema and should be invalidated.",
        hints: &["Bump the cache namespace when value types change", "Wrap typed values in a `{ version, payload }` envelope", "Run `cache.invalidate_stale()` after upgrades", "Check both writer and reader use the same serde features"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheSerializationError", "CacheIdCollision"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_DUPLICATE_CONTENT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(18),
        long_code: "HHS_E_CACHE_DUPLICATE_CONTENT",
        short_code: "E0018",
        title: "Identical Content Already Stored Under Another Key",
        short_description: "An insert was rejected because identical content already exists in the cache under a different id.",
        long_description: "The cache enforces content-addressed deduplication: two entries with the same hash cannot live under different ids. When you try to insert content that is already present, the cache rejects the write and reports both the requested key and the existing one.\n\nThis usually means you are computing ids manually instead of letting the cache hash content for you, or you are inserting the same payload twice with different metadata. Reuse the existing id rather than creating a new one, or add the metadata as a side index.\n\nDeduplication is a feature, not a bug — it keeps storage bounded for repeated prompts and shared rules.",
        hints: &["Call `cache.put_or_get(content)` instead of `put(id, content)`", "Use the existing id reported in the error message", "Store metadata in a side table keyed by content hash", "Don't pre-hash content yourself unless you trust your hash"],
        example_bad: Some("cache.put(\"v1\", payload);\ncache.put(\"v2\", payload); // same bytes, fails"),
        example_good: Some("let id = cache.put_or_get(payload);"),
        see_also: &["CacheIdCollision", "CacheConstitutionNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_ID_COLLISION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(19),
        long_code: "HHS_E_CACHE_ID_COLLISION",
        short_code: "E0019",
        title: "Cache Id Collision Between Different Contents",
        short_description: "Two distinct payloads hashed to the same cache id, indicating a hash collision or manual id reuse.",
        long_description: "Each cache entry is keyed by a content hash. A collision means the same id maps to two different byte sequences, which should not happen with the default cryptographic hash unless you are supplying ids manually or have downgraded to a weaker hash for testing.\n\nIf you are passing custom ids, switch to the auto-derived id. If you are seeing this with auto ids, it almost certainly means corruption — restore from snapshot.\n\nDo not catch this and continue: silent collisions cause silent data loss for downstream consumers.",
        hints: &["Stop passing manual ids — let the cache hash content", "Verify hash function isn't downgraded in test config", "Restore from the most recent good snapshot", "File a bug if you can reproduce with default settings"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheDuplicateContent", "CacheDeserializationError"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_LAW_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(20),
        long_code: "HHS_E_CACHE_LAW_NOT_FOUND",
        short_code: "E0020",
        title: "Law Entry Missing From Cache",
        short_description: "A specific law document referenced under a constitution could not be located in the cache.",
        long_description: "Laws live one level below constitutions in the cache hierarchy. This error means the constitution may exist but the requested law id is not present — either it was never written, was evicted, or you are reading from a stale snapshot.\n\nFetch the parent constitution first and enumerate its laws to confirm what is actually present. If the law was added recently, give the cache replication a chance to converge before retrying.\n\nDuring constitutional updates this is normal mid-flight; treat it as eventually-consistent and retry with backoff.",
        hints: &["List laws via `constitution.laws()` to see what exists", "Retry with backoff if you just wrote the law", "Make sure you reference the law by its content id", "Confirm eviction policy isn't pruning your law tier"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheConstitutionNotFound", "CacheRuleNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_QUOTA_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(21),
        long_code: "HHS_E_CACHE_QUOTA_EXCEEDED",
        short_code: "E0021",
        title: "Cache Capacity Quota Exceeded",
        short_description: "An insert was rejected because the cache has hit its configured capacity quota.",
        long_description: "The cache enforces a hard quota — bytes, entries, or both — to keep memory bounded. Once full, writes either evict older entries (under LRU policy) or reject new inserts (under reject-on-full policy). This error means the policy is reject-on-full and you have hit the ceiling.\n\nRaise the quota in config, switch the policy to LRU, or explicitly evict cold entries before retrying. Don't ignore this — falling back to an unbounded store will eventually OOM the process.\n\nMonitor quota usage with `cache.stats()` so you see pressure before it becomes a hard error.",
        hints: &["Increase `cache.max_bytes` or `cache.max_entries`", "Switch eviction policy to LRU if you can tolerate misses", "Call `cache.evict_oldest(n)` to make room manually", "Track `cache.stats().pressure` in your dashboards"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheConstitutionNotFound", "TokenomicsInsufficientBudget"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_RULE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(22),
        long_code: "HHS_E_CACHE_RULE_NOT_FOUND",
        short_code: "E0022",
        title: "Rule Entry Missing From Cache",
        short_description: "A specific rule document referenced under a law could not be located in the cache.",
        long_description: "Rules are the leaf level of the cache hierarchy (constitution > law > rule). This error means a lookup for a specific rule id failed. As with laws, the parent law may still exist; only the leaf is missing.\n\nEnumerate the parent law's rules to see what is present, and confirm you are using the rule's content id rather than a draft alias. Rules are the most frequently evicted tier under LRU because they are both numerous and individually small.\n\nIf you depend on a specific rule being resident, mark it as pinned via `cache.pin(id)`.",
        hints: &["Enumerate `law.rules()` to verify the id", "Pin critical rules with `cache.pin(id)`", "Lower the eviction aggressiveness for the rule tier", "Re-derive the rule from source if it's reproducible"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheLawNotFound", "CacheConstitutionNotFound"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };

pub const CACHE_SERIALIZATION_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(23),
        long_code: "HHS_E_CACHE_SERIALIZATION_ERROR",
        short_code: "E0023",
        title: "Cache Value Failed To Serialize",
        short_description: "A value could not be encoded for storage in the cache.",
        long_description: "Before writing, the cache encodes values into bytes. This error fires when the encoder rejects the value — usually because it contains a non-serializable field (a closure, a raw pointer, a non-`Serialize` extension type) or recursive cycles the codec can't handle.\n\nReplace non-serializable fields with serializable surrogates, or use `#[serde(skip)]` for transient state that doesn't need persisting. For cyclic structures, switch to id-based references.\n\nWhen mixing JSON and bincode codecs, be aware that JSON cannot represent some integer types and binary blobs efficiently.",
        hints: &["Add `#[serde(skip)]` to runtime-only fields", "Replace closures with named functions or enum tags", "Break cycles using id references rather than direct links", "Pick bincode over JSON for binary-heavy payloads"],
        example_bad: None,
        example_good: None,
        see_also: &["CacheDeserializationError"],
        since_version: "0.4.2",
        category: ErrorCategory::Storage,
    };
