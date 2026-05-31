use hudhudscript_runtime::raii::{AgentSession, ConnectionHandle, Disposable, ScopedResource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

struct TrackingResource {
    name: String,
    disposed: Arc<AtomicBool>,
}

impl Disposable for TrackingResource {
    fn dispose(&self) {
        self.disposed.store(true, Ordering::SeqCst);
    }
    fn resource_name(&self) -> &str {
        &self.name
    }
}

#[test]
fn scoped_resource_disposes_on_drop() {
    let flag = Arc::new(AtomicBool::new(false));
    {
        let resource = TrackingResource {
            name: "test".to_string(),
            disposed: flag.clone(),
        };
        let _guard = ScopedResource::new(resource);
        assert!(!flag.load(Ordering::SeqCst));
    } // _guard dropped here
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn scoped_resource_dispose_now() {
    let flag = Arc::new(AtomicBool::new(false));
    let resource = TrackingResource {
        name: "test".to_string(),
        disposed: flag.clone(),
    };
    let mut guard = ScopedResource::new(resource);
    guard.dispose_now();
    assert!(flag.load(Ordering::SeqCst));
}

#[tokio::test]
async fn agent_session_disposes_resources_lifo() {
    let session = AgentSession::new("test-agent");

    let order: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

    // Register cleanup hooks — they must fire in LIFO order.
    let o1 = order.clone();
    session
        .register_cleanup("hook-1", move || {
            // Use try_lock so we don't need async in the hook.
            if let Ok(mut v) = o1.try_lock() {
                v.push(1);
            }
        })
        .await;

    let o2 = order.clone();
    session
        .register_cleanup("hook-2", move || {
            if let Ok(mut v) = o2.try_lock() {
                v.push(2);
            }
        })
        .await;

    session.dispose().await;

    let result = order.lock().await;
    // LIFO: hook-2 fires before hook-1.
    assert_eq!(*result, vec![2, 1]);
}

#[tokio::test]
async fn agent_session_idempotent_dispose() {
    let session = AgentSession::new("agent-idempotent");
    session.dispose().await;
    session.dispose().await; // second call must not panic
    assert!(session.is_disposed());
}

#[tokio::test]
async fn connection_handle_disposes_once() {
    let count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let c = count.clone();
    let handle = Arc::new(ConnectionHandle::new("db", move || {
        c.fetch_add(1, Ordering::SeqCst);
    }));

    handle.dispose();
    handle.dispose(); // second call must be a no-op
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
