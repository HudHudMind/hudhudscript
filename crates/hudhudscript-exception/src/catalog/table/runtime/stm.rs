use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_STM_MAX_RETRIES_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(245),
        long_code: "HHS_E_RUNTIME_STM_MAX_RETRIES_EXCEEDED",
        short_code: "E0245",
        title: "STM transaction exceeded retry limit",
        short_description: "An `atomically` block conflicted with other transactions too many times in a row and was aborted to prevent livelock.",
        long_description: "HudHudScript's STM is optimistic: each transaction runs on a snapshot, then validates and commits. On a conflict, the transaction retries. To prevent livelock, the runtime caps consecutive retries; when the cap is hit, this error is raised rather than looping forever.

Livelock usually indicates high write contention on a small set of TVars. To fix, reduce contention by splitting hot TVars, ordering operations so writes do not overlap, reducing transaction size (shorter transactions conflict less often), or using coarser locking if the workload is truly write-heavy.

Keep in mind that `await` is forbidden inside `atomically` — if you suspect long transactions, audit for blocking operations that should live outside the atomic block.",
        hints: &["Split hot TVars so writers do not all collide on the same cell", "Shrink the atomic block — do expensive work outside it", "Never `await` inside `atomically` (it is forbidden)", "Consider an actor/message pattern for extreme write contention"],
        example_bad: Some("// tight loop of conflicting writers on a single TVar
atomically { counter.set(counter.get() + 1); }"),
        example_good: Some("// shard the counter to reduce contention
atomically { shards[tid % N].update(|v| v + 1); }"),
        see_also: &["HHS_E_RUNTIME_STM_TIMEOUT", "HHS_E_RUNTIME_STATE_ERROR", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.4.10",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_STM_TIMEOUT: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(246),
        long_code: "HHS_E_RUNTIME_STM_TIMEOUT",
        short_code: "E0246",
        title: "STM transaction timed out",
        short_description: "An `atomically` block ran longer than its configured time limit and was aborted.",
        long_description: "In addition to a retry cap, STM transactions have a wall-clock timeout to bound their worst-case latency. When a transaction (including its retries) exceeds the limit, the runtime aborts it with this error. The message reports the elapsed time and the configured limit.

The root cause is usually one of: the transaction body is genuinely slow (too much work inside the atomic block), extreme contention is causing many retries, or the timeout is mis-configured for the workload.

Fix by shrinking the transaction, reducing contention, or — if the workload is legitimate and bounded — raising the STM timeout via the embedder. Remember that `await` is forbidden inside `atomically`, so unexpected latency should not come from async operations.",
        hints: &["Shrink the atomic block — move expensive computation outside it", "Reduce contention (see: StmMaxRetriesExceeded)", "Never `await` inside `atomically`", "If legitimate, ask the embedder to raise the STM timeout"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_STM_MAX_RETRIES_EXCEEDED", "HHS_E_RUNTIME_OUT_OF_GAS", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.4.10",
        category: ExceptionCategory::Runtime,
    };
