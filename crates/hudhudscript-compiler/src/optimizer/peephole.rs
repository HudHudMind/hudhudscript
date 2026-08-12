//! Peephole optimizer: register-based simple patterns.

use crate::optimizer::utils::adjust_jumps_after_remove_full;
use hudhudscript_bytecode::{CmpJumpPayload, Instruction, LoopPayload, SuperInstrPayload};

/// Self-Move elimination + simple peephole patterns.
pub fn peephole_optimize_with_positions(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    cmp_jump_payloads: &mut [CmpJumpPayload],
    super_instr_payloads: &mut [SuperInstrPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let has_char_dispatch = instructions.iter().any(|i| matches!(i, Instruction::CharDispatch { .. }));
    if has_char_dispatch {
        return;
    }

    let mut i = 0;
    while i < instructions.len() {
        // Self-Move: Move { dst: X, src: Y } where X == Y → remove
        if let Instruction::Move { dst, src } = &instructions[i] {
            if dst == src {
                adjust_jumps_after_remove_full(instructions, loop_payloads, cmp_jump_payloads, super_instr_payloads, i);
                instructions.remove(i);
                if i < source_positions.len() {
                    source_positions.remove(i);
                }
                continue;
            }
        }
        
        // Not + JumpIfFalse -> JumpIfTrue
        if i + 1 < instructions.len() {
            if let (
                Instruction::Not { dst: not_dst, src: not_src },
                Instruction::JumpIfFalse { src: cond, offset }
            ) = (&instructions[i], &instructions[i + 1]) {
                if *not_dst == *cond {
                    let not_dst = *not_dst;
                    // Check if not_dst is single-use
                    let mut reused = false;
                    for instr_after in &instructions[i + 2..] {
                        if crate::optimizer::fuse_slot::writes_reg(instr_after, not_dst) {
                            break; // Overwritten, so old value is dead
                        }
                        if crate::optimizer::fuse_slot::instruction_reads_reg(instr_after, not_dst) {
                            reused = true;
                            break;
                        }
                    }
                    if !reused {
                        instructions[i] = Instruction::JumpIfTrue {
                            src: *not_src,
                            offset: *offset + 1,
                        };
                        adjust_jumps_after_remove_full(instructions, loop_payloads, cmp_jump_payloads, super_instr_payloads, i + 1);
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
