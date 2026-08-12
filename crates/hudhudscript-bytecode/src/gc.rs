use crate::{DynamicData, DynamicKind, DynamicObject, PromiseState16, Repr, Value16};
use std::{cell::RefCell, mem};

const ENV_GC_STRESS: &str = "HUDHUD_GC_STRESS";
const ENV_GC_VERIFY: &str = "HUDHUD_GC_VERIFY";
pub const DEFAULT_GC_MIN_OBJECTS: usize = 1024;
pub const DEFAULT_GC_GROWTH: usize = 2;
const MIN_GC_GROWTH: usize = 1;

#[derive(Default)]
pub struct GcHeap {
    objects: Vec<*mut DynamicObject>,
    bytes_alloc: usize,
    next_gc: usize,
    collections: u64,
    total_freed: u64,
    last_pause_micros: u64,
    pause_micros_total: u64,
    pause_micros_max: u64,
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
            FALLBACK_HEAP.with(|fallback| {
                f(&mut *fallback.borrow_mut())
            })
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
    static GC_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// HUDHUD_GC_STRESS=1 env önbelleği (her safepoint'te env okumamak için).
    static GC_STRESS: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    /// P6: BigInt free-list — reclaimed DynamicObject slots, reused without heap alloc.
    static BIGINT_FREE_LIST: std::cell::RefCell<Vec<*mut DynamicObject>> = std::cell::RefCell::new(Vec::new());
}

const BIGINT_FREE_LIST_MAX: usize = 1024;

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

fn gc_min_threshold() -> usize {
    GC_MIN.with(|c| c.get())
}

fn gc_growth_factor() -> usize {
    GC_GROWTH.with(|c| c.get())
}

fn gc_stress_enabled() -> bool {
    GC_STRESS.with(|cache| {
        if let Some(enabled) = cache.get() {
            return enabled;
        }
        let enabled = std::env::var(ENV_GC_STRESS).map(|v| v == "1").unwrap_or(false);
        cache.set(Some(enabled));
        enabled
    })
}

std::thread_local! {
    /// ISSUE-1: GC verify flag cached — env read once, reused.
    static GC_VERIFY: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

fn gc_verify_enabled() -> bool {
    GC_VERIFY.with(|cache| {
        if let Some(enabled) = cache.get() {
            return enabled;
        }
        let enabled = std::env::var(ENV_GC_VERIFY).map(|v| v == "1").unwrap_or(false);
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
                heap.bytes_alloc = heap.bytes_alloc.saturating_add(mem::size_of_val(unsafe { &*ptr }));
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

#[inline]
pub fn trace_value(value: Value16, gray: &mut Vec<*mut DynamicObject>) {
    if !value.is_dynamic() {
        return;
    }

    let Some(ptr) = value.0.as_ptr() else {
        return;
    };
    let obj = unsafe { &*(ptr as *const DynamicObject) };
    if obj.marked.get() {
        return;
    }

    obj.marked.set(true);
    gray.push(ptr as *mut DynamicObject);
}

#[inline]
pub fn trace_children(obj: &DynamicObject, gray: &mut Vec<*mut DynamicObject>) {
    match &obj.data {
        DynamicData::String(_) => {}
        DynamicData::Array(items) | DynamicData::Set(items) => {
            for item in items {
                trace_value(*item, gray);
            }
        }
        DynamicData::Object(map) => {
            for value in map.values() {
                trace_value(*value, gray);
            }
        }
        DynamicData::Function(function) => {
            for capture in function.captures.values() {
                trace_value(*capture.read(), gray);
            }
        }
        DynamicData::Instance(instance) => {
            for value in instance.fields.values() {
                trace_value(*value, gray);
            }
            trace_value(instance.class, gray);
        }
        DynamicData::Promise(promise) => {
            if let PromiseState16::Resolved(value) = promise {
                trace_value(*value.as_ref(), gray);
            }
        }
        DynamicData::Class(class) => {
            for value in class.methods.values() {
                trace_value(*value, gray);
            }
            for value in class.fields.values() {
                trace_value(*value, gray);
            }
            if let Some(parent) = class.parent {
                trace_value(parent, gray);
            }
            for value in class.vtable.values() {
                trace_value(*value, gray);
            }
        }
        DynamicData::Data(data) => {
            for value in data.fields.values() {
                trace_value(*value, gray);
            }
        }
        DynamicData::Map(pairs) => {
            for (key, value) in pairs {
                trace_value(*key, gray);
                trace_value(*value, gray);
            }
        }
        DynamicData::Generator(state) => {
            let state = state.lock();
            for value in &state.pending {
                trace_value(*value, gray);
            }
            for value in &state.buffered {
                trace_value(*value, gray);
            }
        }
        DynamicData::Tool(_) | DynamicData::Resource(_) | DynamicData::BigInt(_) => {}
        DynamicData::Option(option) => {
            if let Some(value) = option.as_ref() {
                trace_value(*value.as_ref(), gray);
            }
        }
        DynamicData::Result(result) => {
            if let Ok(value) = result.as_ref() {
                trace_value(*value.as_ref(), gray);
            }
        }
    }
}

#[inline]
pub fn drain_gray(gray: &mut Vec<*mut DynamicObject>) {
    while let Some(ptr) = gray.pop() {
        unsafe { trace_children(&*ptr, gray) };
    }
}

#[inline]
pub fn collect(roots: &impl GcRootSource) {
    let start = std::time::Instant::now();
    GC_PENDING.with(|p| p.set(false));
    roots.mark_roots();
    // V2-C2: Pinned values survive even when called directly (standalone gc::collect)
    let mut gray = Vec::new();
    crate::gc_pin::trace_pinned(&mut gray);
    crate::gc::drain_gray(&mut gray);
    // Faz 1: borrow ALTINDA sadece ayrıştır — burada Drop ÇALIŞTIRILMAZ
    // (Drop, heap'e erişirse RefCell çifte-borrow paniği olur).
    let dead: Vec<*mut DynamicObject> = with_heap(|heap| {
        
        let mut dead = Vec::new();
        heap.objects.retain(|ptr| {
            let obj = unsafe { &**ptr };
            if obj.marked.get() {
                obj.marked.set(false);
                true
            } else {
                dead.push(*ptr);
                false
            }
        });
        heap.bytes_alloc = 0;
        heap.next_gc = heap
            .objects
            .len()
            .saturating_mul(gc_growth_factor())
            .max(gc_min_threshold());
        // P8: Telemetri — Faz 1 borrow'u içinde güncelle.
        heap.collections += 1;
        heap.total_freed += dead.len() as u64;
        dead
    });
    // Faz 2: borrow BIRAKILDI. BigInt'leri free-list'e koy.
    for ptr in dead {
        let is_bigint = unsafe { matches!((*ptr).kind, DynamicKind::BigInt) };
        if is_bigint {
            let added = BIGINT_FREE_LIST.with(|fl| {
                let mut fl = fl.borrow_mut();
                if fl.len() < BIGINT_FREE_LIST_MAX {
                    fl.push(ptr);
                    true
                } else {
                    false
                }
            });
            if added { continue; }
        }
        unsafe { drop(Box::from_raw(ptr)) };
    }
    // P8: Pause süresini yaz (ayrı borrow).
    let elapsed = start.elapsed().as_micros() as u64;
    with_heap(|heap| {
        heap.last_pause_micros = elapsed;
        heap.pause_micros_total += elapsed;
        if elapsed > heap.pause_micros_max {
            heap.pause_micros_max = elapsed;
        }
    });
    // P7/W2: Debug doğrulaması — sweep sonrası canlı objelerde mark kalmamalı.
    if gc_verify_enabled() {
        with_heap(|heap| {
            let heap = heap;
            for ptr in &heap.objects {
                debug_assert!(
                    !unsafe { &**ptr }.marked.get(),
                    "GC VERIFY: sweep sonrası mark biti kalmış obje var"
                );
            }
        });
    }
}

#[inline]
pub fn is_marked(value: Value16) -> bool {
    let Some(ptr) = value.0.as_ptr() else {
        return false;
    };
    unsafe { (*(ptr as *const DynamicObject)).marked.get() }
}

#[cfg(test)]
mod tests {
    use super::{bytes_allocated, heap_object_count, trace_children, trace_value};
    use crate::{
        ClassData, DataData, DynamicObject, FunctionData, GeneratorState16, InstanceData,
        ObjMap, PromiseState16, SymId, Value16,
    };
    use parking_lot::{Mutex, RwLock};
    use std::{collections::HashMap, mem, sync::Arc};

    #[test]
    fn constructors_register_allocations() {
        let before_count = heap_object_count();
        let before_bytes = bytes_allocated();

        let _string = Value16::string("0123456789abcdef");
        let _array = Value16::array(vec![]);
        let _option = Value16::option(None);

        assert_eq!(heap_object_count(), before_count + 3);
        assert_eq!(
            bytes_allocated(),
            before_bytes + (3 * mem::size_of::<DynamicObject>())
        );
    }

    #[test]
    fn tracing_marks_reachable_dynamic_graph() {
        fn marked(value: Value16) -> bool {
            let ptr = value.0.as_ptr().expect("dynamic value");
            unsafe { (*(ptr as *const DynamicObject)).marked.get() }
        }

        let leaf = Value16::string("leaf-node-over-15");
        let promise = Value16::promise(PromiseState16::Resolved(Box::new(leaf)));

        let generator_state = Arc::new(Mutex::new(GeneratorState16::from(vec![leaf])));
        assert_eq!(generator_state.lock().advance(), Some(leaf));
        let generator = Value16::generator(generator_state);

        let data = Value16::data(DataData {
            type_name: "TraceData".to_string(),
            fields: {
                let mut m = ObjMap::default();
                m.insert(SymId::from("promise"), promise);
                m
            },
        });

        let class = Value16::class(ClassData {
            name: "TraceClass".to_string(),
            methods: {
                let mut m = ObjMap::default();
                m.insert(SymId::from("method"), generator);
                m
            },
            fields: {
                let mut m = ObjMap::default();
                m.insert(SymId::from("data"), data);
                m
            },
            parent: Some(leaf),
            vtable: {
                let mut m = ObjMap::default();
                m.insert(SymId::from("method"), generator);
                m
            },
            method_access: HashMap::new(),
            is_abstract: false,
        });

        let instance = Value16::instance(InstanceData {
            class_name: "TraceClass".to_string(),
            fields: {
                let mut m = ObjMap::default();
                m.insert(SymId::from("leaf"), leaf);
                m
            },
            class,
        });

        let captured = Arc::new(RwLock::new(instance));
        let function = Value16::function(FunctionData {
            name: "traceFn".to_string(),
            params: vec![],
            chunk_name: "trace_chunk".to_string(),
            chunk_sym: crate::interner::intern("trace_chunk").0,
            captures: HashMap::from([("captured".to_string(), captured)]),
        });

        let option = Value16::option(Some(function));
        let result = Value16::result(Ok(class));
        let root = {
            let mut m = ObjMap::default();
            m.insert(SymId::from("option"), option);
            m.insert(SymId::from("result"), result);
            m.insert(SymId::from("map"), Value16::map(vec![(leaf, data)]));
            m.insert(SymId::from("set"), Value16::set(vec![generator]));
            Value16::object(m)
        };
        let unreachable = Value16::string("unreachable-over-15");

        let mut gray = Vec::new();
        trace_value(root, &mut gray);
        while let Some(ptr) = gray.pop() {
            unsafe { trace_children(&*ptr, &mut gray) };
        }

        assert!(marked(root));
        assert!(marked(option));
        assert!(marked(result));
        assert!(marked(function));
        assert!(marked(instance));
        assert!(marked(class));
        assert!(marked(data));
        assert!(marked(generator));
        assert!(marked(promise));
        assert!(marked(leaf));
        assert!(!marked(unreachable));
    }

    #[test]
    fn bigint_constructor_normalizes_down_to_int_for_small_values() {
        let five = Value16::bigint(num_bigint::BigInt::from(5u8));
        assert!(five.is_int(), "bigint(5) should be Int (fast path)");
        assert_eq!(five.as_int(), Some(5));
    }

    #[test]
    fn bigint_constructor_allocates_for_large_values() {
        let big = Value16::bigint(num_bigint::BigInt::from(2u32).pow(100));
        assert!(big.is_bigint(), "bigint(2^100) should be BigInt (heap)");
        assert!(!big.is_int());
    }

    #[test]
    fn bigint_equality_is_content_based() {
        let big_val = num_bigint::BigInt::from(2u32).pow(100);
        let a = Value16::bigint(big_val.clone());
        let b = Value16::bigint(big_val);
        assert!(a.values_equal(&b), "same-content BigInt should be equal");
        // BigInt and Int should NOT be equal (different types).
        let c = Value16::int(100);
        assert!(!a.values_equal(&c), "BigInt(2^100) != Int(100)");
    }

    #[test]
    fn bigint_as_number_converts_to_f64() {
        let big = Value16::bigint(num_bigint::BigInt::from(42u8));
        let n = big.as_number().expect("BigInt should convert to f64");
        assert!((n - 42.0).abs() < 0.001);
    }
}
