//! Extended STM tests — edge cases and retry behavior.
//! Covers: TVar::id, TVarRegistry operations, atomically retry limits, StmConfig.

use hudhudscript_stm::{
    atomically, atomically_with_config, StmConfig, TVar, TVarRegistry,
};
use std::sync::Arc;

type V = i64;

// ── TVar identity ──────────────────────────────────────────────────

#[test]
fn tvar_id_is_unique() {
    let a: Arc<TVar<V>> = TVar::new(1);
    let b: Arc<TVar<V>> = TVar::new(2);
    assert_ne!(a.id(), b.id());
}

#[test]
fn tvar_eq_reflexive() {
    let a: Arc<TVar<V>> = TVar::new(42);
    assert_eq!(a, a);
}

#[test]
fn tvar_eq_different_tvars() {
    let a: Arc<TVar<V>> = TVar::new(1);
    let b: Arc<TVar<V>> = TVar::new(1);
    assert_ne!(a, b);
}

#[test]
fn tvar_hash_consistent_with_eq() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let a: Arc<TVar<V>> = TVar::new(1);
    let b: Arc<TVar<V>> = TVar::new(1);
    let mut ha = DefaultHasher::new();
    let mut hb = DefaultHasher::new();
    a.hash(&mut ha);
    b.hash(&mut hb);
    assert_ne!(ha.finish(), hb.finish());
}

#[test]
fn tvar_initial_value_is_correct() {
    let t: Arc<TVar<V>> = TVar::new(99);
    assert_eq!(t.read_committed().0, 99);
}

// ── Transaction read/write log ──────────────────────────────────────

#[test]
fn transaction_read_count_zero_when_empty() {
    use hudhudscript_stm::Transaction;
    let tx: Transaction<V> = Transaction::new();
    assert_eq!(tx.read_count(), 0);
    assert_eq!(tx.write_count(), 0);
}

#[test]
fn transaction_read_count_increases_after_read() {
    use hudhudscript_stm::Transaction;
    let t: Arc<TVar<V>> = TVar::new(0);
    let mut tx: Transaction<V> = Transaction::new();
    let _ = tx.read(&t);
    assert_eq!(tx.read_count(), 1);
}

#[test]
fn transaction_write_count_increases_after_write() {
    use hudhudscript_stm::Transaction;
    let t: Arc<TVar<V>> = TVar::new(0);
    let mut tx: Transaction<V> = Transaction::new();
    tx.write(&t, 42);
    assert_eq!(tx.write_count(), 1);
}

#[test]
fn transaction_read_after_write_sees_written_value() {
    use hudhudscript_stm::Transaction;
    let t: Arc<TVar<V>> = TVar::new(0);
    let mut tx: Transaction<V> = Transaction::new();
    tx.write(&t, 42);
    let v = tx.read(&t);
    assert_eq!(v, 42);
}

#[test]
fn transaction_read_counts_once_even_after_multiple_reads() {
    use hudhudscript_stm::Transaction;
    let t: Arc<TVar<V>> = TVar::new(0);
    let mut tx: Transaction<V> = Transaction::new();
    let _ = tx.read(&t);
    let _ = tx.read(&t);
    let _ = tx.read(&t);
    assert_eq!(tx.read_count(), 1);
}

#[test]
fn transaction_multiple_tvars() {
    use hudhudscript_stm::Transaction;
    let a: Arc<TVar<V>> = TVar::new(10);
    let b: Arc<TVar<V>> = TVar::new(20);
    let mut tx: Transaction<V> = Transaction::new();
    let va = tx.read(&a);
    let vb = tx.read(&b);
    tx.write(&a, va + vb);
    assert!(tx.try_commit());
    assert_eq!(a.read_committed().0, 30);
    assert_eq!(b.read_committed().0, 20);
}

// ── atomically basic ops ────────────────────────────────────────────

#[test]
fn atomically_write_multiple_vars() {
    let a: Arc<TVar<V>> = TVar::new(0);
    let b: Arc<TVar<V>> = TVar::new(0);
    atomically::<V, _, _>(|tx| {
        tx.write(&a, 10);
        tx.write(&b, 20);
        Ok(())
    })
    .unwrap();
    assert_eq!(a.read_committed().0, 10);
    assert_eq!(b.read_committed().0, 20);
}

#[test]
fn atomically_swap_two_vars() {
    let a: Arc<TVar<V>> = TVar::new(100);
    let b: Arc<TVar<V>> = TVar::new(200);
    atomically::<V, _, _>(|tx| {
        let va = tx.read(&a);
        let vb = tx.read(&b);
        tx.write(&a, vb);
        tx.write(&b, va);
        Ok(())
    })
    .unwrap();
    assert_eq!(a.read_committed().0, 200);
    assert_eq!(b.read_committed().0, 100);
}

#[test]
fn atomically_accumulate_across_transactions() {
    let t: Arc<TVar<V>> = TVar::new(0);
    for i in 1..=100 {
        atomically::<V, _, _>(|tx| {
            let v = tx.read(&t);
            tx.write(&t, v + i);
            Ok(())
        })
        .unwrap();
    }
    assert_eq!(t.read_committed().0, 5050); // sum(1..=100)
}

// ── atomically retry on failure ─────────────────────────────────────

#[test]
fn atomically_retries_when_commit_fails() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let t: Arc<TVar<V>> = TVar::new(0);
    let attempts = AtomicUsize::new(0);

    let result = atomically::<V, _, _>(|tx| {
        let n = attempts.fetch_add(1, Ordering::SeqCst);
        let v = tx.read(&t);
        if n == 0 {
            // Simulate conflict: external write before commit
            t.commit_write(v + 100);
        }
        tx.write(&t, v + 1);
        Ok::<(i64,), _>((v + 1,))
    });

    assert!(result.is_ok());
    assert!(attempts.load(Ordering::SeqCst) >= 2);
}

#[test]
fn atomically_config_max_retries_reached() {
    let t: Arc<TVar<V>> = TVar::new(0);
    let config = StmConfig {
        max_retries: 2,
        timeout_ms: 10000,
        initial_backoff_us: 0,
        max_backoff_us: 0,
    };

    let result = atomically_with_config::<V, _, _>(
        |tx| {
            let v = tx.read(&t);
            // Always cause a conflict
            t.commit_write(v + 1);
            tx.write(&t, v + 2);
            Ok(())
        },
        config,
    );
    assert!(result.is_err());
}

#[test]
fn atomically_config_default_has_reasonable_values() {
    let config = StmConfig::default();
    assert!(config.max_retries > 0);
    assert!(config.timeout_ms > 0);
    assert!(config.max_backoff_us >= config.initial_backoff_us);
}

// ── Registry operations ────────────────────────────────────────────

#[test]
fn registry_create_read_default_id() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let id = reg.create(42);
    assert_eq!(reg.read(&id).unwrap(), 42);
}

#[test]
fn registry_len_tracks_entries() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    assert_eq!(reg.len(), 0);
    reg.create(1);
    assert_eq!(reg.len(), 1);
    reg.create(2);
    assert_eq!(reg.len(), 2);
}

#[test]
fn registry_read_nonexistent_returns_error() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let result = reg.read("nonexistent");
    assert!(result.is_err());
}

#[test]
fn registry_write_direct_nonexistent_returns_error() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let result = reg.write_direct("nonexistent", 5);
    assert!(result.is_err());
}

#[test]
fn registry_create_multiple_different_ids() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let id1 = reg.create(1);
    let id2 = reg.create(2);
    assert_ne!(id1, id2);
    assert_eq!(reg.read(&id1).unwrap(), 1);
    assert_eq!(reg.read(&id2).unwrap(), 2);
}

#[test]
fn registry_create_with_id_reads_back_correctly() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let id = reg.create_with_id("counter", 0);
    assert_eq!(reg.read("counter").unwrap(), 0);
    reg.write_direct(&id, 5).unwrap();
    assert_eq!(reg.read("counter").unwrap(), 5);
}
