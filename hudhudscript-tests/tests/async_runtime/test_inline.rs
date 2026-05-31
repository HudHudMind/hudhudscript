use hudhudscript_async::{
    promise_all, promise_race, AsyncRuntime, Promise, PromiseError, PromiseState,
};
use tokio::time::{sleep, Duration};

// ── combinators.rs tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_promise_all_success() {
    let p1 = Promise::resolved(1);
    let p2 = Promise::resolved(2);
    let p3 = Promise::resolved(3);

    let results = promise_all(vec![p1, p2, p3]).await.unwrap();
    assert_eq!(results, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_promise_all_failure() {
    let p1 = Promise::resolved(1);
    let p2: Promise<i32> = Promise::rejected("error".to_string());
    let p3 = Promise::resolved(3);

    let result = promise_all(vec![p1, p2, p3]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise_all_async() {
    let (p1, r1) = Promise::new();
    let (p2, r2) = Promise::new();

    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        r1.resolve(1).await.unwrap();
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        r2.resolve(2).await.unwrap();
    });

    let results = promise_all(vec![p1, p2]).await.unwrap();
    assert_eq!(results, vec![1, 2]);
}

#[tokio::test]
async fn test_promise_race_first_wins() {
    let (p1, r1) = Promise::new();
    let (p2, r2) = Promise::new();

    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        r1.resolve(1).await.unwrap();
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        r2.resolve(2).await.unwrap();
    });

    let result = promise_race(vec![p1, p2]).await.unwrap();
    assert_eq!(result, 1);
}

#[tokio::test]
async fn test_promise_race_empty() {
    let result: Result<i32, _> = promise_race(vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise_race_immediate() {
    let p1 = Promise::resolved(42);
    let (p2, _r2) = Promise::new();

    let result = promise_race(vec![p1, p2]).await.unwrap();
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_promise_all_single_element() {
    let p = Promise::resolved(99);
    let results = promise_all(vec![p]).await.unwrap();
    assert_eq!(results, vec![99]);
}

#[tokio::test]
async fn test_promise_race_rejected_first() {
    let p1: Promise<i32> = Promise::rejected("fast fail".to_string());
    let (p2, _r2) = Promise::<i32>::new();

    let result = promise_race(vec![p1, p2]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise_race_single_element() {
    let p = Promise::resolved(7);
    let result = promise_race(vec![p]).await.unwrap();
    assert_eq!(result, 7);
}

// ── promise.rs tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_promise_resolved() {
    let promise = Promise::resolved(42);
    assert!(promise.is_resolved().await);
    assert_eq!(promise.await_result().await.unwrap(), 42);
}

#[tokio::test]
async fn test_promise_rejected() {
    let promise: Promise<i32> = Promise::rejected("error".to_string());
    assert!(promise.is_rejected().await);
    assert!(promise.await_result().await.is_err());
}

#[tokio::test]
async fn test_promise_resolve() {
    let (promise, resolver) = Promise::new();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        resolver.resolve(42).await.unwrap();
    });

    let result = promise.await_result().await.unwrap();
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_promise_reject() {
    let (promise, resolver) = Promise::<i32>::new();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        resolver.reject("test error".to_string()).await.unwrap();
    });

    let result = promise.await_result().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise_state() {
    let (promise, resolver) = Promise::new();
    assert!(promise.is_pending().await);

    resolver.resolve(42).await.unwrap();
    assert!(promise.is_resolved().await);
}

#[tokio::test]
async fn test_promise_state_rejected() {
    let (promise, resolver) = Promise::<i32>::new();
    resolver.reject("err".to_string()).await.unwrap();

    let state = promise.state().await;
    assert!(matches!(state, PromiseState::Rejected(ref msg) if msg == "err"));
}

#[tokio::test]
async fn test_promise_resolve_already_resolved() {
    let (_promise, resolver) = Promise::<i32>::new();
    resolver.resolve(1).await.unwrap();
    // Can't call resolve again since resolver is consumed (moved).
    // This is enforced by Rust's type system.
}

#[tokio::test]
async fn test_promise_debug_format() {
    let (promise, resolver) = Promise::<i32>::new();
    let dbg_promise = format!("{:?}", promise);
    assert!(dbg_promise.contains("Promise"));

    let dbg_resolver = format!("{:?}", resolver);
    assert!(dbg_resolver.contains("PromiseResolver"));
}

#[tokio::test]
async fn test_promise_resolver_id() {
    let (promise, resolver) = Promise::<i32>::new();
    assert_eq!(promise.id(), resolver.id());
}

#[tokio::test]
async fn test_promise_error_display() {
    let e1 = PromiseError::AlreadyResolved;
    assert!(format!("{}", e1).contains("Promise already resolved"));

    let e2 = PromiseError::AlreadyRejected;
    assert!(format!("{}", e2).contains("Promise already rejected"));

    let e3 = PromiseError::Rejected("oops".to_string());
    assert!(format!("{}", e3).contains("Promise was rejected: oops"));

    let e4 = PromiseError::ReceiverDropped;
    assert!(format!("{}", e4).contains("Promise receiver dropped"));
}

#[tokio::test]
async fn test_promise_await_result_receiver_dropped() {
    let (promise, resolver) = Promise::<i32>::new();
    drop(resolver);
    let result = promise.await_result().await;
    assert!(matches!(result, Err(PromiseError::ReceiverDropped)));
}

// ── runtime.rs tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_runtime_spawn_task() {
    let runtime = AsyncRuntime::new();

    let promise = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(10)).await;
                resolver.resolve(42).await.unwrap();
            })
        })
        .await;

    let result = promise.await_result().await.unwrap();
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_runtime_get_promise() {
    let runtime = AsyncRuntime::new();

    let promise = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                resolver.resolve(42).await.unwrap();
            })
        })
        .await;

    let promise_id = promise.id();
    let retrieved = runtime.get_promise(promise_id).await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_runtime_remove_promise() {
    let runtime = AsyncRuntime::new();

    let promise = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(42).await.unwrap();
            })
        })
        .await;

    let promise_id = promise.id();
    runtime.remove_promise(promise_id).await;

    assert!(runtime.get_promise(promise_id).await.is_none());
}

#[tokio::test]
async fn test_runtime_active_count() {
    let runtime = AsyncRuntime::new();
    assert_eq!(runtime.active_count().await, 0);

    let _promise = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                resolver.resolve(42).await.unwrap();
            })
        })
        .await;

    assert_eq!(runtime.active_count().await, 1);
}

#[tokio::test]
async fn test_runtime_cancel_all() {
    let runtime = AsyncRuntime::new();

    let _p1 = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                resolver.resolve(1).await.unwrap();
            })
        })
        .await;

    let _p2 = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                resolver.resolve(2).await.unwrap();
            })
        })
        .await;

    assert_eq!(runtime.active_count().await, 2);

    runtime.cancel_all().await;
    assert_eq!(runtime.active_count().await, 0);
}

#[tokio::test]
async fn test_runtime_get_nonexistent_promise() {
    let runtime: AsyncRuntime<i32> = AsyncRuntime::new();
    let fake_id = uuid::Uuid::new_v4();
    assert!(runtime.get_promise(fake_id).await.is_none());
}

#[tokio::test]
async fn test_runtime_error_display() {
    use hudhudscript_async::AsyncRuntimeError;
    let e1 = AsyncRuntimeError::PromiseNotFound(uuid::Uuid::nil());
    assert!(format!("{}", e1).contains("Promise not found"));

    let e2 = AsyncRuntimeError::TaskSpawnFailed("spawn failed".to_string());
    assert!(format!("{}", e2).contains("Task spawn failed: spawn failed"));

    let e3 = AsyncRuntimeError::RuntimeError("generic error".to_string());
    assert!(format!("{}", e3).contains("Runtime error: generic error"));
}

#[tokio::test]
async fn test_runtime_remove_nonexistent_no_panic() {
    let runtime: AsyncRuntime<i32> = AsyncRuntime::new();
    let fake_id = uuid::Uuid::new_v4();
    runtime.remove_promise(fake_id).await;
    assert_eq!(runtime.active_count().await, 0);
}

// ── lib.rs tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_promise_create_and_resolve() {
    let (promise, resolver) = Promise::new();
    assert!(promise.is_pending().await);

    tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        resolver.resolve(100).await.unwrap();
    });

    let result = promise.await_result().await.unwrap();
    assert_eq!(result, 100);
}

#[tokio::test]
async fn test_promise_reject_and_check() {
    let (promise, resolver) = Promise::<i32>::new();
    resolver.reject("failure".to_string()).await.unwrap();
    assert!(promise.is_rejected().await);
    let err = promise.await_result().await.unwrap_err();
    assert!(matches!(err, PromiseError::Rejected(msg) if msg == "failure"));
}

#[tokio::test]
async fn test_promise_resolved_factory() {
    let promise = Promise::resolved("done".to_string());
    assert!(promise.is_resolved().await);
    assert!(!promise.is_pending().await);
    assert!(!promise.is_rejected().await);
    let val = promise.await_result().await.unwrap();
    assert_eq!(val, "done");
}

#[tokio::test]
async fn test_promise_rejected_factory() {
    let promise: Promise<i32> = Promise::rejected("err".to_string());
    assert!(promise.is_rejected().await);
    assert!(promise.await_result().await.is_err());
}

#[tokio::test]
async fn test_runtime_spawn_and_await() {
    let runtime = AsyncRuntime::new();
    assert_eq!(runtime.active_count().await, 0);

    let promise = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(5)).await;
                resolver.resolve(42).await.unwrap();
            })
        })
        .await;

    assert_eq!(runtime.active_count().await, 1);
    let val = promise.await_result().await.unwrap();
    assert_eq!(val, 42);
}

#[tokio::test]
async fn test_runtime_default() {
    let runtime: AsyncRuntime<i32> = AsyncRuntime::default();
    assert_eq!(runtime.active_count().await, 0);
}

#[tokio::test]
async fn test_promise_all_empty() {
    let result: Result<Vec<i32>, _> = promise_all(vec![]).await;
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[tokio::test]
async fn test_promise_all_mixed_timing() {
    let (p1, r1) = Promise::new();
    let (p2, r2) = Promise::new();

    tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        r1.resolve(1).await.unwrap();
    });
    tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        r2.resolve(2).await.unwrap();
    });

    let results = promise_all(vec![p1, p2]).await.unwrap();
    assert_eq!(results, vec![1, 2]);
}

#[tokio::test]
async fn test_promise_race_fastest_wins() {
    let (p1, r1) = Promise::new();
    let p2 = Promise::resolved(99);

    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        let _ = r1.resolve(1).await;
    });

    let result = promise_race(vec![p1, p2]).await.unwrap();
    assert_eq!(result, 99);
}

#[tokio::test]
async fn test_promise_race_empty_errors() {
    let result: Result<i32, _> = promise_race(vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_promise_state_transitions() {
    let (promise, resolver) = Promise::<String>::new();
    let state = promise.state().await;
    assert!(matches!(state, PromiseState::Pending));

    resolver.resolve("hello".to_string()).await.unwrap();
    let state = promise.state().await;
    assert!(matches!(state, PromiseState::Resolved(ref s) if s.as_ref() == "hello"));
}

#[tokio::test]
async fn test_promise_id_unique() {
    let (p1, _) = Promise::<i32>::new();
    let (p2, _) = Promise::<i32>::new();
    assert_ne!(p1.id(), p2.id());
}

#[tokio::test]
async fn test_promise_all_one_reject_aborts_remaining() {
    let p1 = Promise::resolved(1);
    let p2: Promise<i32> = Promise::rejected("fail".to_string());
    let p3 = Promise::resolved(3);

    let result = promise_all(vec![p1, p2, p3]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_runtime_multiple_spawns_and_get() {
    let runtime = AsyncRuntime::new();

    let p1 = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(10).await.unwrap();
            })
        })
        .await;

    let p2 = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(20).await.unwrap();
            })
        })
        .await;

    assert_eq!(runtime.active_count().await, 2);

    let retrieved1 = runtime.get_promise(p1.id()).await;
    assert!(retrieved1.is_some());

    let retrieved2 = runtime.get_promise(p2.id()).await;
    assert!(retrieved2.is_some());
}

#[tokio::test]
async fn test_promise_state_rejected_lib() {
    let (promise, resolver) = Promise::<i32>::new();
    resolver.reject("fail".to_string()).await.unwrap();
    let state = promise.state().await;
    assert!(matches!(state, PromiseState::Rejected(ref s) if s == "fail"));
}

#[tokio::test]
async fn test_promise_all_preserves_order() {
    let p1 = Promise::resolved(10);
    let p2 = Promise::resolved(20);
    let p3 = Promise::resolved(30);

    let results = promise_all(vec![p1, p2, p3]).await.unwrap();
    assert_eq!(results, vec![10, 20, 30]);
}

#[tokio::test]
async fn test_runtime_cancel_all_clears() {
    let runtime = AsyncRuntime::new();
    let _p = runtime
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_secs(60)).await;
                let _ = resolver.resolve(1).await;
            })
        })
        .await;
    assert_eq!(runtime.active_count().await, 1);
    runtime.cancel_all().await;
    assert_eq!(runtime.active_count().await, 0);
}
