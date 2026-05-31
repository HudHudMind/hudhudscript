use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Value16;
use parking_lot::RwLock;
use std::sync::Arc;

impl VM {
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

    /// Look up a variable by walking the scope stack from top to bottom.
    ///
    /// NOTE: This does NOT resolve upvalue cells (because the return is a
    /// borrow into the scope HashMap).  Hot paths that need the effective
    /// value should call [`get_var_cloned`], which prefers any installed
    /// cell via [`find_cell`].
    pub(crate) fn get_var(&self, name: &str) -> Option<&Value16> {
        self.globals.get(name)
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
                            self.name_sym_cache.borrow_mut().insert(name.to_string(), id);
                            Some(id)
                        }
                    }
                };
                if let Some(sym_id) = sym_id {
                    if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                        let slot = local_syms[idx].1 as i32;
                        if slot >= 0 {
                            return Some(self.registers[slot as usize]);
                        }
                    }
                }
            }
        }
        self.globals.get(name).cloned()
    }
}
