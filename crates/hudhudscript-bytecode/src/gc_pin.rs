//! V2-C2: GcPin — RAII root for values that live outside VM dispatch.
//! Values pinned while crossing await boundaries or held by native code
//! survive garbage collection until the pin is dropped.

use crate::Value16;
use std::cell::RefCell;

thread_local! {
    /// Pinned GC roots — scanned by collect() after mark_roots.
    static PINNED: RefCell<Vec<Value16>> = RefCell::new(Vec::new());
}

/// RAII guard that keeps a value alive across GC collections.
/// When dropped, the value becomes eligible for collection again.
///
/// # Usage
/// ```ignore
/// let pin = gc::pin(my_value);
/// // ... suspend (await, native call, etc.) — collect may run
/// // my_value survives
/// drop(pin); // now collectable
/// ```
pub struct GcPin {
    index: usize,
}

/// Pin a value — it survives GC until the returned `GcPin` is dropped.
pub fn pin(value: Value16) -> GcPin {
    let index = PINNED.with(|p| {
        let mut v = p.borrow_mut();
        // G1.4.3: reuse null tombstone slots to prevent unbounded Vec growth
        if let Some(idx) = v.iter().position(|val| val.is_null()) {
            v[idx] = value;
            idx
        } else {
            let idx = v.len();
            v.push(value);
            idx
        }
    });
    GcPin { index }
}

impl Drop for GcPin {
    fn drop(&mut self) {
        PINNED.with(|p| {
            let mut v = p.borrow_mut();
            v[self.index] = Value16::null();
        });
    }
}

/// Number of currently active (non-null) pinned values.
pub fn pinned_count() -> usize {
    PINNED.with(|p| p.borrow().iter().filter(|v| !v.is_null()).count())
}

/// Called by collect() to trace pinned values as roots.
/// Scans all non-null slots.
pub fn trace_pinned(gray: &mut Vec<*mut super::DynamicObject>) {
    PINNED.with(|p| {
        let v = p.borrow();
        for value in v.iter() {
            if !value.is_null() {
                crate::gc::trace_value(*value, gray);
            }
        }
    });
}
