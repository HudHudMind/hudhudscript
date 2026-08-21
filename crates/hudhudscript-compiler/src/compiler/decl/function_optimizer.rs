//! Per-function optimization passes with loop-payload isolation.
//!
//! BUG4b: function-local loop payloads from nested functions must not be
//! adjusted by the enclosing function's optimizer, because they refer to
//! nested instruction IPs. The helpers here split out only payloads
//! referenced by the current instruction stream, run the passes, then merge
//! the optimized values back.

use crate::bytecode::{Bytecode, FunctionChunk, Instruction, LoopPayload};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// INVARIANT (v0.8.36): optimizer passes that receive `local_mut()` may mutate
/// existing loop payloads in place but must **never** grow the pool (push/extend).
/// `restore_global` relies on this: it builds `local_to_global_map` from the
/// pre-optimizer index set and asserts that every `LoopBegin` index in the
/// post-optimizer instruction stream is found in that map. If a future pass
/// adds a new payload, the assert fires with a clear message — silent
/// corruption (the same bug class as FAZ C) is not possible.
pub struct PayloadIsolation {
    original: Vec<LoopPayload>,
    local: Vec<LoopPayload>,
    local_to_global: Vec<(u32, u32)>,
    global_to_local: HashMap<u32, u32>,
}

impl PayloadIsolation {
    pub fn new(instructions: &[Instruction], loop_payloads: Vec<LoopPayload>) -> Self {
        let referenced: HashSet<u32> = instructions
            .iter()
            .filter_map(|instr| match instr {
                Instruction::LoopBegin(idx) => Some(*idx),
                _ => None,
            })
            .collect();
        let mut local = Vec::with_capacity(referenced.len());
        let mut local_to_global = Vec::with_capacity(referenced.len());
        for (global_idx, payload) in loop_payloads.iter().enumerate() {
            let global_idx = global_idx as u32;
            if referenced.contains(&global_idx) {
                let local_idx = local.len() as u32;
                local.push(*payload);
                local_to_global.push((local_idx, global_idx));
            }
        }
        let global_to_local: HashMap<u32, u32> =
            local_to_global.iter().map(|(l, g)| (*g, *l)).collect();
        Self {
            original: loop_payloads,
            local,
            local_to_global,
            global_to_local,
        }
    }

    pub fn rewrite_to_local(&self, instructions: &mut [Instruction]) {
        for instr in instructions.iter_mut() {
            if let Instruction::LoopBegin(idx) = instr {
                if let Some(&local_idx) = self.global_to_local.get(idx) {
                    *idx = local_idx;
                }
            }
        }
    }

    pub fn restore_global(self, instructions: &mut [Instruction]) -> Vec<LoopPayload> {
        let mut final_lp = self.original;
        for (local_idx, global_idx) in self.local_to_global.iter().copied() {
            final_lp[global_idx as usize] = self.local[local_idx as usize];
        }
        // Build reverse mapping: local_idx → global_idx
        let local_to_global_map: HashMap<u32, u32> =
            self.local_to_global.iter().map(|(l, g)| (*l, *g)).collect();
        // Defensive invariant check (v0.8.36): every LoopBegin idx must be
        // found in the map. Optimizer passes must never grow the payload pool.
        #[cfg(debug_assertions)]
        {
            for instr in instructions.iter() {
                if let Instruction::LoopBegin(idx) = instr {
                    debug_assert!(
                        local_to_global_map.contains_key(idx),
                        "Compiler invariant violation: LoopBegin idx {} not in payload isolation map.\n\
                         An optimizer pass grew the loop payload pool — this is forbidden.\n\
                         See PayloadIsolation INVARIANT doc comment.",
                        idx
                    );
                }
            }
        }
        for instr in instructions.iter_mut() {
            if let Instruction::LoopBegin(idx) = instr {
                if let Some(&global_idx) = local_to_global_map.get(idx) {
                    *idx = global_idx;
                }
            }
        }
        final_lp
    }

    pub fn local_mut(&mut self) -> &mut Vec<LoopPayload> {
        &mut self.local
    }
}

/// Run the function-level optimization passes while protecting loop payloads
/// that belong to nested functions from being shifted by this function's own
/// instruction stream changes.
pub fn run_function_optimizer_passes(
    bytecode: &mut Bytecode,
    source_positions: &mut Vec<Option<(usize, usize)>>,
    protected_below: u8,
) {
    while source_positions.len() < bytecode.instructions.len() {
        source_positions.push(None);
    }
    source_positions.truncate(bytecode.instructions.len());

    let mut isolation = PayloadIsolation::new(
        &bytecode.instructions,
        std::mem::take(&mut bytecode.loop_payloads),
    );
    isolation.rewrite_to_local(&mut bytecode.instructions);

    {
        let num_consts = std::mem::take(&mut bytecode.numeric_constants);
        let int_consts = std::mem::take(&mut bytecode.int_constants);
        crate::optimizer::fuse_slot_immediate_with_positions(
            &mut bytecode.instructions,
            &num_consts,
            &int_consts,
            isolation.local_mut(),
            source_positions,
            protected_below,
        );
        crate::optimizer::fuse_intmodcmpi_chain(
            &mut bytecode.instructions,
            isolation.local_mut(),
            source_positions,
        );
        bytecode.numeric_constants = num_consts;
        bytecode.int_constants = int_consts;
    }
    crate::optimizer::fuse_super_instructions_with_positions(
        &mut bytecode.instructions,
        isolation.local_mut(),
        &bytecode.call_payloads,
        &mut bytecode.super_instr_payloads,
        source_positions,
    );

    // G5: MOVE birleştirme — İZOLASYON PENCERESİ İÇİNDE (BUG4b: restore
    // sonrası adjust iç-içe fonksiyon payload'larını kaydırıp bozar; ilk
    // deneme nested_while_true_break'i sonsuz döngüye sokmuştu).
    crate::optimizer::fuse_helpers::coalesce_moves(
        &mut bytecode.instructions,
        isolation.local_mut(),
        source_positions,
        protected_below,
    );

    bytecode.loop_payloads = isolation.restore_global(&mut bytecode.instructions);

    // G4: fonksiyon gövdesindeki cmp+branch'leri payload-tablolu packed
    // forma çevir — ana-chunk yolundaki entry.rs dönüşümünün chunk eşi
    // (Kural 7: aynı pack_cmp_jumps fonksiyonu). Payload'lar bu geçici
    // bytecode'un tablosuna girer; function_context merge'i dış tabloya
    // base-kaydırmalı taşır.
    crate::optimizer::fuse_helpers::pack_cmp_jumps(
        &mut bytecode.instructions,
        &mut bytecode.cmp_jump_payloads,
    );
}

/// Merge function-local numeric/int constant pools into the outer bytecode
/// and rewrite LoadNumConst/LoadIntConst indices in both the function body
/// and nested function chunks.
pub fn merge_function_constant_pools(
    outer_bytecode: &mut Bytecode,
    func_numeric: Vec<u64>,
    func_int: Vec<i64>,
    func_instructions: &mut [Instruction],
    func_nested: &mut [Arc<FunctionChunk>],
) {
    if !func_numeric.is_empty() {
        let mut index_map: Vec<u32> = Vec::with_capacity(func_numeric.len());
        for bits in &func_numeric {
            index_map.push(outer_bytecode.add_numeric_constant_bits(*bits));
        }
        for instr in func_instructions.iter_mut() {
            match instr {
                // G12: FConst da sayısal havuzu indeksler — remap edilmezse
                // fonksiyon-yerel indeks dış havuzda YANLIŞ sabiti okur
                // (sessiz yanlış değer; t4 probe'unda -1.0 yerine 2.0).
                Instruction::LoadNumConst { const_idx, .. }
                | Instruction::FConst { const_idx, .. } => {
                    *const_idx = index_map[*const_idx as usize] as u16;
                }
                _ => {}
            }
        }
        for chunk in func_nested.iter_mut() {
            let c = Arc::make_mut(chunk);
            for instr in c.instructions.iter_mut() {
                match instr {
                    Instruction::LoadNumConst { const_idx, .. }
                    | Instruction::FConst { const_idx, .. } => {
                        if let Some(&new_idx) = index_map.get(*const_idx as usize) {
                            *const_idx = new_idx as u16;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if !func_int.is_empty() {
        let mut index_map: Vec<u32> = Vec::with_capacity(func_int.len());
        for v in &func_int {
            index_map.push(outer_bytecode.add_int_constant(*v));
        }
        for instr in func_instructions.iter_mut() {
            if let Instruction::LoadIntConst { const_idx, .. } = instr {
                if (*const_idx as usize) < index_map.len() {
                    *const_idx = index_map[*const_idx as usize] as u16;
                }
            }
            if let Instruction::ArrayPushIntConst { const_idx, .. } = instr {
                if (*const_idx as usize) < index_map.len() {
                    *const_idx = index_map[*const_idx as usize] as u16;
                }
            }
        }
        for chunk in func_nested.iter_mut() {
            let c = Arc::make_mut(chunk);
            for instr in c.instructions.iter_mut() {
                if let Instruction::LoadIntConst { const_idx, .. } = instr {
                    if let Some(&new_idx) = index_map.get(*const_idx as usize) {
                        *const_idx = new_idx as u16;
                    }
                }
                if let Instruction::ArrayPushIntConst { const_idx, .. } = instr {
                    if let Some(&new_idx) = index_map.get(*const_idx as usize) {
                        *const_idx = new_idx as u16;
                    }
                }
            }
        }
    }
}
