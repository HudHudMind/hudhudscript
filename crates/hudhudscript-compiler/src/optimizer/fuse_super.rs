//! Super-instruction fusion pass — register-based peephole optimizer.

use hudhudscript_bytecode::{Instruction, LoopPayload, CallPayload, SuperInstrPayload};

pub fn fuse_super_instructions_with_positions(
    instructions: &mut Vec<Instruction>,
    _loop_payloads: &mut [LoopPayload],
    _call_payloads: &[CallPayload],
    super_instr_payloads: &mut Vec<SuperInstrPayload>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        match (&instructions[i], &instructions[i + 1]) {
            (
                Instruction::IntSubI { dst, src, imm },
                Instruction::Call { dst: _call_dst, payload_idx, first_arg, arg_count },
            ) if *dst == 1 && *first_arg == 1 && *arg_count == 1 => {
                let super_idx = super_instr_payloads.len() as u32;
                super_instr_payloads.push(SuperInstrPayload {
                    call_idx: *payload_idx as u32,
                    slot: *src as u32,
                    imm: *imm,
                    offset: 0,
                });
                instructions[i] = Instruction::IntSubCall1(super_idx);
                instructions.remove(i + 1);
                if i + 1 < source_positions.len() {
                    source_positions.remove(i + 1);
                }
                continue;
            }
            (
                Instruction::IntAddI { dst, src, imm },
                Instruction::Call { dst: _call_dst, payload_idx, first_arg, arg_count },
            ) if *dst == 1 && *first_arg == 1 && *arg_count == 1 => {
                let super_idx = super_instr_payloads.len() as u32;
                super_instr_payloads.push(SuperInstrPayload {
                    call_idx: *payload_idx as u32,
                    slot: *src as u32,
                    imm: *imm,
                    offset: 0,
                });
                instructions[i] = Instruction::IntAddCall1(super_idx);
                instructions.remove(i + 1);
                if i + 1 < source_positions.len() {
                    source_positions.remove(i + 1);
                }
                continue;
            }
            // Pattern 3: IntCmp(Le/Lt) + JumpIfFalse → disabled
            // These compare two registers but IntLeJumpIfFalse compares
            // register vs immediate. Requires 3-instruction fusion
            // (LoadIntConst + IntCmp + JumpIfFalse) to work correctly.
            // Pattern 5: IntAdd + Return → IntAddReturn
            (
                Instruction::IntAdd { dst, src1, src2 },
                Instruction::Return { src },
            ) if *dst == *src => {
                instructions[i] = Instruction::IntAddReturn { src1: *src1, src2: *src2 };
                instructions.remove(i + 1);
                if i + 1 < source_positions.len() {
                    source_positions.remove(i + 1);
                }
                continue;
            }
            // Pattern 6: IntSub + Return → IntSubReturn
            (
                Instruction::IntSub { dst, src1, src2 },
                Instruction::Return { src },
            ) if *dst == *src => {
                instructions[i] = Instruction::IntSubReturn { src1: *src1, src2: *src2 };
                instructions.remove(i + 1);
                if i + 1 < source_positions.len() {
                    source_positions.remove(i + 1);
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
}
