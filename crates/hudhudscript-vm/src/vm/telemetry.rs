//! Feature-gated VM telemetry counters (GATE 2).
//!
//! Enabled via `telemetry` Cargo feature. When disabled, the entire module
//! compiles away — no fields, no init, no overhead.
//!
//! Counters use plain `u64` (not atomics) because they are per-VM,
//! single-threaded during execution. Reset per `VM::execute()` call.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-VM telemetry counters. All fields are plain `u64` — no atomics.
/// The VM owns this struct and provides `&mut` access during execution.
#[derive(Debug, Clone)]
pub struct Telemetry {
    // ── Opcode counters ───────────────────────────────────────────
    pub total_instructions: u64,

    // ── Cache counters ────────────────────────────────────────────
    pub call_cache_hit: u64,
    pub call_cache_miss: u64,
    pub chunk_cache_hit: u64,
    pub chunk_cache_miss: u64,

    // ── BigInt counters ───────────────────────────────────────────
    /// Number of Int→BigInt promotions (overflow in add/sub/mul).
    pub bigint_promotion: u64,
    /// Number of BigInt heap allocations (includes promotions and direct creates).
    pub bigint_alloc: u64,

    // ── B1: Packed/unpacked dispatch counters ────────────────────
    pub packed_dispatch_count: u64,
    pub unpacked_dispatch_count: u64,
    pub packed_fallthrough_count: u64,

    // ── B1: String index counters ────────────────────────────────
    pub string_index_count: u64,
    pub string_index_clone_count: u64,
    pub string_index_clone_bytes: u64,

    // ── B1: Loop counters ────────────────────────────────────────
    pub loop_begin_end_count: u64,

    // ── P0: Dense opcode histogram (256 slots, index = dense opcode) ──
    pub opcode_counts: Vec<u64>,
    /// P0: fallthrough broken down by dense opcode (256 slots).
    pub fallthrough_by_opcode: Vec<u64>,
    /// P0: (previous_dense, current_dense) bigram histogram.
    pub opcode_bigrams: HashMap<(u16, u16), u64>,
    /// P0: allocation count by object kind (index = DynamicKind discriminant).
    pub alloc_count_by_kind: Vec<u64>,

    // ── G5A: REFSEM öncesi hot-path sayaçları ──────────────────
    pub property_lookup_count: u64,
    pub scope_cell_lookup_count: u64,

    // ── internal bookkeeping (not serialized) ────────────────────
    pub last_dense: u16,
    pub unpacked_opcode_counts: HashMap<&'static str, u64>,

    // ── GC counters ──────────────────────────────────────────────
    pub gc_cycle_count: u64,
    pub gc_mark_count: u64,
    pub gc_sweep_count: u64,
    pub gc_pause_ns_total: u64,
    pub gc_pause_ns_max: u64,
    pub gc_heap_bytes_after_sweep: u64,

    // ── Fusion counters ──────────────────────────────────────────
    pub fusion_emitted_by_opcode: HashMap<&'static str, u64>,
    pub fusion_executed_by_opcode: HashMap<&'static str, u64>,
    pub fusion_rejected_by_reason: HashMap<&'static str, u64>,

    // ── Int slow-path ────────────────────────────────────────────
    pub int_add_slow_count: u64,
    pub site_call_count: u64,
    pub site_property_count: u64,
    pub site_index_count: u64,

    // ── G2: Site type counters ────────────────────────────────────
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            total_instructions: 0,
            call_cache_hit: 0,
            call_cache_miss: 0,
            chunk_cache_hit: 0,
            chunk_cache_miss: 0,
            bigint_promotion: 0,
            bigint_alloc: 0,
            packed_dispatch_count: 0,
            unpacked_dispatch_count: 0,
            packed_fallthrough_count: 0,
            string_index_count: 0,
            string_index_clone_count: 0,
            string_index_clone_bytes: 0,
            loop_begin_end_count: 0,
            opcode_counts: vec![0u64; 256],
            fallthrough_by_opcode: vec![0u64; 256],
            opcode_bigrams: HashMap::new(),
            alloc_count_by_kind: vec![0u64; 17],
            last_dense: 0xFFFF,
            unpacked_opcode_counts: HashMap::new(),
            property_lookup_count: 0,
            scope_cell_lookup_count: 0,
            gc_cycle_count: 0,
            gc_mark_count: 0,
            gc_sweep_count: 0,
            gc_pause_ns_total: 0,
            gc_pause_ns_max: 0,
            gc_heap_bytes_after_sweep: 0,
            fusion_emitted_by_opcode: HashMap::new(),
            fusion_executed_by_opcode: HashMap::new(),
            fusion_rejected_by_reason: HashMap::new(),
            int_add_slow_count: 0,
            site_call_count: 0,
            site_property_count: 0,
            site_index_count: 0,
        }
    }
}

impl Telemetry {
    /// Reset all counters to zero for a new VM execution.
    pub fn reset(&mut self) {
        self.total_instructions = 0;
        self.call_cache_hit = 0;
        self.call_cache_miss = 0;
        self.chunk_cache_hit = 0;
        self.chunk_cache_miss = 0;
        self.bigint_promotion = 0;
        self.bigint_alloc = 0;
        self.packed_dispatch_count = 0;
        self.unpacked_dispatch_count = 0;
        self.packed_fallthrough_count = 0;
        self.string_index_count = 0;
        self.string_index_clone_count = 0;
        self.string_index_clone_bytes = 0;
        self.loop_begin_end_count = 0;
        // P0: reset vecs by zeroing in-place (avoid re-alloc)
        for v in self.opcode_counts.iter_mut() { *v = 0; }
        for v in self.fallthrough_by_opcode.iter_mut() { *v = 0; }
        self.opcode_bigrams.clear();
        for v in self.alloc_count_by_kind.iter_mut() { *v = 0; }
        self.last_dense = 0xFFFF;
        self.unpacked_opcode_counts.clear();
        self.property_lookup_count = 0;
        self.scope_cell_lookup_count = 0;
        self.gc_cycle_count = 0;
        self.gc_mark_count = 0;
        self.gc_sweep_count = 0;
        self.gc_pause_ns_total = 0;
        self.gc_pause_ns_max = 0;
        self.gc_heap_bytes_after_sweep = 0;
        self.fusion_emitted_by_opcode.clear();
        self.fusion_executed_by_opcode.clear();
        self.fusion_rejected_by_reason.clear();
        self.int_add_slow_count = 0;
        self.site_call_count = 0;
        self.site_property_count = 0;
        self.site_index_count = 0;
    }

    /// FAZ0-A/E2 — fusion "emitted" sayımı: NİHAİ bytecode kesiti.
    /// `emit()` noktasında sayım optimizer pass'lerinin (fuse_super*,
    /// fuse_slot_immediate) emit SONRASI ürettiği fused komutları kaçırır;
    /// tek doğru boğaz, optimizer dahil her yolun çıktısı olan nihai komut
    /// akışıdır. Ana chunk + o anda kayıtlı tüm fonksiyon chunk'ları
    /// taranır (çalışma anında geç tanımlanan chunk'lar kapsam dışıdır —
    /// census execute başında bir kez alınır).
    #[cfg(feature = "telemetry")]
    pub fn census_fusion_emitted(&mut self, bytecode: &hudhudscript_bytecode::Bytecode) {
        for instr in &bytecode.instructions {
            if let Some(name) = fused_name(instr) {
                *self.fusion_emitted_by_opcode.entry(name).or_insert(0) += 1;
            }
        }
        for chunk in bytecode.functions.borrow().iter() {
            for instr in &chunk.instructions {
                if let Some(name) = fused_name(instr) {
                    *self.fusion_emitted_by_opcode.entry(name).or_insert(0) += 1;
                }
            }
        }
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            total_instructions: self.total_instructions,
            call_cache_hit: self.call_cache_hit,
            call_cache_miss: self.call_cache_miss,
            chunk_cache_hit: self.chunk_cache_hit,
            chunk_cache_miss: self.chunk_cache_miss,
            bigint_promotion: self.bigint_promotion,
            bigint_alloc: self.bigint_alloc,
            packed_dispatch_count: self.packed_dispatch_count,
            unpacked_dispatch_count: self.unpacked_dispatch_count,
            packed_fallthrough_count: self.packed_fallthrough_count,
            string_index_count: self.string_index_count,
            string_index_clone_count: self.string_index_clone_count,
            string_index_clone_bytes: self.string_index_clone_bytes,
            loop_begin_end_count: self.loop_begin_end_count,
            opcode_counts: self.opcode_counts.clone(),
            fallthrough_by_opcode: self.fallthrough_by_opcode.clone(),
            unpacked_opcode_counts: self.unpacked_opcode_counts.iter()
                .map(|(k, v)| (k.to_string(), *v)).collect(),
            opcode_bigrams: self.opcode_bigrams.iter().map(|(k, v)| (*k, *v)).collect(),
            property_lookup_count: self.property_lookup_count,
            scope_cell_lookup_count: self.scope_cell_lookup_count,
            alloc_count_by_kind: self.alloc_count_by_kind.clone(),
            gc_cycle_count: self.gc_cycle_count,
            gc_mark_count: self.gc_mark_count,
            gc_sweep_count: self.gc_sweep_count,
            gc_pause_ns_total: self.gc_pause_ns_total,
            gc_pause_ns_max: self.gc_pause_ns_max,
            gc_heap_bytes_after_sweep: self.gc_heap_bytes_after_sweep,
            fusion_emitted_by_opcode: self.fusion_emitted_by_opcode.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            fusion_executed_by_opcode: self.fusion_executed_by_opcode.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            fusion_rejected_by_reason: self.fusion_rejected_by_reason.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            int_add_slow_count: self.int_add_slow_count,
            site_call_count: self.site_call_count,
            site_property_count: self.site_property_count,
            site_index_count: self.site_index_count,
        }
    }
}

/// A snapshot read from `Telemetry`, suitable for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub total_instructions: u64,
    pub call_cache_hit: u64,
    pub call_cache_miss: u64,
    pub chunk_cache_hit: u64,
    pub chunk_cache_miss: u64,
    pub bigint_promotion: u64,
    pub bigint_alloc: u64,
    pub packed_dispatch_count: u64,
    pub unpacked_dispatch_count: u64,
    pub packed_fallthrough_count: u64,
    pub string_index_count: u64,
    pub string_index_clone_count: u64,
    pub string_index_clone_bytes: u64,
    pub loop_begin_end_count: u64,
    pub opcode_counts: Vec<u64>,
    pub fallthrough_by_opcode: Vec<u64>,
    pub unpacked_opcode_counts: Vec<(String, u64)>,
    pub opcode_bigrams: Vec<((u16, u16), u64)>,
    // G5A: REFSEM öncesi hot-path sayaçları
    pub property_lookup_count: u64,
    pub scope_cell_lookup_count: u64,
    pub alloc_count_by_kind: Vec<u64>,
    pub gc_cycle_count: u64,
    pub gc_mark_count: u64,
    pub gc_sweep_count: u64,
    pub gc_pause_ns_total: u64,
    pub gc_pause_ns_max: u64,
    pub gc_heap_bytes_after_sweep: u64,
    pub fusion_emitted_by_opcode: Vec<(String, u64)>,
    pub fusion_executed_by_opcode: Vec<(String, u64)>,
    pub fusion_rejected_by_reason: Vec<(String, u64)>,
    pub int_add_slow_count: u64,
    pub site_call_count: u64,
    pub site_property_count: u64,
    pub site_index_count: u64,
}

/// FAZ0-A/E2 — TEK fusion envanteri (Kural 7): bir Instruction'ın "fused"
/// (birleşik/süper-komut ya da immediate-katlanmış) olup olmadığına yalnız
/// buradan karar verilir; isimler tek tablodan (`instruction_name`) gelir.
/// Hem emitted census (`census_fusion_emitted`) hem executed sayacı
/// (`step_one` dispatch girişi) bu envanteri kullanır — ikinci isim
/// tablosu ya da ikinci karar noktası YASAK.
#[cfg(feature = "telemetry")]
pub fn fused_name(instr: &hudhudscript_bytecode::Instruction) -> Option<&'static str> {
    use hudhudscript_bytecode::Instruction as I;
    let fused = matches!(
        instr,
        // immediate-katlama (fuse_slot_immediate)
        I::IntAddI { .. } | I::IntSubI { .. } | I::IntMulI { .. } | I::IntDivI { .. }
            | I::IntModI { .. } | I::NumAddI { .. } | I::NumSubI { .. } | I::NumMulI { .. }
            | I::NumDivI { .. } | I::IntCmpI { .. }
            // aritmetik compound (fuse_super*)
            | I::IntMulAddAssign { .. } | I::NumMulAddAssign { .. } | I::NumMulAddIndexed { .. }
            | I::FloatMulAdd { .. } | I::IntMulModI { .. } | I::IntMulMod { .. }
            | I::IntModCmpI { .. } | I::PropertySubAssign { .. }
            // const-katlama (LoadIntConst HARİÇ: o doğrudan codegen, fusion değil)
            | I::ReturnConst { .. } | I::StoreConst { .. } | I::StoreGlobalConst { .. }
            | I::ArrayPushConst { .. } | I::ArrayPushIntConst { .. }
            // cmp+branch / döngü kuyruğu
            | I::IntCmpIJumpIfFalse { .. } | I::IntCmpRRJumpIfFalse { .. }
            | I::IntCmpIJumpIfTrue { .. } | I::IntLtJumpIfFalse(_) | I::IntLeJumpIfFalse(_)
            | I::IntLtRRJumpPacked(_) | I::IntLeRRJumpPacked(_) | I::IntCmpRRJumpPacked { .. }
            | I::IntLtRRJumpIfFalse { .. } | I::IntLeRRJumpIfFalse { .. }
            | I::IntAddIJump { .. } | I::IntSubIJump { .. } | I::LoopEndIntAddIJump { .. }
            // arith+return / arith+call
            | I::IntMulReturn { .. } | I::IntAddReturn { .. } | I::IntSubReturn { .. }
            | I::IntDivReturn { .. } | I::IntCmpIReturn { .. }
            | I::IntSubCall1(_) | I::IntAddCall1(_)
            // string/char özel
            | I::StrCharEqRR { .. } | I::StrCat3 { .. } | I::CharDispatch { .. }
            // 2D index
            | I::Index2D { .. } | I::IndexAssign2D { .. }
    );
    if fused {
        Some(instruction_name(instr))
    } else {
        None
    }
}

/// Return a short static string name for an instruction variant, suitable
/// for unpacked-opcode telemetry counting.
#[cfg(feature = "telemetry")]
pub fn instruction_name(instr: &hudhudscript_bytecode::Instruction) -> &'static str {
    use hudhudscript_bytecode::Instruction;
    match instr {
        Instruction::Call { .. } => "Call",
        Instruction::MethodCall { .. } => "MethodCall",
        Instruction::LoadGlobal { .. } => "LoadGlobal",
        Instruction::StoreGlobal { .. } => "StoreGlobal",
        Instruction::DeclGlobal { .. } => "DeclGlobal",
        Instruction::LoadConst { .. } => "LoadConst",
        Instruction::LoadNumConst { .. } => "LoadNumConst",
        Instruction::Move { .. } => "Move",
        Instruction::Jump { .. } => "Jump",
        Instruction::JumpIfFalse { .. } => "JumpIfFalse",
        Instruction::JumpIfTrue { .. } => "JumpIfTrue",
        Instruction::Return { .. } => "Return",
        Instruction::IntAdd { .. } => "IntAdd",
        Instruction::IntSub { .. } => "IntSub",
        Instruction::IntMulI { .. } => "IntMulI",
        Instruction::IntMul { .. } => "IntMul",
        Instruction::IntDiv { .. } => "IntDiv",
        Instruction::IntMod { .. } => "IntMod",
        Instruction::NumAdd { .. } => "NumAdd",
        Instruction::NumSub { .. } => "NumSub",
        Instruction::NumMul { .. } => "NumMul",
        Instruction::NumDiv { .. } => "NumDiv",
        Instruction::NumMod { .. } => "NumMod",
        Instruction::IntCmp { .. } => "IntCmp",
        Instruction::IntCmpI { .. } => "IntCmpI",
        Instruction::IntCmpIJumpIfFalse { .. } => "IntCmpIJumpIfFalse",
        Instruction::IntCmpRRJumpIfFalse { .. } => "IntCmpRRJumpIfFalse",
        Instruction::IntCmpRRJumpPacked { .. } => "IntCmpRRJumpPacked",
        Instruction::IntCmpIJumpIfTrue { .. } => "IntCmpIJumpIfTrue",
        Instruction::GetProperty { .. } => "GetProperty",
        Instruction::SetProperty { .. } => "SetProperty",
        Instruction::ObjLitSet { .. } => "ObjLitSet",
        Instruction::Index { .. } => "Index",
        Instruction::IndexAssign { .. } => "IndexAssign",
        Instruction::MakeArray { .. } => "MakeArray",
        Instruction::MakeObject { .. } => "MakeObject",
        Instruction::ArrayPush { .. } => "ArrayPush",
        Instruction::ArrayPop { .. } => "ArrayPop",
        Instruction::ArrayLen { .. } => "ArrayLen",
        Instruction::StringLen { .. } => "StringLen",
        Instruction::StrCat { .. } => "StrCat",
        Instruction::IntAddIJump { .. } => "IntAddIJump",
        Instruction::IntSubIJump { .. } => "IntSubIJump",
        Instruction::LoopBegin { .. } => "LoopBegin",
        Instruction::LoopEnd => "LoopEnd",
        Instruction::Break => "Break",
        Instruction::Continue => "Continue",
        Instruction::LoadIntConst { .. } => "LoadIntConst",
        Instruction::ReturnConst { .. } => "ReturnConst",
        Instruction::StoreGlobalConst { .. } => "StoreGlobalConst",
        Instruction::LoadClosureSlot { .. } => "LoadClosureSlot",
        Instruction::StoreClosureSlot { .. } => "StoreClosureSlot",
        Instruction::IntMulReturn { .. } => "IntMulReturn",
        Instruction::IntAddReturn { .. } => "IntAddReturn",
        Instruction::IntSubReturn { .. } => "IntSubReturn",
        Instruction::IntDivReturn { .. } => "IntDivReturn",
        Instruction::IntCmpIReturn { .. } => "IntCmpIReturn",
        Instruction::Not { .. } => "Not",
        Instruction::Neg { .. } => "Neg",
        Instruction::NumMulAddIndexed { .. } => "NumMulAddIndexed",
        Instruction::IntLtRRJumpPacked(_) => "IntLtRRJumpPacked",
        Instruction::IntLeRRJumpPacked(_) => "IntLeRRJumpPacked",
        Instruction::IntLtJumpIfFalse(_) => "IntLtJumpIfFalse",
        Instruction::IntLeJumpIfFalse(_) => "IntLeJumpIfFalse",
        Instruction::IndexArray { .. } => "IndexArray",
        Instruction::IntMulModI { .. } => "IntMulModI",
        Instruction::StrCharEqRR { .. } => "StrCharEqRR",
        Instruction::StringIndexOf { .. } => "StringIndexOf",
        Instruction::StringContains { .. } => "StringContains",
        Instruction::ArrayLen { .. } => "ArrayLen",
        Instruction::StringLen { .. } => "StringLen",
        Instruction::ArrayPop { .. } => "ArrayPop",
        Instruction::IndexArray { .. } => "IndexArray",
        Instruction::ArrayPush { .. } => "ArrayPush",
        Instruction::MakeArray { .. } => "MakeArray",
        Instruction::MakeObject { .. } => "MakeObject",
        Instruction::StoreConst { .. } => "StoreConst",
        Instruction::LoopEnd => "LoopEnd",
        Instruction::IntAddIJump { .. } => "IntAddIJump",
        Instruction::IntSubIJump { .. } => "IntSubIJump",
        Instruction::LoopEndIntAddIJump { .. } => "LoopEndIntAddIJump",
        Instruction::IntMulModI { .. } => "IntMulModI",
        Instruction::Neg { .. } => "Neg",
        Instruction::Not { .. } => "Not",
        Instruction::NumMulAddIndexed { .. } => "NumMulAddIndexed",
        Instruction::IntLtRRJumpPacked(_) => "IntLtRRJumpPacked",
        Instruction::IntLeRRJumpPacked(_) => "IntLeRRJumpPacked",
        Instruction::IntLtJumpIfFalse(_) => "IntLtJumpIfFalse",
        Instruction::IntLeJumpIfFalse(_) => "IntLeJumpIfFalse",
        Instruction::Jump(_) => "Jump",
        Instruction::TryBegin(_) => "TryBegin",
        Instruction::FinallyBegin(_) => "FinallyBegin",
        Instruction::FinallyExit(_) => "FinallyExit",
        Instruction::LoopBegin(_) => "LoopBegin",
        Instruction::IntSubCall1(_) => "IntSubCall1",
        Instruction::IntAddCall1(_) => "IntAddCall1",
        Instruction::DestructArray(..) => "DestructArray",
        Instruction::Index2D { .. } => "Index2D",
        Instruction::IndexAssign2D { .. } => "IndexAssign2D",
        Instruction::IntMulAddAssign { .. } => "IntMulAddAssign",
        Instruction::DefineFunction(_) => "DefineFunction",
        Instruction::MakeGenerator { .. } => "MakeGenerator",
        Instruction::TailCall { .. } => "TailCall",
        Instruction::CallSpread(_) => "CallSpread",
        Instruction::StrCat3 { .. } => "StrCat3",
        Instruction::StrCatMut { .. } => "StrCatMut",
        Instruction::StringConcat { .. } => "StringConcat",
        Instruction::ForIn { .. } => "ForIn",
        Instruction::IterNext { .. } => "IterNext",
        Instruction::IntCmpIReturn { .. } => "IntCmpIReturn",
        Instruction::IntDivReturn { .. } => "IntDivReturn",
        Instruction::IndexAssignArray { .. } => "IndexAssignArray",
        Instruction::NumSqrt { .. } => "NumSqrt",
        Instruction::NumSin { .. } => "NumSin",
        Instruction::NumCos { .. } => "NumCos",
        Instruction::FLoadNum { .. } => "FLoadNum",
        Instruction::FStoreNum { .. } => "FStoreNum",
        Instruction::FAdd { .. } => "FAdd",
        Instruction::FSub { .. } => "FSub",
        Instruction::FMul { .. } => "FMul",
        Instruction::FDiv { .. } => "FDiv",
        Instruction::FSin { .. } => "FSin",
        Instruction::FCos { .. } => "FCos",
        Instruction::FSqrt { .. } => "FSqrt",
        Instruction::FConst { .. } => "FConst",
        Instruction::FMove { .. } => "FMove",
        Instruction::FloatMulAdd { .. } => "FloatMulAdd",
        Instruction::FloatAdd { .. } => "FloatAdd",
        Instruction::FloatMul { .. } => "FloatMul",
        Instruction::NumMulAddAssign { .. } => "NumMulAddAssign",
        Instruction::NumAddI { .. } => "NumAddI",
        Instruction::NumSubI { .. } => "NumSubI",
        Instruction::NumMulI { .. } => "NumMulI",
        Instruction::NumDivI { .. } => "NumDivI",
        Instruction::NewInstance { .. } => "NewInstance",
        Instruction::DeclStore { .. } => "DeclStore",
        Instruction::SpreadIntoArray { .. } => "SpreadIntoArray",
        Instruction::SpreadIntoObject { .. } => "SpreadIntoObject",
        Instruction::ArrayPushConst { .. } => "ArrayPushConst",
        Instruction::ArrayPushIntConst { .. } => "ArrayPushIntConst",
        Instruction::PropertySubAssign { .. } => "PropertySubAssign",
        Instruction::Spawn { .. } => "Spawn",
        Instruction::Despawn { .. } => "Despawn",
        Instruction::ViewAs { .. } => "ViewAs",
        Instruction::Send { .. } => "Send",
        Instruction::Receive { .. } => "Receive",
        Instruction::Require { .. } => "Require",
        Instruction::Perform { .. } => "Perform",
        Instruction::Remember { .. } => "Remember",
        Instruction::Recall { .. } => "Recall",
        Instruction::Forget { .. } => "Forget",
        Instruction::TryEnd => "TryEnd",
        Instruction::FinallyEnd => "FinallyEnd",
        Instruction::Throw { .. } => "Throw",
        Instruction::Yield { .. } => "Yield",
        Instruction::Await { .. } => "Await",
        Instruction::IntMulMod { .. } => "IntMulMod",
        Instruction::IntMulModI { .. } => "IntMulModI",
        Instruction::NumMulAddIndexed { .. } => "NumMulAddIndexed",
        Instruction::IntAddI { .. } => "IntAddI",
        Instruction::IntSubI { .. } => "IntSubI",
        Instruction::IntDivI { .. } => "IntDivI",
        Instruction::IntModI { .. } => "IntModI",
        Instruction::IntModCmpI { .. } => "IntModCmpI",
        Instruction::Break => "Break",
        Instruction::Continue => "Continue",
        Instruction::LoopEnd => "LoopEnd",
        Instruction::CharDispatch { .. } => "CharDispatch",
        Instruction::IntLeRRJumpIfFalse { .. } => "IntLeRRJumpIfFalse",
        Instruction::IntLtRRJumpIfFalse { .. } => "IntLtRRJumpIfFalse",
        Instruction::BindVar(_) => "BindVar",
        Instruction::EnumDecl(_) => "EnumDecl",
        Instruction::MatchVariant(_) => "MatchVariant",
        Instruction::ClassDecl(_) => "ClassDecl",
        Instruction::TraitCheck(_) => "TraitCheck",
        Instruction::LoadModule(_) => "LoadModule",
        Instruction::ClassStaticDecl(_) => "ClassStaticDecl",
        Instruction::GetStatic(_) => "GetStatic",
        Instruction::SuperCall { .. } => "SuperCall",
        Instruction::MethodCallSpread(_) => "MethodCallSpread",
        Instruction::DestructObject(_) => "DestructObject",
        Instruction::IndexStringAscii { .. } => "IndexStringAscii",
        Instruction::MakeArray2 { .. } => "MakeArray2",
        Instruction::TryEnd => "TryEnd",
        Instruction::FinallyEnd => "FinallyEnd",
        _ => "other",
    }
}
