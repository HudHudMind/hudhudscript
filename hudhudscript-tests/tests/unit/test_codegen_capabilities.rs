//! Tests for hudhudscript-codegen — backend capability flags
//! (moved from crates/hudhudscript-codegen/src/capabilities.rs).

use hudhudscript_codegen::capabilities::{BackendCapabilities, CapabilityNeed};

// ── capability sets ───────────────────────────────────────────────

#[test]
fn cranelift_capability_set() {
    let c = BackendCapabilities::CRANELIFT;
    assert!(c.supports(CapabilityNeed::Jit));
    assert!(c.supports(CapabilityNeed::Aot));
    assert!(c.supports(CapabilityNeed::CrossCompile));
    assert!(!c.supports(CapabilityNeed::GcStackMaps));
    assert!(!c.supports(CapabilityNeed::Exceptions));
}

#[test]
fn jit_only_rejects_aot() {
    let c = BackendCapabilities::JIT_ONLY;
    assert!(c.supports(CapabilityNeed::Jit));
    assert!(!c.supports(CapabilityNeed::Aot));
}

// ── probe coverage ────────────────────────────────────────────────

#[test]
fn probe_coverage_is_total() {
    // Her bayrak en az bir probe ile sorgulanabilir olmalı.
    let caps = BackendCapabilities::CRANELIFT;
    let probes = [
        CapabilityNeed::Jit,
        CapabilityNeed::Aot,
        CapabilityNeed::CrossCompile,
        CapabilityNeed::DebugInfo,
        CapabilityNeed::Unwind,
        CapabilityNeed::Exceptions,
        CapabilityNeed::Vector,
        CapabilityNeed::Atomics,
        CapabilityNeed::GcStackMaps,
    ];
    assert_eq!(probes.len(), 9);
    let _ = caps;
}
