use std::collections::HashMap;
use std::sync::Arc;

use crate::tvar::{commit_lock, TVar};

struct ReadEntry {
    version_at_read: u64,
}

struct WriteEntry<V> {
    new_value: V,
}

/// Per-transaction log of reads and writes.
pub struct Transaction<V> {
    reads: HashMap<Arc<TVar<V>>, ReadEntry>,
    writes: HashMap<Arc<TVar<V>>, WriteEntry<V>>,
}

impl<V: Clone> Default for Transaction<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone> Transaction<V> {
    pub fn new() -> Self {
        Transaction {
            reads: HashMap::new(),
            writes: HashMap::new(),
        }
    }

    /// Read from `tvar`, returning the value visible in this transaction.
    pub fn read(&mut self, tvar: &Arc<TVar<V>>) -> V {
        if let Some(entry) = self.writes.get(tvar) {
            return entry.new_value.clone();
        }
        let (value, version) = tvar.read_committed();
        self.reads.entry(Arc::clone(tvar)).or_insert(ReadEntry {
            version_at_read: version,
        });
        value
    }

    /// Stage a write to `tvar` within this transaction.
    pub fn write(&mut self, tvar: &Arc<TVar<V>>, value: V) {
        self.reads.entry(Arc::clone(tvar)).or_insert_with(|| {
            let (_, version) = tvar.read_committed();
            ReadEntry {
                version_at_read: version,
            }
        });
        self.writes
            .insert(Arc::clone(tvar), WriteEntry { new_value: value });
    }

    /// Attempt to commit. Returns `true` on success, `false` on conflict.
    pub fn try_commit(self) -> bool {
        let _guard = commit_lock().lock();

        for (tvar, read_entry) in &self.reads {
            let (_, current_version) = tvar.read_committed();
            if current_version != read_entry.version_at_read {
                return false;
            }
        }

        for (tvar, write_entry) in self.writes {
            tvar.commit_write(write_entry.new_value);
        }

        true
    }

    /// Number of TVars this transaction has read. Useful for testing.
    pub fn read_count(&self) -> usize {
        self.reads.len()
    }

    /// Number of TVars this transaction has staged writes for. Useful for testing.
    pub fn write_count(&self) -> usize {
        self.writes.len()
    }
}
