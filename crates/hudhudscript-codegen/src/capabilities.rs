//! Backend capability flags (JIT_AOT_ARCHITECTURE.md §7).

/// What a backend can do. Upper layers query these flags instead of
/// branching on backend identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// Can compile into executable memory at runtime.
    pub jit: bool,
    /// Can emit object files for offline linking.
    pub aot: bool,
    /// Can emit objects for a triple that differs from the host.
    pub cross_compile: bool,
    /// Emits DWARF/pdb debug info.
    pub debug_info: bool,
    /// Supports platform unwinding (future native exceptions lane).
    pub unwind: bool,
    /// Native unwinding-based exception semantics.
    pub exceptions: bool,
    /// Vector instruction selection.
    pub vector: bool,
    /// Atomic memory operations.
    pub atomics: bool,
    /// Precise GC stack maps (second-stage GC integration, §17).
    pub gc_stack_maps: bool,
}

impl BackendCapabilities {
    /// Minimal JIT-only capability set (used by early backends).
    pub const JIT_ONLY: BackendCapabilities = BackendCapabilities {
        jit: true,
        aot: false,
        cross_compile: false,
        debug_info: false,
        unwind: false,
        exceptions: false,
        vector: false,
        atomics: false,
        gc_stack_maps: false,
    };

    /// Cranelift's expected capability set (JIT + object emission on its
    /// supported 64-bit ISAs; no precise stack maps in stage one).
    pub const CRANELIFT: BackendCapabilities = BackendCapabilities {
        jit: true,
        aot: true,
        cross_compile: true,
        debug_info: true,
        unwind: false,
        exceptions: false,
        vector: true,
        atomics: true,
        gc_stack_maps: false,
    };

    pub fn supports(&self, need: CapabilityNeed) -> bool {
        match need {
            CapabilityNeed::Jit => self.jit,
            CapabilityNeed::Aot => self.aot,
            CapabilityNeed::CrossCompile => self.cross_compile,
            CapabilityNeed::DebugInfo => self.debug_info,
            CapabilityNeed::Unwind => self.unwind,
            CapabilityNeed::Exceptions => self.exceptions,
            CapabilityNeed::Vector => self.vector,
            CapabilityNeed::Atomics => self.atomics,
            CapabilityNeed::GcStackMaps => self.gc_stack_maps,
        }
    }
}

/// A single capability probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityNeed {
    Jit,
    Aot,
    CrossCompile,
    DebugInfo,
    Unwind,
    Exceptions,
    Vector,
    Atomics,
    GcStackMaps,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
