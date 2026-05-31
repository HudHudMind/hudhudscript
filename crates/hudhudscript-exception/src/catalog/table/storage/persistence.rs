use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const PERSISTENCE_IO: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(188),
        long_code: "HHS_E_PERSISTENCE_IO",
        short_code: "E0188",
        title: "Snapshot Or Restore I/O Failure",
        short_description: "Reading or writing actor/STM snapshot files failed at the OS level.",
        long_description: "The runtime persists actor and STM state by writing snapshot files. This error wraps an OS error (permission denied, disk full, file not found) raised during snapshot or restore.

Verify the snapshot directory exists and is writable, check disk space, and make sure no other process holds an exclusive lock on the snapshot files. The wrapped OS error is the authoritative diagnosis.

If the underlying failure is transient (e.g. NFS hiccup), the runtime will retry on the next snapshot cycle — but corruption from a partial write requires manual intervention.",
        hints: &["Verify the snapshot directory is writable", "Check disk space and inode count", "Ensure no other process locks the snapshot files", "Inspect the wrapped OS error for the root cause"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceNotFound", "PersistenceSerialization", "ConversationIo"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const PERSISTENCE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(189),
        long_code: "HHS_E_PERSISTENCE_NOT_FOUND",
        short_code: "E0189",
        title: "No Snapshot Found For Agent",
        short_description: "A restore was attempted for an agent that has no saved snapshot on disk.",
        long_description: "Each agent's persisted state is keyed by agent id. This error fires when restore is requested for an id that has no snapshot — first run, snapshot was deleted, or the id is wrong.

If this is the first run, treat the missing snapshot as expected and start from initial state. If you expected a snapshot, verify the agent id matches what was used at write time, and check the snapshot directory for files.

Agent ids should be stable across restarts — randomly regenerating them on each boot is a common source of this error.",
        hints: &["Treat missing snapshots as 'first run' for new agents", "Verify agent id is stable across restarts", "List the snapshot directory to confirm what's saved", "Use `persist.try_restore()` to handle missing gracefully"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceIo", "PersistenceSerialization"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };

pub const PERSISTENCE_SERIALIZATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(190),
        long_code: "HHS_E_PERSISTENCE_SERIALIZATION",
        short_code: "E0190",
        title: "Snapshot Serialization Or Parse Failure",
        short_description: "An actor or STM snapshot could not be encoded for saving or decoded on restore.",
        long_description: "Snapshots are serialized representations of actor mailboxes and STM transaction logs. This error means the codec rejected the value — usually a schema mismatch between the running version and the version that wrote the snapshot.

Do not silently overwrite the bad snapshot. Quarantine it, run `persist.migrate()` if a migration exists, or roll back the runtime to a compatible version. For STM specifically, a corrupt snapshot can lose committed transactions, so caution is warranted.

Version your actor state types explicitly with serde to enable forward-compatible migrations.",
        hints: &["Quarantine corrupt snapshots instead of overwriting", "Run `persist.migrate()` after runtime upgrades", "Roll back the runtime if migration isn't available", "Version actor state types with explicit schema tags"],
        example_bad: None,
        example_good: None,
        see_also: &["PersistenceIo", "PersistenceNotFound", "MemorySerialization"],
        since_version: "0.4.2",
        category: ExceptionCategory::Storage,
    };
