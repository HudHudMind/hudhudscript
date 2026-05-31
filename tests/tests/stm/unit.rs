use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use hudhudscript_errors::ErrorCode;
use hudhudscript_stm::atomically;
use hudhudscript_stm::TVar;
use hudhudscript_stm::TVarRegistry;
use hudhudscript_stm::{err_max_retries_exceeded, err_timeout};

type V = i64;

#[test]
fn tvar_basic_read_write() {
    let t: Arc<TVar<V>> = TVar::new(10);
    assert_eq!(t.read_committed().0, 10);
    atomically::<V, _, _>(|tx| {
        tx.write(&t, 42);
        Ok(())
    })
    .unwrap();
    assert_eq!(t.read_committed().0, 42);
}

#[test]
fn atomically_read_modify_write() {
    let t: Arc<TVar<V>> = TVar::new(0);
    for _ in 0..5 {
        atomically::<V, _, _>(|tx| {
            let v = tx.read(&t);
            tx.write(&t, v + 1);
            Ok(())
        })
        .unwrap();
    }
    assert_eq!(t.read_committed().0, 5);
}

#[test]
fn conflict_causes_retry() {
    let t: Arc<TVar<V>> = TVar::new(0);
    let attempts = AtomicUsize::new(0);

    atomically::<V, _, _>(|tx| {
        let n = attempts.fetch_add(1, Ordering::SeqCst);
        let v = tx.read(&t);
        if n == 0 {
            t.commit_write(v + 100);
        }
        tx.write(&t, v + 1);
        Ok(())
    })
    .unwrap();

    assert!(attempts.load(Ordering::SeqCst) >= 2, "must have retried");
    assert_eq!(t.read_committed().0, 101);
}

#[test]
fn concurrent_increments_serialize() {
    use std::thread;
    let t: Arc<TVar<V>> = TVar::new(0);
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let t = Arc::clone(&t);
            thread::spawn(move || {
                for _ in 0..50 {
                    atomically::<V, _, _>(|tx| {
                        let v = tx.read(&t);
                        tx.write(&t, v + 1);
                        Ok(())
                    })
                    .unwrap();
                }
            })
        })
        .collect();
    for h in threads {
        h.join().unwrap();
    }
    assert_eq!(t.read_committed().0, 400);
}

#[test]
fn registry_roundtrip() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let id = reg.create(7);
    assert_eq!(reg.read(&id).unwrap(), 7);
    reg.write_direct(&id, 99).unwrap();
    assert_eq!(reg.read(&id).unwrap(), 99);
    assert_eq!(reg.len(), 1);
}

#[test]
fn create_with_id_is_idempotent() {
    let reg: TVarRegistry<V> = TVarRegistry::new();
    let id1 = reg.create_with_id("counter", 10);
    let id2 = reg.create_with_id("counter", 999);
    assert_eq!(id1, id2);
    assert_eq!(reg.read("counter").unwrap(), 10);
}

#[test]
fn max_retries_exceeded_error_code() {
    let e = err_max_retries_exceeded(1000);
    assert_eq!(e.code, ErrorCode::RuntimeStmMaxRetriesExceeded);
}

#[test]
fn timeout_error_code() {
    let e = err_timeout(100, 200);
    assert_eq!(e.code, ErrorCode::RuntimeStmTimeout);
}
