use crate::{DynamicData, DynamicKind, DynamicObject, Repr, Value16};
use std::mem;

// Mark-sweep machinery lives in `gc_trace`; re-exported here to keep the
// historical `gc::` paths (used across the workspace) stable.
pub use crate::gc_trace::{collect, drain_gray, is_marked, trace_children, trace_value};

const ENV_GC_STRESS: &str = "HUDHUD_GC_STRESS";
const ENV_GC_VERIFY: &str = "HUDHUD_GC_VERIFY";
pub const DEFAULT_GC_MIN_OBJECTS: usize = 1024;
pub const DEFAULT_GC_GROWTH: usize = 2;
const MIN_GC_GROWTH: usize = 1;

#[derive(Default)]
pub struct GcHeap {
    pub(super) objects: Vec<*mut DynamicObject>,
    pub(super) bytes_alloc: usize,
    pub(super) next_gc: usize,
    pub(super) collections: u64,
    pub(super) total_freed: u64,
    pub(super) last_pause_micros: u64,
    pub(super) pause_micros_total: u64,
    pub(super) pause_micros_max: u64,
    #[cfg(feature = "telemetry")]
    alloc_count_by_kind: [u64; 17],
}

impl Drop for GcHeap {
    fn drop(&mut self) {
        for ptr in self.objects.drain(..) {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
        // Drain BigInt free-list if thread-local still accessible.
        let _ = BIGINT_FREE_LIST.try_with(|fl| {
            for ptr in fl.borrow_mut().drain(..) {
                unsafe { drop(Box::from_raw(ptr)) };
            }
        });
    }
}

std::thread_local! {
    pub static CURRENT_HEAP: std::cell::Cell<*mut GcHeap> = std::cell::Cell::new(std::ptr::null_mut());
    pub static FALLBACK_HEAP: std::cell::RefCell<GcHeap> = std::cell::RefCell::new(GcHeap {
        objects: Vec::new(),
        bytes_alloc: 0,
        next_gc: 0,
        collections: 0,
        total_freed: 0,
        last_pause_micros: 0,
        pause_micros_total: 0,
        pause_micros_max: 0,
        #[cfg(feature = "telemetry")]
        alloc_count_by_kind: [0; 17],
    });
}

#[inline]
pub fn with_heap<R>(f: impl FnOnce(&mut GcHeap) -> R) -> R {
    CURRENT_HEAP.with(|c| {
        let ptr = c.get();
        if ptr.is_null() {
            FALLBACK_HEAP.with(|fallback| f(&mut *fallback.borrow_mut()))
        } else {
            f(unsafe { &mut *ptr })
        }
    })
}

std::thread_local! {
    /// P2: alloc eşik aşımında SADECE bu bayrağı kaldırır; collect'i yalnız
    /// VM dispatch safepoint'i (instruction sınırı) çağırır.
    /// INVARIANT: alloc() HİÇBİR KOŞULDA collect tetiklemez (yerli koddaki
    /// Value16 geçicileri trace edilemez → alloc-içi collect = use-after-free).
    pub(super) static GC_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// HUDHUD_GC_STRESS=1 env önbelleği (her safepoint'te env okumamak için).
    static GC_STRESS: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    /// P6: BigInt free-list — reclaimed DynamicObject slots, reused without heap alloc.
    pub(super) static BIGINT_FREE_LIST: std::cell::RefCell<Vec<*mut DynamicObject>> = std::cell::RefCell::new(Vec::new());
}

pub(super) const BIGINT_FREE_LIST_MAX: usize = 1024;

pub trait GcRootSource {
    fn mark_roots(&self);
}

std::thread_local! {
    /// ISSUE-1: GC tuning values cached in thread-local storage — NO env reads in hot path.
    /// set_gc_tuning() is called once at startup (from CLI config, if provided).
    static GC_MIN: std::cell::Cell<usize> = const { std::cell::Cell::new(DEFAULT_GC_MIN_OBJECTS) };
    static GC_GROWTH: std::cell::Cell<usize> = const { std::cell::Cell::new(DEFAULT_GC_GROWTH) };
}

/// Called once at startup (before any VM execution) to set GC tuning from config.
/// If never called, defaults are used — no env reads in the hot path.
pub fn set_gc_tuning(min: usize, growth: usize) {
    GC_MIN.with(|c| c.set(min.max(1)));
    GC_GROWTH.with(|c| c.set(growth.max(MIN_GC_GROWTH)));
}

pub(super) fn gc_min_threshold() -> usize {
    GC_MIN.with(|c| c.get())
}

pub(super) fn gc_growth_factor() -> usize {
    GC_GROWTH.with(|c| c.get())
}

fn gc_stress_enabled() -> bool {
    GC_STRESS.with(|cache| {
        if let Some(enabled) = cache.get() {
            return enabled;
        }
        let enabled = std::env::var(ENV_GC_STRESS)
            .map(|v| v == "1")
            .unwrap_or(false);
        cache.set(Some(enabled));
        enabled
    })
}

std::thread_local! {
    /// ISSUE-1: GC verify flag cached — env read once, reused.
    static GC_VERIFY: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

pub(super) fn gc_verify_enabled() -> bool {
    GC_VERIFY.with(|cache| {
        if let Some(enabled) = cache.get() {
            return enabled;
        }
        let enabled = std::env::var(ENV_GC_VERIFY)
            .map(|v| v == "1")
            .unwrap_or(false);
        cache.set(Some(enabled));
        enabled
    })
}

/// Safepoint (VM dispatch) bunu sorar: GC zamanı geldi mi?
#[inline]
pub fn safepoint_due() -> bool {
    gc_stress_enabled() || GC_PENDING.with(|p| p.get())
}

/// Central dynamic allocation entry point.
///
/// Dynamic objects are owned by the GC heap and released by manual collection
/// or heap teardown.
#[inline]
pub fn alloc(kind: DynamicKind, data: DynamicData) -> Value16 {
    // P6: BigInt free-list fast path — reuse a reclaimed slot.
    if matches!(kind, DynamicKind::BigInt) {
        if let Some(ptr) = BIGINT_FREE_LIST.with(|fl| fl.borrow_mut().pop()) {
            let obj = unsafe { &mut *ptr };
            obj.marked.set(false);
            obj.data = data;
            with_heap(|heap| {
                heap.objects.push(ptr);
                heap.bytes_alloc = heap
                    .bytes_alloc
                    .saturating_add(mem::size_of_val(unsafe { &*ptr }));
                #[cfg(feature = "telemetry")]
                {
                    heap.alloc_count_by_kind[kind as usize] += 1;
                }
            });
            return Value16(Repr::new_dynamic(ptr as *const ()));
        }
    }
    let obj = Box::new(DynamicObject {
        kind,
        marked: std::cell::Cell::new(false),
        data,
    });
    let bytes = mem::size_of_val(obj.as_ref());
    let ptr = Box::into_raw(obj);

    with_heap(|heap| {
        heap.objects.push(ptr);
        heap.bytes_alloc = heap.bytes_alloc.saturating_add(bytes);
        #[cfg(feature = "telemetry")]
        {
            heap.alloc_count_by_kind[kind as usize] += 1;
        }
        // P2: eşik obje SAYISI bazlı (next_gc, collect'te canlı*GROWTH olarak güncellenir).
        if heap.objects.len() >= heap.next_gc.max(gc_min_threshold()) {
            GC_PENDING.with(|p| p.set(true));
        }
    });

    Value16(Repr::new_dynamic(ptr as *const ()))
}

#[inline]
pub fn heap_object_count() -> usize {
    with_heap(|heap| heap.objects.len())
}

#[inline]
pub fn bytes_allocated() -> usize {
    with_heap(|heap| heap.bytes_alloc)
}

#[inline]
pub fn next_gc_threshold() -> usize {
    with_heap(|heap| heap.next_gc)
}

// ── P8/W3: GC telemetrisi ──

#[derive(Debug, Clone, Copy)]
pub struct GcStats {
    pub collections: u64,
    pub live_objects: usize,
    pub total_freed: u64,
    pub last_pause_micros: u64,
    pub next_gc: usize,
    pub pinned_count: usize,
}

pub fn stats() -> GcStats {
    let pinned_count = crate::gc_pin::pinned_count();
    with_heap(|heap| {
        let h = heap;
        GcStats {
            collections: h.collections,
            live_objects: h.objects.len(),
            total_freed: h.total_freed,
            last_pause_micros: h.last_pause_micros,
            next_gc: h.next_gc,
            pinned_count,
        }
    })
}

#[cfg(feature = "telemetry")]
pub fn take_telemetry_alloc_counts(out: &mut [u64]) {
    with_heap(|heap| {
        out.copy_from_slice(&heap.alloc_count_by_kind);
        heap.alloc_count_by_kind.fill(0);
    });
}

#[cfg(feature = "telemetry")]
pub fn take_telemetry_gc_stats(
    cycle_count: &mut u64,
    mark_count: &mut u64,
    sweep_count: &mut u64,
    pause_ns_total: &mut u64,
    pause_ns_max: &mut u64,
    heap_bytes: &mut u64,
) {
    with_heap(|heap| {
        *cycle_count = heap.collections;
        *mark_count = 0; // Not tracked
        *sweep_count = heap.total_freed;
        *pause_ns_total = heap.pause_micros_total * 1000;
        *pause_ns_max = heap.pause_micros_max * 1000;
        *heap_bytes = (heap.objects.len() * std::mem::size_of::<*mut DynamicObject>()) as u64;
    });
}
