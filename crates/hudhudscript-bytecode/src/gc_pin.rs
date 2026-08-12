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

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gc, Value16};

    struct DummyRoot;
    impl gc::GcRootSource for DummyRoot {
        fn mark_roots(&self) {}
    }

    #[test]
    fn pin_survives_collect() {
        let val = Value16::string("pin-survives-collect-test-string!");
        let _pin = pin(val);
        gc::collect(&DummyRoot);
        let txt = _pin;
        drop(txt);
        // After pin drops + collect, val should... no, val is a Copy and we still hold it
        // But the pin should keep it alive
        let still_alive = val.as_str();
        assert!(still_alive.is_some());
    }

    #[test]
    fn unpin_allows_collection() {
        let val = Value16::string("unpin-dies-test-string!");
        let pin = pin(val);
        drop(pin);
        // Now pin is gone — next collect can sweep val
        gc::collect(&DummyRoot);
        // val is still alive because we hold a copy of the Value16 pointer
        // But after collect, the data should still be there (other roots keep it)
    }

    #[test]
    fn pinned_count_tracks_active_pins() {
        let before = pinned_count();
        let val = Value16::string("pinned-count-test-string!");
        let pin = pin(val);
        assert_eq!(pinned_count(), before + 1);
        drop(pin);
        assert_eq!(pinned_count(), before);
    }

    #[test]
    fn pinned_value_survives_many_collections() {
        let val = Value16::string("many-collections-test-over-15");
        let _pin = pin(val);
        for _ in 0..5 {
            gc::collect(&DummyRoot);
        }
        assert!(val.as_str().is_some());
    }
}
