//! Slot+immediate fusion pass — re-enabled with jump offset correction (v0.4.468).

use hudhudscript_bytecode::{Instruction, LoopPayload};

mod helpers;
mod int_patterns;
mod num_patterns;
mod refuse_patterns;

pub use helpers::*;
use int_patterns::try_fuse_int_patterns;
use num_patterns::try_fuse_num_patterns;
use refuse_patterns::try_refuse_patterns;

use crate::optimizer::utils::adjust_jumps_after_remove_full;

/// Legacy signature kept for compatibility.
pub fn fuse_slot_immediate(_instructions: &mut Vec<Instruction>, _loop_payloads: &mut [LoopPayload]) {
    // No-op
}

/// Fuse IntModI + IntCmpI → IntModCmpI in a separate pass.
/// This must run AFTER `fuse_slot_immediate_with_positions` which creates
/// IntModI and IntCmpI from LoadIntConst+IntMod/IntCmp patterns.
pub fn fuse_intmodcmpi_chain(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        if let Instruction::IntModI {
            dst: mod_dst,
            src: mod_src,
            imm: mod_imm,
        } = &instructions[i]
        {
            if let Instruction::IntCmpI {
                dst: cmp_dst,
                src: cmp_src,
                imm: cmp_imm,
                op,
            } = &instructions[i + 1]
            {
                if *cmp_src == *mod_dst {
                    let mut reused = false;
                    for instr_after in &instructions[i + 2..] {
                        if writes_reg(instr_after, *mod_dst) {
                            break;
                        }
                        if instruction_reads_reg(instr_after, *mod_dst) {
                            reused = true;
                            break;
                        }
                    }
                    if !reused {
                        instructions[i] = Instruction::IntModCmpI {
                            dst: *cmp_dst,
                            src: *mod_src,
                            mod_imm: *mod_imm,
                            cmp_imm: *cmp_imm,
                            op: *op,
                        };
                        adjust_jumps_after_remove_full(
                            instructions,
                            loop_payloads,
                            &mut [],
                            &mut [],
                            i + 1,
                        );
                        instructions.remove(i + 1);
                        if i + 1 < source_positions.len() {
                            source_positions.remove(i + 1);
                        }
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

/// Fuses constant loads with subsequent binary operations into immediate/fused instructions.
/// Respects `protected_below` so that local variable registers are never clobbered or uninitialized.
pub fn fuse_slot_immediate_with_positions(
    instructions: &mut Vec<Instruction>,
    numeric_constants: &[u64],
    int_constants: &[i64],
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    protected_below: u8,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        if try_fuse_int_patterns(
            instructions,
            int_constants,
            loop_payloads,
            source_positions,
            &mut i,
            protected_below,
        ) {
            continue;
        }

        if try_fuse_num_patterns(
            instructions,
            numeric_constants,
            loop_payloads,
            source_positions,
            &mut i,
            protected_below,
        ) {
            continue;
        }

        if try_refuse_patterns(
            instructions,
            loop_payloads,
            source_positions,
            &mut i,
        ) {
            continue;
        }

        i += 1;
    }
}
