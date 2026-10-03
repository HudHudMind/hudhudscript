//! Tests for hudhudscript-native-abi `JitExit` — the uniform native-entry
//! result block: C layout and status constructors/names.

use hudhudscript_native_abi::jit_exit::{JitExit, JIT_EXIT_RETURNED};

// ── C layout ─────────────────────────────────────────────────────

#[test]
fn layout_is_16_bytes_8_aligned() {
    assert_eq!(std::mem::size_of::<JitExit>(), 16);
    assert_eq!(std::mem::align_of::<JitExit>(), 8);
}

// ── constructors and status names ────────────────────────────────

#[test]
fn constructors_and_names() {
    let r = JitExit::returned(5);
    assert_eq!(r.status, JIT_EXIT_RETURNED);
    assert_eq!(r.value, 5);
    assert_eq!(r.status_name(), "Returned");
    assert_eq!(JitExit::overflow().status_name(), "Overflow");
}
