//! Peephole optimizer: register-based simple patterns.

use crate::optimizer::utils::adjust_jumps_after_remove;
use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Self-Move elimination + simple peephole patterns.
pub fn peephole_optimize_with_positions(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i < instructions.len() {
        // Self-Move: Move { dst: X, src: Y } where X == Y → remove
        if let Instruction::Move { dst, src } = &instructions[i] {
            if dst == src {
                adjust_jumps_after_remove(instructions, loop_payloads, i);
                instructions.remove(i);
                if i < source_positions.len() {
                    source_positions.remove(i);
                }
                continue;
            }
        }
        i += 1;
    }
}
