use crate::error::compile_codes;
use crate::{Bytecode, BytecodeResult, Instruction, ReprTag, Value16, BYTECODE_VERSION};

impl Bytecode {
    /// Add a numeric constant (already NaN-boxed to `u64`) to the pool.
    ///
    /// O(1) amortized via `numeric_index` reverse map (Audit v3 F2.2).
    /// Prefer this over `numeric_constants.iter().position()` at call sites
    /// that already hold the NaN-boxed bits (e.g. the compiler's inner loop
    /// that handles ranges and array-index patterns).
    ///
    /// Returns a `u32` index (CROSS-2b): `LoadNumConst` carries a u32.
    pub fn add_numeric_constant_bits(&mut self, bits: u64) -> u32 {
        self.rebuild_indices_if_stale();
        if let Some(&idx) = self.numeric_index.get(&bits) {
            return idx;
        }
        let idx = self.numeric_constants.len() as u32;
        self.numeric_constants.push(bits);
        self.numeric_index.insert(bits, idx);
        idx
    }

    /// Add a numeric constant to the packed pool (NaN-boxed, 8 bytes).
    ///
    /// Returns the `u32` index into `numeric_constants`.  Deduplicates
    /// by raw `u64` bits so that e.g. two `1.0` literals share one slot.
    pub fn add_numeric_constant(&mut self, value: f64) -> u32 {
        let bits = value.to_bits();
        self.rebuild_indices_if_stale();
        if let Some(&idx) = self.numeric_index.get(&bits) {
            return idx;
        }
        let idx = self.numeric_constants.len() as u32;
        self.numeric_constants.push(bits);
        self.numeric_index.insert(bits, idx);
        idx
    }

    /// Retrieve a numeric constant by index, unpacking from NaN-boxed bits.
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds — this indicates a compiler bug.
    #[inline]
    pub fn get_numeric_constant(&self, idx: usize) -> f64 {
        let bits = self.numeric_constants[idx];
        f64::from_bits(bits)
    }

    /// Add an integer constant to the pool (A3b).  Deduplicates on raw `i64`
    /// so `1`, `2`, `-1` etc. each occupy one slot no matter how often the
    /// source repeats them.  Returns a `u32` index for
    /// `Instruction::LoadIntConst(idx)`.
    pub fn add_int_constant(&mut self, value: i64) -> u32 {
        self.rebuild_indices_if_stale();
        if let Some(&idx) = self.int_index.get(&value) {
            return idx;
        }
        let idx = self.int_constants.len() as u32;
        self.int_constants.push(value);
        self.int_index.insert(value, idx);
        idx
    }

    /// Retrieve an integer constant by index (A3b).
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds — compiler invariant violation
    /// (Kural 7c: no runtime fallback).
    #[inline]
    pub fn get_int_constant(&self, idx: usize) -> i64 {
        self.int_constants[idx]
    }

    /// Add constant to pool, return its `u32` index (CROSS-2b, Struct-3d-b).
    /// Deduplicates identical constants to save memory (#459).
    pub fn add_constant(&mut self, value: Value16) -> u32 {
        // Check for existing identical constant
        for (i, existing) in self.constants.iter().enumerate() {
            if Self::value16_values_equal(existing, &value) {
                return i as u32;
            }
        }
        self.constants.push(value);
        (self.constants.len() - 1) as u32
    }

    /// Simple structural equality check for Value16 constant pool deduplication.
    fn value16_values_equal(a: &Value16, b: &Value16) -> bool {
        match (a.0.tag(), b.0.tag()) {
            (ReprTag::Null, ReprTag::Null) => true,
            (ReprTag::Bool, ReprTag::Bool) => a.as_bool() == b.as_bool(),
            (ReprTag::Int, ReprTag::Int) => a.as_int() == b.as_int(),
            (ReprTag::Number, ReprTag::Number) => {
                a.as_number().map(|x| x.to_bits()) == b.as_number().map(|x| x.to_bits())
            }
            _ => false,
        }
    }

    /// Back-compat add_constant (converts Value -> Value16).
    #[inline]
    /// Push a source position entry corresponding to the last emitted instruction.
    pub fn push_source_position(&mut self, pos: Option<(usize, usize)>) {
        self.source_positions.push(pos);
    }

    /// Look up the source position for a given instruction index.
    pub fn get_source_position(&self, ip: usize) -> Option<(usize, usize)> {
        self.source_positions.get(ip).copied().flatten()
    }

    /// Push an instruction AND keep `source_positions` parallel by
    /// appending `None`. Callers that want to attach a specific source
    /// position should overwrite the trailing entry via
    /// `source_positions.last_mut()` afterward.
    ///
    /// Using this helper (instead of `instructions.push(x)` directly)
    /// lets the compiler rely on a parallel-vector invariant —
    /// `source_positions[ip]` always makes sense for every valid `ip`.
    #[inline]
    pub fn push_instr(&mut self, instr: Instruction) {
        self.instructions.push(instr);
        self.source_positions.push(None);
    }

    /// Pad `source_positions` with trailing `None`s up to the current
    /// length of `instructions`. Called by the compiler after all
    /// instructions have been emitted but before bytecode is returned,
    /// so the debug hook can safely index `source_positions[ip]`
    /// without bounds checks.
    ///
    /// Kept as a post-compile normalization step (rather than enforcing
    /// the invariant on every single push) because many emit sites in
    /// the compiler use raw `instructions.push()` for clarity / legacy
    /// reasons — this one-shot pad closes the gap after compilation.
    pub fn pad_source_positions(&mut self) {
        while self.source_positions.len() < self.instructions.len() {
            self.source_positions.push(None);
        }
        // If optimization shrank instructions below source_positions,
        // drop the tail (safest if the optimizer rewrote in-place).
        self.source_positions.truncate(self.instructions.len());
    }

    /// Serialize to bytes using postcard (varint-encoded, typically
    /// 40-60% smaller than bincode's fixed-width layout).  This is the
    /// sole production wire format for `.hudb` / `.hudc` caches as of
    /// BYTECODE_VERSION v8 (PERF-49, Audit v3 Finding 15.1).
    ///
    /// The error is returned as a `String` so call-sites can route
    /// through `HudcError::io` / `CliError::Io` uniformly; the underlying
    /// `postcard::Error` is formatted in-place.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        // COMPILE0001: snapshot interner into bytecode.symbols before serialize
        let mut bc = self.clone();
        bc.symbols = crate::interner::snapshot();
        bc.serialized_function_names = self.function_keys();
        postcard::to_stdvec(&bc).map_err(|e| format!("postcard serialize: {}", e))
    }

    /// Deserialize from bytes using postcard.
    /// Returns an error if the bytecode version does not match the
    /// expected [`BYTECODE_VERSION`] — old bincode-encoded caches are
    /// rejected here (Kural 7c: single encoding, no fallback).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut bc: Self =
            postcard::from_bytes(bytes).map_err(|e| format!("postcard deserialize: {}", e))?;
        if bc.version != BYTECODE_VERSION {
            return Err(format!(
                "Bytecode version mismatch: expected {}, got {}",
                BYTECODE_VERSION, bc.version
            ));
        }
        // COMPILE0002: restore global interner from bytecode.symbols
        if !bc.symbols.is_empty() {
            crate::interner::restore(bc.symbols.clone())
                .map_err(|e| format!("interner restore: {}", e))?;
        }
        bc.rebuild_function_names();
        bc.resolve_call_payload_function_indices();
        Ok(bc)
    }

    /// Serialize to JSON (for debugging/inspection only)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON (for debugging/inspection only)
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate bytecode before execution.
    ///
    /// Checks that:
    /// - All jump targets (Jump, JumpIfFalse, JumpIfTrue, IterNext, TryBegin) are
    ///   within instruction bounds.
    /// - All LoadConst indices are within the constants pool bounds.
    /// - The same checks are applied to every function chunk.
    pub fn validate(&self) -> BytecodeResult<()> {
        Self::validate_instructions(&self.instructions, &self.constants, self, "main")?;
        for (idx, chunk) in self.functions.borrow().iter().enumerate() {
            // Function chunks share the parent bytecode pools; only their
            // `constants` pool is local.
            Self::validate_instructions(&chunk.instructions, &chunk.constants, self, &format!("fn:{}", idx))?;
        }
        Ok(())
    }

    fn validate_instructions(
        instructions: &[Instruction],
        constants: &[Value16],
        bc: &Bytecode,
        label: &str,
    ) -> BytecodeResult<()> {
        let len = instructions.len();
        for (ip, instr) in instructions.iter().enumerate() {
            match instr {
                Instruction::Jump(offset)
                | Instruction::TryBegin(offset)
                | Instruction::FinallyBegin(offset)
                | Instruction::FinallyExit(offset) => {
                    // Relative offset: target = ip + offset (i64 arithmetic
                    // to avoid underflow on backward jumps).
                    let target = (ip as i64).wrapping_add(*offset as i64);
                    if target < 0 || target > len as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid relative jump at ip={} offset={} → target={} out of range [0,{}] in {}",
                            ip, offset, target, len, label
                        )));
                    }
                }
                Instruction::JumpIfFalse { src: 255, offset }
                | Instruction::JumpIfTrue { src: 255, offset } => {
                    let target = (ip as i64).wrapping_add(*offset as i64);
                    if target < 0 || target > len as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid relative jump at ip={} offset={} → target={} out of range [0,{}] in {}",
                            ip, offset, target, len, label
                        )));
                    }
                }
                Instruction::IterNext { end_offset, .. } => {
                    let target = (ip as i64).wrapping_add(*end_offset as i64);
                    if target < 0 || target > len as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid relative jump at ip={} offset={} → target={} out of range [0,{}] in {}",
                            ip, end_offset, target, len, label
                        )));
                    }
                }
                Instruction::LoadConst { const_idx, .. } => {
                    if (*const_idx as usize) >= constants.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid constant index {} at instruction {} in {}; pool size is {}",
                            const_idx,
                            ip,
                            label,
                            constants.len()
                        )));
                    }
                }
                // LoadNumConst removed — validated via LoadNumConst
                Instruction::LoadIntConst { const_idx, .. } => {
                    if (*const_idx as usize) >= bc.int_constants.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid int constant index {} at instruction {} in {}; pool size is {}",
                            const_idx,
                            ip,
                            label,
                            bc.int_constants.len()
                        )));
                    }
                }
                Instruction::LoopBegin(idx) => {
                    if (*idx as usize) >= bc.loop_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid loop payload index {} at instruction {} in {}; pool size is {}",
                            idx,
                            ip,
                            label,
                            bc.loop_payloads.len()
                        )));
                    }
                }
                // ── CROSS-2a: validate side-table indices for the 7
                //    externalised Box variants.  Out-of-range indices
                //    indicate a compiler bug (Kural 7c — invariant
                //    checked here, no fallback at runtime).
                Instruction::EnumDecl(idx) => {
                    if (*idx as usize) >= bc.enum_decl_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid enum_decl_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.enum_decl_payloads.len()
                        )));
                    }
                }
                Instruction::ClassDecl(idx) => {
                    if (*idx as usize) >= bc.class_decl_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid class_decl_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.class_decl_payloads.len()
                        )));
                    }
                }
                Instruction::TraitCheck(idx) => {
                    if (*idx as usize) >= bc.trait_check_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid trait_check_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.trait_check_payloads.len()
                        )));
                    }
                }
                Instruction::LoadModule(idx) => {
                    if (*idx as usize) >= bc.load_module_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid load_module_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.load_module_payloads.len()
                        )));
                    }
                }
                Instruction::DefineFunction(idx) => {
                    if (*idx as usize) >= bc.define_function_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid define_function_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.define_function_payloads.len()
                        )));
                    }
                }
                Instruction::ClassStaticDecl(idx) => {
                    if (*idx as usize) >= bc.class_static_decl_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid class_static_decl_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.class_static_decl_payloads.len()
                        )));
                    }
                }
                Instruction::DestructObject(idx) => {
                    if (*idx as usize) >= bc.destruct_object_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid destruct_object_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.destruct_object_payloads.len()
                        )));
                    }
                }
                // ── CROSS-2c: 7 call-family variants share `call_payloads`.
                //    Kural 7c: a bad idx is a compiler bug, not a runtime
                //    fallback path.
                Instruction::NewInstance {
                    payload_idx: idx, ..
                }
                | Instruction::MakeGenerator {
                    payload_idx: idx, ..
                } => {
                    if (*idx as usize) >= bc.call_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid call_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.call_payloads.len()
                        )));
                    }
                }
                Instruction::Spawn { payload_idx, .. } => {
                    if (*payload_idx as usize) >= bc.call_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid call_payload index {} at instruction {} in {}; pool size is {}",
                            payload_idx, ip, label, bc.call_payloads.len()
                        )));
                    }
                }
                Instruction::TailCall { .. }
                | Instruction::MethodCall { .. }
                | Instruction::SuperCall { .. } => {
                    // These have validated payload indices at runtime; skip here
                }
                // ── CROSS-2d: 3 two-symbol variants share `two_sym_payloads`.
                Instruction::MatchVariant(idx) | Instruction::GetStatic(idx) => {
                    if (*idx as usize) >= bc.two_sym_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid two_sym_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.two_sym_payloads.len()
                        )));
                    }
                }
                Instruction::DeclStore { payload_idx, .. } => {
                    if (*payload_idx as usize) >= bc.two_sym_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid two_sym_payload index {} at instruction {} in {}; pool size is {}",
                            payload_idx, ip, label, bc.two_sym_payloads.len()
                        )));
                    }
                }
                // ── CROSS-2d: 3 optional-symbol variants share `opt_sym_payloads`.
                Instruction::Remember { store_idx: idx, .. }
                | Instruction::Recall { store_idx: idx, .. }
                | Instruction::Forget { store_idx: idx, .. } => {
                    if (*idx as usize) >= bc.opt_sym_payloads.len() {
                        return Err(compile_codes::runtime_error(format!(
                            "Invalid opt_sym_payload index {} at instruction {} in {}; pool size is {}",
                            idx, ip, label, bc.opt_sym_payloads.len()
                        )));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
