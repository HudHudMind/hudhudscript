use crate::{Bytecode, SymId};

impl Bytecode {
    /// Intern a symbol name, returning its index. Deduplicates: if the
    /// same string was interned before, returns the existing index.
    ///
    /// Used by the compiler when emitting `LoadVar`, `StoreVar`,
    /// `DeclVar`, `StoreConst`, and other instructions that reference
    /// variable or property names.
    pub fn intern_symbol(&mut self, name: &str) -> u32 {
        self.rebuild_indices_if_stale();
        if let Some(&idx) = self.symbol_index.get(name) {
            return idx;
        }
        let idx = self.symbols.len() as u32;
        self.symbols.push(name.to_string());
        self.symbol_index.insert(name.to_string(), idx);
        idx
    }

    /// Resolve a symbol index back to a string.
    ///
    /// Uses the global interner from shared-builtins (same interner the
    /// compiler writes to via `ct_intern`). The per-bytecode `symbols`
    /// table is kept for serialization; at runtime the global interner
    /// is the source of truth.
    pub fn resolve_symbol(&self, idx: u32) -> String {
        crate::interner::resolve(crate::interner::SymbolId(idx))
    }

    /// Store a list of symbols in the side table, returning its index
    /// (Issue #1059, P7.2).
    ///
    /// The returned `u32` can be embedded directly into a compact
    /// instruction variant in place of a `Vec<SymId>` payload.
    /// Deduplicates: if an identical list was already stored, the
    /// existing index is returned.
    pub fn add_symbol_list(&mut self, list: Vec<SymId>) -> u32 {
        // O(1) amortized dedup via reverse index (Audit v3 F2.2 ext).
        // Previous path was O(N·L): for every call, linearly scan all
        // stored lists and element-wise compare.  In a workspace compile
        // with many class/destructure patterns the list count N grows
        // large.
        self.rebuild_indices_if_stale();
        if let Some(&idx) = self.symbol_list_index.get(&list) {
            return idx;
        }
        let idx = self.symbol_lists.len() as u32;
        self.symbol_list_index.insert(list.clone(), idx);
        self.symbol_lists.push(list);
        idx
    }

    /// Retrieve a symbol list by index (Issue #1059, P7.2).
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds — this indicates a compiler bug.
    #[inline]
    pub fn get_symbol_list(&self, idx: u32) -> &[SymId] {
        &self.symbol_lists[idx as usize]
    }
}
