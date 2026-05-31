#[cfg(test)]
mod tests {
    use hudhudscript_async::blocking_registry::*;
    use std::sync::mpsc;

    #[test]
    fn spawn_then_await_returns_real_value() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id = reg.spawn_task(|| Ok(42));
        let val = reg.await_blocking(&id).expect("should resolve");
        assert_eq!(val, 42);
    }

    #[test]
    fn spawn_then_await_propagates_rejection() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id = reg.spawn_task(|| Err("boom".to_string()));
        match reg.await_blocking(&id) {
            Err(RegistryError::Rejected(msg)) => assert_eq!(msg, "boom"),
            other => panic!("expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn store_result_is_retrieved() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id = reg.store_result(Ok(7));
        let val = reg.await_blocking(&id).expect("cached resolves");
        assert_eq!(val, 7);
    }

    #[test]
    fn unregistered_id_reports_error_not_null() {
        // Core correctness invariant (previously silently fabricated a
        // value — that silent path was eliminated): await on an unknown
        // id must surface an error rather than return a default.
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        match reg.await_blocking("promise-unknown") {
            Err(RegistryError::Unregistered(_)) => {}
            other => panic!("expected Unregistered, got {:?}", other),
        }
    }

    #[test]
    fn await_blocking_consumes_entry_once() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id = reg.store_result(Ok(1));
        let _ = reg.await_blocking(&id).unwrap();
        match reg.await_blocking(&id) {
            Err(RegistryError::Unregistered(_)) => {}
            other => panic!("expected Unregistered on second await, got {:?}", other),
        }
    }

    #[test]
    fn register_external_wires_caller_owned_channel() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let (tx, rx) = mpsc::channel();
        let id = reg.register_external(rx);
        tx.send(Ok(99)).unwrap();
        assert_eq!(reg.await_blocking(&id).unwrap(), 99);
    }

    #[test]
    fn await_all_blocking_returns_values_in_input_order() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        // Two tasks with inverted completion order: id_a sleeps longer.
        let id_a = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(80));
            Ok(1)
        });
        let id_b = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(20));
            Ok(2)
        });
        let ids = [id_a.as_str(), id_b.as_str()];
        let start = std::time::Instant::now();
        let out = reg.await_all_blocking(&ids).expect("all resolve");
        let elapsed = start.elapsed();
        assert_eq!(out, vec![1, 2], "results must follow input order");
        // Concurrent: should be ~max(80, 20) ≈ 80ms, not 100ms.
        assert!(
            elapsed < std::time::Duration::from_millis(180),
            "await_all_blocking should be concurrent, took {:?}",
            elapsed
        );
    }

    #[test]
    fn await_all_blocking_propagates_first_rejection() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id_a = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(1)
        });
        let id_b = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            Err("boom".to_string())
        });
        let ids = [id_a.as_str(), id_b.as_str()];
        let start = std::time::Instant::now();
        let err = reg.await_all_blocking(&ids).unwrap_err();
        let elapsed = start.elapsed();
        match err {
            RegistryError::Rejected(msg) => assert_eq!(msg, "boom"),
            other => panic!("expected Rejected, got {:?}", other),
        }
        // Should return near-immediately after the fast rejection, not
        // wait for the slow resolver.
        assert!(
            elapsed < std::time::Duration::from_millis(80),
            "await_all_blocking should short-circuit on reject, took {:?}",
            elapsed
        );
    }

    #[test]
    fn await_all_blocking_handles_cached_and_threaded_mix() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let cached_id = reg.store_result(Ok(7));
        let live_id = reg.spawn_task(|| Ok(8));
        let ids = [cached_id.as_str(), live_id.as_str()];
        let out = reg.await_all_blocking(&ids).expect("both resolve");
        assert_eq!(out, vec![7, 8]);
    }

    #[test]
    fn await_race_blocking_first_to_finish_wins() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id_slow = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(120));
            Ok(100)
        });
        let id_fast = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(20));
            Ok(200)
        });
        let ids = [id_slow.as_str(), id_fast.as_str()];
        let start = std::time::Instant::now();
        let (idx, val) = reg.await_race_blocking(&ids).expect("one resolves");
        let elapsed = start.elapsed();
        assert_eq!(idx, 1, "fast promise at idx 1 must win");
        assert_eq!(val, 200);
        assert!(
            elapsed < std::time::Duration::from_millis(90),
            "race must return near the fast completion, took {:?}",
            elapsed
        );
    }

    #[test]
    fn await_race_blocking_cached_short_circuits_in_order() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id_live = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok(1)
        });
        let id_cached = reg.store_result(Ok(42));
        let ids = [id_live.as_str(), id_cached.as_str()];
        let (idx, val) = reg.await_race_blocking(&ids).expect("resolves");
        assert_eq!(idx, 1);
        assert_eq!(val, 42);
    }

    #[test]
    fn await_race_blocking_rejection_wins_if_first() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        let id_rej = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            Err("early".to_string())
        });
        let id_ok = reg.spawn_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(80));
            Ok(1)
        });
        let ids = [id_rej.as_str(), id_ok.as_str()];
        match reg.await_race_blocking(&ids) {
            Err(RegistryError::Rejected(msg)) => assert_eq!(msg, "early"),
            other => panic!("expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn await_race_blocking_empty_input_reports_error() {
        let mut reg: PromiseRegistry<i32> = PromiseRegistry::new();
        match reg.await_race_blocking(&[]) {
            Err(RegistryError::Unregistered(tag)) => assert_eq!(tag, "race:empty"),
            other => panic!("expected Unregistered(race:empty), got {:?}", other),
        }
    }
}
