//! Mark-sweep machinery (`trace_value`/`trace_children`/`drain_gray`/
//! `collect`/`is_marked`) split from gc.rs.

use crate::gc::{
    gc_growth_factor, gc_min_threshold, gc_verify_enabled, with_heap, GcRootSource,
    BIGINT_FREE_LIST, BIGINT_FREE_LIST_MAX, GC_PENDING,
};
use crate::{DynamicData, DynamicKind, DynamicObject, PromiseState16, Value16};

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
            if added {
                continue;
            }
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
