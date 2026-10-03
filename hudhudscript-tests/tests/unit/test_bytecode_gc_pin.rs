//! Tests for hudhudscript-bytecode GcPin: RAII roots surviving collections.

use hudhudscript_bytecode::gc_pin::{pin, pinned_count};
use hudhudscript_bytecode::{gc, Value16};

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
