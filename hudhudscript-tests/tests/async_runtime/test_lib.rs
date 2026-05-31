//! Public API tests for hudhudscript-async
//! Covers: promise.rs (Promise, PromiseResolver, PromiseState, PromiseError),
//!         runtime.rs (AsyncRuntime, AsyncRuntimeError),
//!         combinators.rs (promise_all, promise_race).

use hudhudscript_async::{
    promise_all, promise_race, AsyncRuntime, Promise, PromiseError, PromiseState,
};
use tokio::time::{sleep, Duration};

// ── PromiseState enum ───────────────────────────────────────────────

#[tokio::test]
async fn promise_state_is_pending_immediately_after_new() {
    let (promise, _resolver) = Promise::<i32>::new();
    assert!(promise.is_pending().await);
    assert!(!promise.is_resolved().await);
    assert!(!promise.is_rejected().await);
}

#[tokio::test]
async fn promise_state_transitions_to_resolved() {
    let (promise, resolver) = Promise::<i32>::new();
    resolver.resolve(42).await.unwrap();
    assert!(!promise.is_pending().await);
    assert!(promise.is_resolved().await);
    assert!(!promise.is_rejected().await);
}

#[tokio::test]
async fn promise_state_transitions_to_rejected() {
    let (promise, resolver) = Promise::<i32>::new();
    resolver.reject("fail".into()).await.unwrap();
    assert!(!promise.is_pending().await);
    assert!(!promise.is_resolved().await);
    assert!(promise.is_rejected().await);
}

#[tokio::test]
async fn promise_state_enum_resolved_variant() {
    let p = Promise::resolved(99i32);
    let state = p.state().await;
    assert!(matches!(state, PromiseState::Resolved(ref v) if **v == 99));
}

#[tokio::test]
async fn promise_state_enum_rejected_variant() {
    let p: Promise<i32> = Promise::rejected("boom".into());
    let state = p.state().await;
    assert!(matches!(state, PromiseState::Rejected(ref msg) if msg == "boom"));
}

#[tokio::test]
async fn promise_state_enum_pending_variant() {
    let (promise, _resolver) = Promise::<String>::new();
    let state = promise.state().await;
    assert!(matches!(state, PromiseState::Pending));
}

// ── Promise::resolved factory ───────────────────────────────────────

#[tokio::test]
async fn promise_resolved_await_returns_value() {
    let p = Promise::resolved(42i32);
    assert_eq!(p.await_result().await.unwrap(), 42);
}

#[tokio::test]
async fn promise_resolved_with_string() {
    let p = Promise::resolved("hello".to_string());
    assert_eq!(p.await_result().await.unwrap(), "hello");
}

#[tokio::test]
async fn promise_resolved_is_resolved_not_pending() {
    let p = Promise::resolved(0i32);
    assert!(p.is_resolved().await);
    assert!(!p.is_pending().await);
}

// ── Promise::rejected factory ───────────────────────────────────────

#[tokio::test]
async fn promise_rejected_await_returns_error() {
    let p: Promise<i32> = Promise::rejected("bad".into());
    let err = p.await_result().await.unwrap_err();
    assert!(matches!(err, PromiseError::Rejected(ref m) if m == "bad"));
}

#[tokio::test]
async fn promise_rejected_with_empty_string() {
    let p: Promise<i32> = Promise::rejected("".into());
    assert!(p.await_result().await.is_err());
}

#[tokio::test]
async fn promise_rejected_is_rejected_not_pending() {
    let p: Promise<i32> = Promise::rejected("x".into());
    assert!(p.is_rejected().await);
    assert!(!p.is_pending().await);
}

// ── Promise ID ─────────────────────────────────────────────────────

#[tokio::test]
async fn promise_id_unique_per_instance() {
    let (p1, _r1) = Promise::<i32>::new();
    let (p2, _r2) = Promise::<i32>::new();
    assert_ne!(p1.id(), p2.id());
}

#[tokio::test]
async fn promise_and_resolver_share_same_id() {
    let (promise, resolver) = Promise::<i32>::new();
    assert_eq!(promise.id(), resolver.id());
}

#[tokio::test]
async fn promise_debug_contains_struct_name() {
    let (p, r) = Promise::<i32>::new();
    assert!(format!("{:?}", p).contains("Promise"));
    assert!(format!("{:?}", r).contains("PromiseResolver"));
}

// ── PromiseResolver::resolve ────────────────────────────────────────

#[tokio::test]
async fn resolver_resolve_wakes_awaiter() {
    let (promise, resolver) = Promise::<i32>::new();
    tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        resolver.resolve(7).await.unwrap();
    });
    assert_eq!(promise.await_result().await.unwrap(), 7);
}

#[tokio::test]
async fn resolver_resolve_returns_ok() {
    let (_p, resolver) = Promise::<u64>::new();
    assert!(resolver.resolve(1000).await.is_ok());
}

// ── PromiseResolver::reject ─────────────────────────────────────────

#[tokio::test]
async fn resolver_reject_wakes_awaiter_with_error() {
    let (promise, resolver) = Promise::<i32>::new();
    tokio::spawn(async move {
        sleep(Duration::from_millis(5)).await;
        resolver.reject("reason".into()).await.unwrap();
    });
    let e = promise.await_result().await.unwrap_err();
    assert!(matches!(e, PromiseError::Rejected(ref m) if m == "reason"));
}

#[tokio::test]
async fn resolver_reject_returns_ok() {
    let (_p, resolver) = Promise::<u64>::new();
    assert!(resolver.reject("err".into()).await.is_ok());
}

// ── Dropped resolver ────────────────────────────────────────────────

#[tokio::test]
async fn await_result_receiver_dropped_when_resolver_dropped() {
    let (promise, resolver) = Promise::<i32>::new();
    drop(resolver);
    let e = promise.await_result().await.unwrap_err();
    assert!(matches!(e, PromiseError::ReceiverDropped));
}

// ── PromiseError display ─────────────────────────────────────────────

#[test]
fn promise_error_already_resolved_display() {
    let e = PromiseError::AlreadyResolved;
    assert!(e.to_string().contains("Promise already resolved"));
}

#[test]
fn promise_error_already_rejected_display() {
    let e = PromiseError::AlreadyRejected;
    assert!(e.to_string().contains("Promise already rejected"));
}

#[test]
fn promise_error_rejected_display_with_message() {
    let e = PromiseError::Rejected("oops".into());
    assert!(e.to_string().contains("Promise was rejected: oops"));
}

#[test]
fn promise_error_receiver_dropped_display() {
    let e = PromiseError::ReceiverDropped;
    assert!(e.to_string().contains("Promise receiver dropped"));
}

// ── AsyncRuntime construction ───────────────────────────────────────

#[tokio::test]
async fn runtime_new_starts_with_zero_active() {
    let rt = AsyncRuntime::<i32>::new();
    assert_eq!(rt.active_count().await, 0);
}

#[tokio::test]
async fn runtime_default_starts_with_zero_active() {
    let rt = AsyncRuntime::<i32>::default();
    assert_eq!(rt.active_count().await, 0);
}

// ── AsyncRuntime::spawn_task ────────────────────────────────────────

#[tokio::test]
async fn runtime_spawn_task_resolves_to_correct_value() {
    let rt = AsyncRuntime::new();
    let promise = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(5)).await;
                resolver.resolve(99i32).await.unwrap();
            })
        })
        .await;
    assert_eq!(promise.await_result().await.unwrap(), 99);
}

#[tokio::test]
async fn runtime_spawn_increments_active_count() {
    let rt = AsyncRuntime::<i32>::new();
    let _p = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(200)).await;
                resolver.resolve(1).await.unwrap();
            })
        })
        .await;
    assert_eq!(rt.active_count().await, 1);
}

#[tokio::test]
async fn runtime_two_spawns_give_active_count_two() {
    let rt = AsyncRuntime::<i32>::new();
    let _p1 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(200)).await;
                resolver.resolve(10).await.unwrap();
            })
        })
        .await;
    let _p2 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(200)).await;
                resolver.resolve(20).await.unwrap();
            })
        })
        .await;
    assert_eq!(rt.active_count().await, 2);
}

// ── AsyncRuntime::get_promise ───────────────────────────────────────

#[tokio::test]
async fn runtime_get_promise_returns_some_after_spawn() {
    let rt = AsyncRuntime::<i32>::new();
    let p = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(200)).await;
                resolver.resolve(5).await.unwrap();
            })
        })
        .await;
    let id = p.id();
    assert!(rt.get_promise(id).await.is_some());
}

#[tokio::test]
async fn runtime_get_promise_returns_none_for_unknown_id() {
    let rt = AsyncRuntime::<i32>::new();
    assert!(rt.get_promise(uuid::Uuid::new_v4()).await.is_none());
}

// ── AsyncRuntime::remove_promise ────────────────────────────────────

#[tokio::test]
async fn runtime_remove_promise_makes_it_unfindable() {
    let rt = AsyncRuntime::<i32>::new();
    let p = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(3).await.unwrap();
            })
        })
        .await;
    let id = p.id();
    rt.remove_promise(id).await;
    assert!(rt.get_promise(id).await.is_none());
    assert_eq!(rt.active_count().await, 0);
}

#[tokio::test]
async fn runtime_remove_nonexistent_id_does_not_panic() {
    let rt = AsyncRuntime::<i32>::new();
    rt.remove_promise(uuid::Uuid::new_v4()).await;
    assert_eq!(rt.active_count().await, 0);
}

// ── AsyncRuntime::cancel_all ────────────────────────────────────────

#[tokio::test]
async fn runtime_cancel_all_clears_all_promises() {
    let rt = AsyncRuntime::<i32>::new();
    let _p1 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                resolver.resolve(1).await.unwrap();
            })
        })
        .await;
    let _p2 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                resolver.resolve(2).await.unwrap();
            })
        })
        .await;
    assert_eq!(rt.active_count().await, 2);
    rt.cancel_all().await;
    assert_eq!(rt.active_count().await, 0);
}

#[tokio::test]
async fn runtime_cancel_all_on_empty_runtime_is_safe() {
    let rt = AsyncRuntime::<i32>::new();
    rt.cancel_all().await;
    assert_eq!(rt.active_count().await, 0);
}

// ── AsyncRuntime multiple tasks resolve independently ───────────────

#[tokio::test]
async fn runtime_multiple_tasks_all_resolve_correctly() {
    let rt = AsyncRuntime::<i32>::new();
    let p1 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(10).await.unwrap();
            })
        })
        .await;
    let p2 = rt
        .spawn_task(|resolver| {
            tokio::spawn(async move {
                resolver.resolve(20).await.unwrap();
            })
        })
        .await;
    let (r1, r2) = tokio::join!(p1.await_result(), p2.await_result());
    assert_eq!(r1.unwrap(), 10);
    assert_eq!(r2.unwrap(), 20);
}

// ── AsyncRuntimeError display ───────────────────────────────────────

#[test]
fn async_runtime_error_promise_not_found_display() {
    use hudhudscript_async::runtime::AsyncRuntimeError;
    let id = uuid::Uuid::nil();
    let e = AsyncRuntimeError::PromiseNotFound(id);
    assert!(e.to_string().contains("Promise not found"));
}

#[test]
fn async_runtime_error_task_spawn_failed_display() {
    use hudhudscript_async::runtime::AsyncRuntimeError;
    let e = AsyncRuntimeError::TaskSpawnFailed("reason".into());
    assert!(e.to_string().contains("Task spawn failed: reason"));
}

#[test]
fn async_runtime_error_runtime_error_display() {
    use hudhudscript_async::runtime::AsyncRuntimeError;
    let e = AsyncRuntimeError::RuntimeError("crash".into());
    assert!(e.to_string().contains("Runtime error: crash"));
}

// ── promise_all combinator ──────────────────────────────────────────

#[tokio::test]
async fn promise_all_empty_vec_returns_empty_ok() {
    let result: Result<Vec<i32>, _> = promise_all(vec![]).await;
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[tokio::test]
async fn promise_all_preserves_input_order() {
    let p1 = Promise::resolved(10);
    let p2 = Promise::resolved(20);
    let p3 = Promise::resolved(30);
    let results = promise_all(vec![p1, p2, p3]).await.unwrap();
    assert_eq!(results, vec![10, 20, 30]);
}

#[tokio::test]
async fn promise_all_single_element() {
    let p = Promise::resolved(42);
    let results = promise_all(vec![p]).await.unwrap();
    assert_eq!(results, vec![42]);
}

#[tokio::test]
async fn promise_all_one_rejected_returns_error() {
    let p1 = Promise::resolved(1);
    let p2: Promise<i32> = Promise::rejected("fail".into());
    let p3 = Promise::resolved(3);
    let result = promise_all(vec![p1, p2, p3]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn promise_all_async_tasks_maintain_order() {
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

// ── promise_race combinator ─────────────────────────────────────────

#[tokio::test]
async fn promise_race_empty_vec_returns_error() {
    let result: Result<i32, _> = promise_race(vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn promise_race_single_element_returns_that_value() {
    let p = Promise::resolved(7);
    let result = promise_race(vec![p]).await.unwrap();
    assert_eq!(result, 7);
}

#[tokio::test]
async fn promise_race_immediate_resolved_wins() {
    let p1 = Promise::resolved(99);
    let (p2, _r2) = Promise::<i32>::new();
    let result = promise_race(vec![p1, p2]).await.unwrap();
    assert_eq!(result, 99);
}

#[tokio::test]
async fn promise_race_rejected_first_returns_error() {
    let p1: Promise<i32> = Promise::rejected("fast fail".into());
    let (p2, _r2) = Promise::<i32>::new();
    let result = promise_race(vec![p1, p2]).await;
    assert!(result.is_err());
}
