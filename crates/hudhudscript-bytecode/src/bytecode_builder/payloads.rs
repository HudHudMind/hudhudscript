use crate::{
    Bytecode, CallPayload, ClassDeclPayload, ClassStaticDeclPayload, DefineFunctionPayload,
    DestructObjectPayload, EnumDeclPayload, LoadModulePayload, LoopPayload, OptSymPayload,
    SuperInstrPayload, CmpJumpPayload, SymId, TraitCheckPayload, TwoSymPayload,
};

impl Bytecode {
    /// Record a loop header payload (CROSS-2b) and return its index for
    /// embedding in `Instruction::LoopBegin(idx)`.
    ///
    /// The returned `u32` is appended unconditionally — loop payloads
    /// are not deduplicated because each textual loop has its own
    /// distinct back-edge, and late patching (see `patch_loop_end`)
    /// needs a stable index.
    #[inline]
    pub fn add_loop_payload(&mut self, start: u32, end: u32) -> u32 {
        let idx = self.loop_payloads.len() as u32;
        self.loop_payloads.push(LoopPayload { start, end });
        idx
    }

    /// Patch the `end` field of a loop payload previously inserted via
    /// [`add_loop_payload`].  Used by the compiler's `while` emitter
    /// which pushes `LoopBegin` before the body is compiled and later
    /// back-fills the exit IP.
    #[inline]
    pub fn patch_loop_end(&mut self, idx: u32, end: u32) {
        self.loop_payloads[idx as usize].end = end;
    }

    /// Resolve a `LoopBegin` payload by index.  Panics on out-of-bounds —
    /// compiler invariant violation (Kural 7c, no fallback).
    #[inline]
    pub fn get_loop_payload(&self, idx: u32) -> LoopPayload {
        self.loop_payloads[idx as usize]
    }

    /// Register an `EnumDecl` payload and return its index (CROSS-2a).
    /// Not deduplicated — each enum declaration has a distinct site.
    #[inline]
    pub fn add_enum_decl_payload(&mut self, payload: EnumDeclPayload) -> u32 {
        let idx = self.enum_decl_payloads.len() as u32;
        self.enum_decl_payloads.push(payload);
        idx
    }

    /// Register a `ClassDecl` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_class_decl_payload(&mut self, payload: ClassDeclPayload) -> u32 {
        let idx = self.class_decl_payloads.len() as u32;
        self.class_decl_payloads.push(payload);
        idx
    }

    /// Register a `TraitCheck` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_trait_check_payload(&mut self, payload: TraitCheckPayload) -> u32 {
        let idx = self.trait_check_payloads.len() as u32;
        self.trait_check_payloads.push(payload);
        idx
    }

    /// Register a `LoadModule` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_load_module_payload(&mut self, payload: LoadModulePayload) -> u32 {
        let idx = self.load_module_payloads.len() as u32;
        self.load_module_payloads.push(payload);
        idx
    }

    /// Register a `DefineFunction` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_define_function_payload(&mut self, payload: DefineFunctionPayload) -> u32 {
        let idx = self.define_function_payloads.len() as u32;
        self.define_function_payloads.push(payload);
        idx
    }

    /// Register a `ClassStaticDecl` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_class_static_decl_payload(&mut self, payload: ClassStaticDeclPayload) -> u32 {
        let idx = self.class_static_decl_payloads.len() as u32;
        self.class_static_decl_payloads.push(payload);
        idx
    }

    /// Register a `DestructObject` payload and return its index (CROSS-2a).
    #[inline]
    pub fn add_destruct_object_payload(&mut self, payload: DestructObjectPayload) -> u32 {
        let idx = self.destruct_object_payloads.len() as u32;
        self.destruct_object_payloads.push(payload);
        idx
    }

    /// Register a call-family payload and return its index (CROSS-2c).
    /// Shared by `Call`, `TailCall`, `MethodCall`, `NewInstance`,
    /// `Spawn`, `SuperCall`, `MakeGenerator`.  Not deduplicated —
    /// distinct call sites keep distinct entries so future per-site
    /// tweaks (inline caching, call-count profiling) remain possible.
    #[inline]
    pub fn add_call_payload(&mut self, sym: SymId, arg_count: u8) -> u32 {
        let idx = self.call_payloads.len() as u32;
        self.call_payloads.push(CallPayload { sym, arg_count, function_idx: u32::MAX, builtin_method_idx: u32::MAX });
        idx
    }

    /// P6: Add a call payload with a known builtin method ID for fast dispatch.
    pub fn add_call_payload_with_builtin(
        &mut self,
        sym: SymId,
        arg_count: u8,
        builtin_method_idx: u32,
    ) -> u32 {
        let idx = self.call_payloads.len() as u32;
        self.call_payloads.push(CallPayload {
            sym,
            arg_count,
            function_idx: u32::MAX,
            builtin_method_idx,
        });
        idx
    }

    /// Resolve a call-family payload by index (CROSS-2c).
    ///
    /// # Panics
    /// Panics on out-of-bounds — compiler invariant violation (Kural 7c,
    /// P6: return reference to avoid copy on every Call/MethodCall.
    #[inline]
    pub fn get_call_payload(&self, idx: u32) -> &CallPayload {
        &self.call_payloads[idx as usize]
    }

    /// Register a two-symbol payload and return its index (CROSS-2d).
    /// Shared by `StoreTyped`, `MatchVariant`, `DeclStore`, `GetStatic`.
    #[inline]
    pub fn add_two_sym_payload(&mut self, first: u32, second: u32) -> u32 {
        let idx = self.two_sym_payloads.len() as u32;
        self.two_sym_payloads.push(TwoSymPayload { first, second });
        idx
    }

    /// Resolve a two-symbol payload by index (CROSS-2d).
    ///
    /// # Panics
    /// Panics on out-of-bounds — compiler invariant violation (Kural 7c).
    #[inline]
    pub fn get_two_sym_payload(&self, idx: u32) -> TwoSymPayload {
        self.two_sym_payloads[idx as usize]
    }

    /// Register an optional-symbol payload and return its index
    /// (CROSS-2d).  Shared by `Remember`, `Recall`, `Forget`.
    #[inline]
    pub fn add_opt_sym_payload(&mut self, sym: Option<SymId>) -> u32 {
        let idx = self.opt_sym_payloads.len() as u32;
        self.opt_sym_payloads.push(OptSymPayload { sym });
        idx
    }

    /// Resolve an optional-symbol payload by index (CROSS-2d).
    ///
    /// # Panics
    /// Panics on out-of-bounds — compiler invariant violation (Kural 7c).
    #[inline]
    pub fn get_opt_sym_payload(&self, idx: u32) -> OptSymPayload {
        self.opt_sym_payloads[idx as usize]
    }

    /// Register an A2 super-instruction payload and return its index.
    /// Currently used by `IntSubCall1(idx)`.
    #[inline]
    pub fn add_super_instr_payload(
        &mut self,
        call_idx: u32,
        slot: u32,
        imm: i16,
        offset: i32,
    ) -> u32 {
        let idx = self.super_instr_payloads.len() as u32;
        self.super_instr_payloads.push(SuperInstrPayload {
            call_idx,
            slot,
            imm,
            offset,
            call_dst: 255,
            arg_reg: 1,
        });
        idx
    }

    /// Resolve an A2 super-instruction payload by index.
    ///
    /// # Panics
    /// Panics on out-of-bounds — compiler invariant violation (Kural 7c).
    #[inline]
    pub fn add_cmp_jump_payload(&mut self, src1: u8, src2: u8, target: u32) -> u32 {
        let idx = self.cmp_jump_payloads.len() as u32;
        self.cmp_jump_payloads.push(CmpJumpPayload { src1, src2, target });
        idx
    }

    pub fn patch_cmp_jump_target(&mut self, idx: u32, target: u32) {
        self.cmp_jump_payloads[idx as usize].target = target;
    }

    pub fn get_super_instr_payload(&self, idx: u32) -> SuperInstrPayload {
        self.super_instr_payloads[idx as usize]
    }
}
