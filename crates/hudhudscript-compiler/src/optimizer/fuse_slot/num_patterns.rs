use super::helpers::can_eliminate_const_load;
use crate::optimizer::utils::adjust_jumps_after_remove_full;
use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Apply fused instruction replacing instruction at i or i+1 based on liveness.
#[inline(always)]
fn apply_fused(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: &mut usize,
    const_dst: u8,
    protected_below: u8,
    fused: Instruction,
) -> bool {
    let idx = *i;
    if can_eliminate_const_load(const_dst, protected_below) {
        instructions[idx] = fused;
        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], idx + 1);
        instructions.remove(idx + 1);
        if idx + 1 < source_positions.len() {
            source_positions.remove(idx + 1);
        }
        true
    } else {
        instructions[idx + 1] = fused;
        *i += 1;
        true
    }
}

/// Try fusing LoadNumConst patterns at position `i`.
pub(super) fn try_fuse_num_patterns(
    instructions: &mut Vec<Instruction>,
    numeric_constants: &[u64],
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: &mut usize,
    protected_below: u8,
) -> bool {
    let idx = *i;
    let (const_dst, const_idx) = match instructions[idx] {
        Instruction::LoadNumConst {
            dst: const_dst,
            const_idx,
        } => (const_dst, const_idx),
        _ => return false,
    };

    let Some(&bits) = numeric_constants.get(const_idx as usize) else {
        return false;
    };
    let val = f64::from_bits(bits);
    let next_instr = &instructions[idx + 1];

    // Identity / zero cases
    if (val - 0.0).abs() < f64::EPSILON {
        match next_instr {
            Instruction::NumMul { dst, src1: _, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::Move {
                        dst: *dst,
                        src: const_dst,
                    },
                );
            }
            Instruction::NumAdd { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    },
                );
            }
            Instruction::NumSub { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    },
                );
            }
            _ => {}
        }
    }

    // Identity one cases
    if (val - 1.0).abs() < f64::EPSILON {
        match next_instr {
            Instruction::NumMul { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    },
                );
            }
            Instruction::NumDiv { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    },
                );
            }
            _ => {}
        }
    }

    // Fuse: LoadNumConst + NumAdd/Sub/Mul/Div → Num*I (small int values)
    let int_val = val as i16;
    if (val - int_val as f64).abs() < f64::EPSILON && int_val as f64 == val {
        match next_instr {
            Instruction::NumAdd { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::NumAddI {
                        dst: *dst,
                        src: *src1,
                        imm: int_val,
                    },
                );
            }
            Instruction::NumSub { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::NumSubI {
                        dst: *dst,
                        src: *src1,
                        imm: int_val,
                    },
                );
            }
            Instruction::NumMul { dst, src1, src2 } if *src2 == const_dst => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::NumMulI {
                        dst: *dst,
                        src: *src1,
                        imm: int_val,
                    },
                );
            }
            Instruction::NumDiv { dst, src1, src2 } if *src2 == const_dst && int_val != 0 => {
                return apply_fused(
                    instructions,
                    loop_payloads,
                    source_positions,
                    i,
                    const_dst,
                    protected_below,
                    Instruction::NumDivI {
                        dst: *dst,
                        src: *src1,
                        imm: int_val,
                    },
                );
            }
            _ => {}
        }
    }

    false
}
