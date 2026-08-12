use hudhudscript_bytecode::*;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

mod perf2_size_guard {
    use hudhudscript_bytecode::*;
    use std::mem::size_of;

    #[test]
    fn heavy_variants_are_pointer_sized_in_enum() {
        // Construction path: each Box'd variant contributes exactly 8 bytes
        // to the enum.  The backing struct can grow freely.
        assert_eq!(size_of::<Box<FunctionData>>(), 8);
        assert_eq!(size_of::<Box<ClassData>>(), 8);
        assert_eq!(size_of::<Box<InstanceData>>(), 8);
        assert_eq!(size_of::<Box<DataData>>(), 8);
    }

    /// Audit v3 Finding 4.1 / PERF-14: `Instruction` enum size guard.
    /// Heavy variants (TraitCheck, ClassDecl, ClassStaticDecl, EnumDecl,
    /// DestructObject, LoadModule, DefineFunction) are Box'd behind a
    /// pointer so the enum stays at 24 B (tag + 2×usize payload).
    /// Baseline pre-box was 56 B; regressions above 32 B fail this test.
    #[test]
    fn instruction_size_is_reasonable() {
        let sz = size_of::<Instruction>();
        eprintln!("Instruction size: {} bytes", sz);
        assert!(
            sz <= 32,
            "Instruction grew to {} bytes — Box the new heavy variant",
            sz
        );
    }

    /// CROSS-2b target: `Instruction` enum must stay at 16 B after
    /// externalising `PushLoop` to the `loop_payloads` side table and
    /// narrowing `LoadConst` / `LoadNumConst` from `usize` to `u32`.
    ///
    /// 16 B means 4 instructions fit in one 64 B L1 cache line (vs 2.67
    /// at 24 B), which is the main driver for the dispatch-locality win
    /// measured against the fib/loop microbenchmarks.
    ///
    /// If this test fails because a new variant pushed the enum back
    /// past 16 B, either Box the heavy payload or add an entry to the
    /// appropriate side-table pool (see `loop_payloads`, `symbol_lists`).
    #[test]
    fn instruction_size_target_16_bytes() {
        let sz = size_of::<Instruction>();
        eprintln!("CROSS-2b Instruction size: {} bytes (target ≤16)", sz);
        assert!(
            sz <= 16,
            "CROSS-2b: Instruction size regression: {} bytes (target ≤16) — \
             externalise new heavy payloads to a side table",
            sz
        );
    }

    /// CROSS-2c+d target: `Instruction` collapses to 8 B after
    /// externalising the 14 remaining payload-carrying variants to
    /// `call_payloads` / `two_sym_payloads` / `opt_sym_payloads`.
    ///
    /// Eight instructions now fit per 64 B L1 cache line (vs four at
    /// 16 B) — the dominant driver for the fib / tight-loop dispatch
    /// locality win.  If this test fails because a new variant pushed
    /// the enum past 8 B, externalise the new payload to an existing
    /// pool (preferring the semantically matching one) or introduce a
    /// new side table with its own `add_/get_` helpers (Kural 7c — no
    /// inline-heavy-payload fallback).
    #[test]
    fn instruction_size_target_8_bytes() {
        let sz = size_of::<Instruction>();
        eprintln!("CROSS-2c+d Instruction size: {} bytes (target ≤8)", sz);
        assert!(
            sz <= 8,
            "CROSS-2c+d: Instruction size: {} bytes (target ≤8) — \
             externalise new heavy payloads to a side table",
            sz
        );
    }

    /// CROSS-2a proof: the 7 `Box<T>` variants have been externalised to
    /// side-table pools, so `Instruction` no longer carries any 8-byte
    /// aligned payload. Alignment should drop to 4 bytes, which is the
    /// prerequisite for future 12 B / 8 B enum targets (CROSS-2c / 2d).
    ///
    /// Size may still be 16 B at this step (the enum tag + u32 payload
    /// still rounds up to a 16 B slot because of other variants like
    /// `Call(SymId, u8)` that occupy 8 B of payload); the alignment
    /// drop is the load-bearing invariant for the next sub-issues.
    #[test]
    fn instruction_alignment_is_4_or_less() {
        use std::mem::align_of;
        let al = align_of::<Instruction>();
        eprintln!("CROSS-2a Instruction alignment: {} bytes (target ≤4)", al);
        assert!(
            al <= 4,
            "CROSS-2a: Instruction alignment not reduced: {} bytes (expected ≤4 after Box removal)",
            al
        );
    }
}

// ===================================================================
// PERF-49 (2026-04-18) — postcard migration roundtrip guard.
//
// Verifies that `to_bytes()` / `from_bytes()` use the postcard
// encoding (no bincode fallback — Kural 7c) and that a non-trivial
// bytecode payload survives a full serialize/deserialize cycle
// including the full `Value` / `Instruction` / `FunctionChunk`
// surface touched by typical compiler output.
// ===================================================================
