use crate::{Bytecode, Instruction, ReprTag, Value16};

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

    /// G5.2: push a Move instruction, skipping self-moves (dst == src).
    #[inline]
    pub fn push_move(&mut self, dst: u8, src: u8) {
        if dst != src {
            self.push_instr(Instruction::Move { dst, src });
        }
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
}
