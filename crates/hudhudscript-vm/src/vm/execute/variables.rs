#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_variables(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        match instr {
            Instruction::LoadGlobal { dst, sym } => {
                let sym_id = *sym as u32;
                let sym_key = hudhudscript_bytecode::interner::SymbolId(sym_id);
                // ISSUE-2a: SymbolId lookup avoids resolve_symbol String alloc.
                // ISSUE-2e-C/D: top-level symbols have a single home determined at
                // compile time.  Main-only symbols live in their absolute register
                // slot; shared top-level symbols live in globals.  Symbols outside
                // the main frame fall back to the cell/local/globals search.
                let value = if let Some(encoded) = self.main_slot_encoded(sym_id) {
                    let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                    if is_shared {
                        let idx = crate::vm::VM::main_slot_shared_index(encoded);
                        debug_assert!(idx < self.shared_globals_vec.len());
                        self.shared_globals_vec[idx]
                    } else if slot != usize::MAX {
                        self.registers.get_absolute(slot)
                    } else {
                        self.get_var_cloned_by_sym(sym_id).ok_or_else(|| {
                            let name = bytecode.resolve_symbol(sym_id);
                            Self::runtime_error_with_pos(
                                format!("Undefined variable: {}", name),
                                bytecode,
                                ip,
                            )
                        })?
                    }
                } else {
                    self.get_var_cloned_by_sym(sym_id).ok_or_else(|| {
                        let name = bytecode.resolve_symbol(sym_id);
                        Self::runtime_error_with_pos(
                            format!("Undefined variable: {}", name),
                            bytecode,
                            ip,
                        )
                    })?
                };
                self.registers[*dst as usize] = value;
            }
            Instruction::StoreGlobalConst { sym, const_idx } => {
                let sid = hudhudscript_bytecode::interner::SymbolId(*sym as u32);
                let val = ctx.constants[*const_idx as usize];
                if let Some(encoded) = self.main_slot_encoded(sid.0) {
                    let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                    if is_shared {
                        let idx = crate::vm::VM::main_slot_shared_index(encoded);
                        if idx < self.shared_globals_vec.len() {
                            self.shared_globals_vec[idx] = val;
                        }
                    } else if slot != usize::MAX {
                        self.registers.set_absolute(slot, val);
                    } else {
                        self.globals.insert(sid, val);
                    }
                } else {
                    self.globals.insert(sid, val);
                }
            }
            Instruction::StoreGlobal { src, sym } => {
                let sym_key = hudhudscript_bytecode::interner::SymbolId(*sym as u32);
                let value = self.registers[*src as usize];
                // 2d/2e: top-level global symbols are never promoted to cells
                // (decl_function.rs excludes top_level_names from captures), so
                // the hot cell-lookup path is only taken for non-top-level names
                // that the compiler emits as StoreGlobal when they are not local
                // regs (e.g. captured variables in nested functions).
                if let Some(encoded) = self.main_slot_encoded(sym_key.0) {
                    // 2e-E: single-storage routing.  Main-only symbols live only
                    // in their absolute register slot; shared symbols live only in
                    // globals.  No slot mirror for shared symbols.
                    let (slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                    if is_shared {
                        let idx = crate::vm::VM::main_slot_shared_index(encoded);
                        if idx < self.shared_globals_vec.len() {
                            self.shared_globals_vec[idx] = value;
                        }
                        if value.is_function() {
                            if let Some(entry) = self.call_cache.get_mut(sym_key.0 as usize) {
                                *entry = None;
                            }
                        }
                    } else if slot != usize::MAX {
                        self.registers.set_absolute(slot, value);
                    }
                } else {
                    // T5.2 parity: `this` single source is VM.cur_this field
                    // (same special case as set_var_by_sym). Compiler emits
                    // `this.f = v` writeback as StoreGlobal.
                    if sym_key.0 == self.this_sym {
                        self.cur_this = value;
                        return Ok(StepAction::Advance);
                    }
                    let name = bytecode.resolve_symbol(sym_key.0);
                    if let Some(cell) = self.find_cell_by_sym(sym_key.0) {
                        *cell.write() = value;
                    } else {
                        self.globals.insert(sym_key, value);
                        if value.is_function() {
                            if let Some(entry) = self.call_cache.get_mut(sym_key.0 as usize) {
                                *entry = None;
                            }
                        }
                    }
                }
            }
            Instruction::DeclGlobal { src, sym } => {
                let sym_key = hudhudscript_bytecode::interner::SymbolId(*sym as u32);
                let value = self.registers[*src as usize];
                // 2e-E: same single-storage split as StoreGlobal.
                if let Some(enc) = self.main_slot_encoded(sym_key.0) {
                    let (slot, is_shared) = crate::vm::VM::main_slot_decode(enc);
                    if is_shared {
                        let idx = crate::vm::VM::main_slot_shared_index(enc);
                        if idx < self.shared_globals_vec.len() {
                            self.shared_globals_vec[idx] = value;
                        }
                    } else if slot != usize::MAX {
                        self.registers.set_absolute(slot, value);
                    }
                } else {
                    // Plain global (not in main_local_slots) — canonical home is globals HashMap.
                    self.globals.insert(sym_key, value);
                }
            }
            Instruction::LoadClosureSlot { dst, slot } => {
                // G5-slotvec: O(1) cell read — no hash, no sym lookup
                let slot_idx = *slot as usize;
                if let Some((cells, _)) = self.scope_cells.last() {
                    if slot_idx < cells.len() {
                        if let Some(cell) = &cells[slot_idx] {
                            let val = *cell.read();
                            self.registers[*dst as usize] = val;
                        }
                    }
                }
            }
            Instruction::StoreClosureSlot { src, slot } => {
                // G5-slotvec: O(1) cell write
                let slot_idx = *slot as usize;
                let value = self.registers[*src as usize];
                if let Some((cells, _)) = self.scope_cells.last_mut() {
                    if slot_idx < cells.len() {
                        if let Some(cell) = &cells[slot_idx] {
                            *cell.write() = value;
                        }
                    }
                }
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
