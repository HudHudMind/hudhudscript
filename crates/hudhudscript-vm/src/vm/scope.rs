use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Value16;
use parking_lot::RwLock;
use std::sync::Arc;

impl VM {
    /// Resolve a SymbolId to a String via the interner, caching the result
    /// in VM::sym_cache for subsequent lock-free lookups.  Hot paths
    /// (LoadGlobal/StoreGlobal/DeclGlobal) call this instead of the raw
    /// bytecode.resolve_symbol, which acquires the global interner RwLock
    /// on every invocation.  Cache miss → lock once; cache hit → no lock.
    #[inline]
    pub(crate) fn resolve_symbol_cached(&mut self, sym_id: u32, bytecode: &hudhudscript_bytecode::Bytecode) -> String {
        if let Some(cached) = self.sym_cache.get(&sym_id) {
            return cached.clone();
        }
        let resolved = bytecode.resolve_symbol(sym_id);
        self.sym_cache.insert(sym_id, resolved.clone());
        resolved
    }

    pub(crate) fn push_scope_cells(&mut self) {
        if let Some(mut map) = self.scope_cells_pool.pop() {
            map.clear();
            self.scope_cells.push(map);
        } else {
            self.scope_cells.push(rustc_hash::FxHashMap::default());
        }
    }

    pub(crate) fn pop_scope_cells(&mut self) {
        if let Some(map) = self.scope_cells.pop() {
            self.scope_cells_pool.push(map);
        }
    }

    /// Install a shared upvalue cell for `name` into the top scope.
    /// Future `get_var`/`set_var` of `name` route through this cell so the
    /// variable stays live for closures that captured it.
    ///
    /// This mirrors the interpreter, where capturing a variable keeps the
    /// `Arc<Environment>` alive; here we keep the single slot alive.
    pub(crate) fn install_cell(&mut self, name: String, cell: Arc<parking_lot::RwLock<Value16>>) {
        if let Some(top) = self.scope_cells.last_mut() {
            top.insert(name, cell);
        }
    }

    /// Find an existing cell for `name` walking the scope chain top-down.
    /// Returns `None` if the variable is not currently promoted to a cell.
    pub(crate) fn find_cell(&self, name: &str) -> Option<Arc<parking_lot::RwLock<Value16>>> {
        for cells in self.scope_cells.iter().rev() {
            if let Some(cell) = cells.get(name) {
                return Some(Arc::clone(cell));
            }
        }
        None
    }

    /// Like `find_cell` but skips the topmost scope.  Used by
    /// `call_chunk_with_captures` when deciding whether to promote a name
    /// to a cell inherited from the enclosing scope.
    pub(crate) fn find_cell_excluding_top(
        &self,
        name: &str,
    ) -> Option<Arc<parking_lot::RwLock<Value16>>> {
        if self.scope_cells.len() < 2 {
            return None;
        }
        for cells in self.scope_cells[..self.scope_cells.len() - 1].iter().rev() {
            if let Some(cell) = cells.get(name) {
                return Some(Arc::clone(cell));
            }
        }
        None
    }

    /// Get-or-create the upvalue cell for `name` in the enclosing scope that
    /// currently owns it (the nearest scope containing either a cell or a
    /// plain entry).  Used at closure-creation time (`DefineFunction` /
    /// loading a `Value::Function` literal) so that the captured cell is the
    /// SAME one that the enclosing scope continues to read/write through.
    ///
    /// Semantics:
    /// - If a cell for `name` already exists, returns it (sharing).
    /// - Otherwise locates the scope holding the plain `Value` for `name`,
    ///   promotes it to a new `Arc<RwLock<Value>>`, installs the cell in
    ///   that scope's `scope_cells`, and returns it.
    /// - Returns `None` when `name` is not bound anywhere.  Callers should
    ///   then skip adding a cell for that name (spurious/dead captures
    ///   emitted by the compiler — e.g. a `let` in a popped block scope
    ///   whose local annotation was dropped — must NOT shadow live
    ///   scope entries with a Null cell).
    pub(crate) fn upvalue_cell_for(
        &mut self,
        name: &str,
    ) -> Option<Arc<parking_lot::RwLock<Value16>>> {
        // Dedup FIRST (S2.2c fix): if a cell for this name already exists
        // anywhere in the enclosing scope chain, reuse it so sibling
        // closures share the SAME upvalue. Previously the slot-path below
        // always constructed a new cell, giving each closure its own
        // `Arc<RwLock<Value>>` — breaking closure-shared-state semantics.
        if let Some(cell) = self.find_cell(name) {
            return Some(cell);
        }
        // PERF-1: check the current frame's slot-based locals FIRST. Since
        // params and local `let` bindings live exclusively in registers
        // (no HashMap insert), closure capture must find them here.
        // Gated on non-empty local_syms to skip the interner lock when
        // called from the main script (global scope).
        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                let sym_id = hudhudscript_bytecode::interner::try_resolve_id(name)
                    .unwrap_or_else(|| hudhudscript_bytecode::interner::intern(name).0);
                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                    let slot = local_syms[idx].1 as i32;
                    if slot >= 0 {
                        let value = self.registers[slot as usize];
                        let cell = Arc::new(parking_lot::RwLock::new(value));
                        if let Some(top) = self.scope_cells.last_mut() {
                            top.insert(name.to_string(), Arc::clone(&cell));
                        }
                        return Some(cell);
                    }
                }
            }
        }
        // Walk scope_cells from innermost to outermost.
        for idx in (0..self.scope_cells.len()).rev() {
            if let Some(cell) = self.scope_cells[idx].get(name) {
                return Some(Arc::clone(cell));
            }
        }
        // No cell found — if the name has a local slot, create a cell
        // from the current live value and install it.
        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                let sym_id = hudhudscript_bytecode::interner::intern(name).0;
                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                    let slot = local_syms[idx].1 as i32;
                    if slot >= 0 {
                        let value = self.registers[slot as usize];
                        let cell = Arc::new(parking_lot::RwLock::new(value));
                        if let Some(top) = self.scope_cells.last_mut() {
                            top.insert(name.to_string(), Arc::clone(&cell));
                        }
                        return Some(cell);
                    }
                }
            }
        }
        None
    }

    /// Look up a variable by SymbolId, bypassing the name→SymbolId conversion.
    /// Mirrors `get_var` but uses the already-resolved symbol.  Checks current
    /// frame slot-based locals first, then globals.  Does NOT resolve upvalue
    /// cells; callers that need the cell value should use `get_var_cloned_by_sym`.
    /// ISSUE-2e-C: main-frame-only top-level symbols are read from their
    /// absolute slot so the value is visible regardless of the active frame base.
    pub(crate) fn get_var_by_sym(&self, sym_id: u32) -> Option<&Value16> {
        // Main-frame top-level path: main-only reads from absolute slot,
        // shared falls through to globals (canonical home per 2e-E).
        if let Some(encoded) = self.main_slot_encoded(sym_id) {
            let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
            if !is_shared && slot != usize::MAX {
                return Some(self.registers.get_absolute_ref(slot));
            }
            // is_shared: skip local_syms, fall to globals
        } else if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            // Function-local relative slot path (non-top-level symbols).
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                    let slot = local_syms[idx].1 as i32;
                    if slot >= 0 {
                        return Some(&self.registers[slot as usize]);
                    }
                }
            }
        }
        self.globals
            .get(&hudhudscript_bytecode::interner::SymbolId(sym_id))
    }

    /// Cloned variant of `get_var_by_sym`.  Prefers a promoted upvalue cell,
    /// then current-frame locals, then globals — all keyed by SymbolId.
    /// ISSUE-2e-C: main-frame-only top-level symbols use their absolute slot.
    /// P0-D: main_local_slots check BEFORE interner::resolve — main-only reads
    /// skip the RwLock entirely; shared reads skip the cell path (globals is
    /// canonical per 2e-E).
    pub(crate) fn get_var_cloned_by_sym(&self, sym_id: u32) -> Option<Value16> {
        // Main-frame top-level hot path FIRST (no interner lock).
        if let Some(encoded) = self.main_slot_encoded(sym_id) {
            let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
            if !is_shared && slot != usize::MAX {
                return Some(self.registers.get_absolute(slot));
            }
            // is_shared: skip cell/local_syms — go directly to shared_globals_vec.
            let idx = crate::vm::VM::main_slot_shared_index(encoded);
            return self.shared_globals_vec.get(idx).copied();
        }
        // Function-local relative slot path (non-top-level).
        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                    let slot = local_syms[idx].1 as i32;
                    if slot >= 0 {
                        return Some(self.registers[slot as usize]);
                    }
                }
            }
        }
        // Cell path — COLD: needs interner.resolve (RwLock) to get the name.
        // Only non-top-level symbols with upvalue cells reach this.
        let name = hudhudscript_bytecode::interner::resolve(
            hudhudscript_bytecode::interner::SymbolId(sym_id),
        );
        if !name.is_empty() {
            if let Some(cell) = self.find_cell(&name) {
                return Some(cell.read().clone());
            }
        }
        self.globals
            .get(&hudhudscript_bytecode::interner::SymbolId(sym_id))
            .cloned()
    }

    /// Look up a variable by walking the scope stack from top to bottom.
    /// Also checks the current frame's slot-based locals (params / let-bound
    /// registers) via `call_stack_local_syms`, but does NOT resolve upvalue
    /// cells (because the return is a borrow into the scope / registers).
    /// Hot paths that need the effective cell value should call
    /// [`get_var_cloned`], which prefers any installed cell via [`find_cell`].
    pub(crate) fn get_var(&self, name: &str) -> Option<&Value16> {
        // P0-A: post-execution path — when all call frames are torn down,
        // top-level main-only symbols still live in their absolute register
        // slot and shared top-level symbols live in globals (canonical home).
        if self.call_stack_local_syms.is_empty() {
            let sym_id = hudhudscript_bytecode::interner::try_resolve_id(name)?;
            if let Some(encoded) = self.main_slot_encoded(sym_id) {
                let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                if !is_shared && slot != usize::MAX {
                    return Some(self.registers.get_absolute_ref(slot));
                }
                if is_shared {
                    let idx = crate::vm::VM::main_slot_shared_index(encoded);
                    return self.shared_globals_vec.get(idx);
                }
            }
            // Names outside main_local_slots may still be pure globals
            // (builtins injected by the host, etc.).
            return self.globals.get(&hudhudscript_bytecode::interner::SymbolId(sym_id));
        }
        // Fast path: current frame's register-based locals.  This avoids the
        // clone + cell checks in get_var_cloned for the ack/collatz hot read
        // path (ISSUE-9a).
        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                let sym_id = {
                    let cache = self.name_sym_cache.borrow();
                    let cached = cache.get(name).copied();
                    drop(cache);
                    match cached {
                        Some(id) => Some(id),
                        None => {
                            hudhudscript_bytecode::interner::try_resolve_id(name).and_then(|id| {
                                self.name_sym_cache
                                    .borrow_mut()
                                    .insert(name.to_string(), id);
                                Some(id)
                            })
                        }
                    }
                };
                if let Some(sym_id) = sym_id {
                    // ISSUE-2e-C: main-frame-only top-level symbols live in an
                    // absolute slot that is independent of the current frame base.
                    // ISSUE-2e-optimize: shared top-level symbols live in shared_globals_vec
                    // (2e-E).  Skip local_syms to avoid reading a stale slot.
                    if let Some(encoded) = self.main_slot_encoded(sym_id) {
                        let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                        if !is_shared && slot != usize::MAX {
                            return Some(self.registers.get_absolute_ref(slot));
                        }
                        if is_shared {
                            let idx = crate::vm::VM::main_slot_shared_index(encoded);
                            return self.shared_globals_vec.get(idx);
                        }
                        // is_shared: skip local_syms, read from shared_globals_vec
                    } else if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s)
                    {
                        let slot = local_syms[idx].1 as i32;
                        if slot >= 0 {
                            return Some(&self.registers[slot as usize]);
                        }
                    }
                }
            }
        }
        let sym = hudhudscript_bytecode::interner::try_resolve_id(name)?;
        self.globals
            .get(&hudhudscript_bytecode::interner::SymbolId(sym))
    }

    /// Look up a variable (cloned). Prefers a promoted upvalue cell (shared
    /// between closures that captured this name) over the plain scope entry.
    ///
    /// PERF-1: also checks the current frame's slot-based locals via
    /// `call_stack_local_syms`. Params and local `let` bindings live in
    /// registers; closure-capture and call-target resolution paths use
    /// this helper and must see those values.
    /// The lookup is gated on non-empty local_syms so the main-script
    /// (global-scope) hot path avoids the interner Mutex lock.
    pub(crate) fn get_var_cloned(&self, name: &str) -> Option<Value16> {
        if let Some(cell) = self.find_cell(name) {
            return Some(cell.read().clone());
        }
        // REGRESSION FIX: empty-frame path — top-level main-only/shared symbols
        // must be read from register/Vec via main_slot, same as get_var does.
        if self.call_stack_local_syms.is_empty() {
            let sym_id = hudhudscript_bytecode::interner::try_resolve_id(name)?;
            if let Some(encoded) = self.main_slot_encoded(sym_id) {
                let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                if !is_shared && slot != usize::MAX {
                    return Some(self.registers.get_absolute(slot));
                }
                if is_shared {
                    let idx = crate::vm::VM::main_slot_shared_index(encoded);
                    return self.shared_globals_vec.get(idx).copied();
                }
            }
            return self.globals.get(&hudhudscript_bytecode::interner::SymbolId(sym_id)).cloned();
        }
        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
            let local_syms = unsafe { &**local_syms_ptr };
            if !local_syms.is_empty() {
                // ISSUE-007: use try_resolve_id (READ lock) instead of
                // intern (WRITE lock). If symbol not in interner yet,
                // fall through to globals.
                let sym_id = {
                    let cache = self.name_sym_cache.borrow();
                    let cached = cache.get(name).copied();
                    drop(cache);
                    match cached {
                        Some(id) => Some(id),
                        None => {
                            let id = hudhudscript_bytecode::interner::try_resolve_id(name)?;
                            self.name_sym_cache
                                .borrow_mut()
                                .insert(name.to_string(), id);
                            Some(id)
                        }
                    }
                };
                if let Some(sym_id) = sym_id {
                    // ISSUE-2e-C: main-frame-only top-level symbols use their
                    // absolute slot so call dispatch and other name-based lookups
                    // see the same value regardless of the active frame base.
                    if let Some(encoded) = self.main_slot_encoded(sym_id) {
                        let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                        if !is_shared && slot != usize::MAX {
                            return Some(self.registers.get_absolute(slot));
                        }
                        if is_shared {
                            let idx = crate::vm::VM::main_slot_shared_index(encoded);
                            return self.shared_globals_vec.get(idx).copied();
                        }
                        // is_shared: skip local_syms, read from shared_globals_vec
                    } else if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s)
                    {
                        let slot = local_syms[idx].1 as i32;
                        if slot >= 0 {
                            return Some(self.registers[slot as usize]);
                        }
                    }
                }
            }
        }
        let sym = hudhudscript_bytecode::interner::try_resolve_id(name)?;
        self.globals
            .get(&hudhudscript_bytecode::interner::SymbolId(sym))
            .cloned()
    }
}
