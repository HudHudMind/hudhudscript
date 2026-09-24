//! Process-global bridge state shared between the VM owner thread and the
//! Dart/UI thread.
//!
//! Why global (not fields on `HudVM`): promise settlement and callback
//! completion arrive from a DIFFERENT thread while the worker thread holds
//! `&mut VM` inside `execute`. Rust aliasing rules forbid touching `HudVM`
//! fields then, so cross-thread state lives here behind a `Mutex`, keyed by
//! an opaque `bridge_id` handed out at VM creation.
//!
//! Promise sender pool: `register_method` closures have no `&mut VM`, so
//! they cannot mint registry ids at call time. Ids are pre-minted in bulk
//! on the worker thread (where `&mut VM` exists) into the VM's promise
//! registry; senders wait here. A callback pops one and returns
//! `AsyncPending(id)` — the VM's Await resolves it through the registry the
//! moment `hud_callback_complete` fires the sender.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;

use hudhudscript_bytecode::Value16;
use hudhudscript_vm::VM;

/// Free-pool target: ids pre-minted per bridge. Concurrent host calls
/// beyond this count fail with a "pool exhausted" runtime error until the
/// pool is refilled at the next worker-thread FFI entry point.
const PROMISE_POOL_TARGET: usize = 64;

pub struct PendingHostCall {
    /// "module.method" the Dart side resolves to a registered handler.
    pub key: String,
    /// Script-supplied arguments; `None` once fetched by `hud_callback_take`.
    pub args: Option<Vec<Value16>>,
    /// Async mode: id of the promise awaiting `hud_callback_complete`.
    pub promise_id: String,
}

pub struct BridgeState {
    /// `extern "C" fn(request_id)` address (Dart `NativeCallable.listener`).
    pub dart_cb: Option<usize>,
    /// Pre-minted, not-yet-used promise ids.
    pub free_ids: Vec<String>,
    /// All live senders for ids minted into the VM's registry.
    pub promise_txs: HashMap<String, Sender<Result<Value16, String>>>,
    /// In-flight host callback requests.
    pub pending_calls: HashMap<i64, PendingHostCall>,
}

impl BridgeState {
    fn new() -> Self {
        Self {
            dart_cb: None,
            free_ids: Vec::new(),
            promise_txs: HashMap::new(),
            pending_calls: HashMap::new(),
        }
    }
}

static BRIDGE_SEQ: AtomicI64 = AtomicI64::new(1);
static REQUEST_SEQ: AtomicI64 = AtomicI64::new(1);
static BRIDGES: Mutex<Option<HashMap<i64, BridgeState>>> = Mutex::new(None);

pub fn register_bridge() -> i64 {
    let id = BRIDGE_SEQ.fetch_add(1, Ordering::Relaxed);
    let mut bridges = BRIDGES.lock().unwrap();
    bridges.get_or_insert_with(HashMap::new).insert(id, BridgeState::new());
    id
}

pub fn drop_bridge(bridge_id: i64) {
    let mut bridges = BRIDGES.lock().unwrap();
    if let Some(map) = bridges.as_mut() {
        map.remove(&bridge_id);
    }
}

pub fn next_request_id() -> i64 {
    REQUEST_SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Run `f` against the bridge state. Returns `None` if the bridge id is
/// unknown (VM freed) so callers can surface a clean error.
pub fn with_bridge<R>(
    bridge_id: i64,
    f: impl FnOnce(&mut BridgeState) -> R,
) -> Option<R> {
    let mut bridges = BRIDGES.lock().unwrap();
    bridges.as_mut()?.get_mut(&bridge_id).map(f)
}

/// Pre-mint promise channels into the VM's registry until the free pool
/// reaches its target. Must run on the VM owner thread (`&mut VM`).
pub fn mint_promise_pool(vm: &mut VM, bridge_id: i64) {
    let need = with_bridge(bridge_id, |state| {
        PROMISE_POOL_TARGET.saturating_sub(state.free_ids.len())
    });
    let Some(need) = need else { return };
    let mut minted: Vec<(String, Sender<Result<Value16, String>>)> = Vec::new();
    for _ in 0..need {
        let (tx, rx) = mpsc::channel();
        let id = vm.register_promise_owned(rx);
        minted.push((id, tx));
    }
    with_bridge(bridge_id, |state| {
        for (id, tx) in minted {
            state.promise_txs.insert(id.clone(), tx);
            state.free_ids.push(id);
        }
    });
}

/// Pop a pre-minted promise id for an async host callback.
pub fn take_free_promise(state: &mut BridgeState) -> Option<String> {
    state.free_ids.pop()
}

/// Settle a promise by firing its stored sender. Thread-safe: called from
/// the Dart/UI thread inside `hud_callback_complete` /
/// `hud_vm_resolve_promise`. Returns `false` when the id is unknown or was
/// already settled (senders are single-use: removed on first settle).
pub fn settle_promise(bridge_id: i64, promise_id: &str, result: Result<Value16, String>) -> bool {
    with_bridge(bridge_id, |state| {
        if let Some(tx) = state.promise_txs.remove(promise_id) {
            let _ = tx.send(result);
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}
