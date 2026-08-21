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

/// Try fusing LoadIntConst patterns at position `i`.
pub(super) fn try_fuse_int_patterns(
    instructions: &mut Vec<Instruction>,
    int_constants: &[i64],
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: &mut usize,
    protected_below: u8,
) -> bool {
    let idx = *i;
    let (const_dst, const_idx) = match instructions[idx] {
        Instruction::LoadIntConst {
            dst: const_dst,
            const_idx,
        } => (const_dst, const_idx),
        _ => return false,
    };

    let const_val = int_constants.get(const_idx as usize).copied();
    let next_instr = &instructions[idx + 1];

    // Identity / fold cases
    match next_instr {
        Instruction::IntDiv { dst, src1, src2 } if *src2 == const_dst && const_val == Some(1) => {
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
        Instruction::IntMul { dst, src1, src2 } if *src2 == const_dst && const_val == Some(1) => {
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
        Instruction::IntMul { dst, src1, src2 } if *src2 == const_dst && const_val == Some(-1) => {
            return apply_fused(
                instructions,
                loop_payloads,
                source_positions,
                i,
                const_dst,
                protected_below,
                Instruction::Neg {
                    dst: *dst,
                    src: *src1,
                },
            );
        }
        Instruction::IntDiv { dst, src1, src2 } if *src2 == const_dst && const_val == Some(-1) => {
            return apply_fused(
                instructions,
                loop_payloads,
                source_positions,
                i,
                const_dst,
                protected_below,
                Instruction::Neg {
                    dst: *dst,
                    src: *src1,
                },
            );
        }
        Instruction::IntMul { dst, src1: _, src2 }
            if *src2 == const_dst && const_val == Some(0) =>
        {
            return apply_fused(
                instructions,
                loop_payloads,
                source_positions,
                i,
                const_dst,
                protected_below,
                Instruction::LoadIntConst {
                    dst: *dst,
                    const_idx,
                },
            );
        }
        Instruction::IntAdd { dst, src1, src2 } if *src2 == const_dst && const_val == Some(0) => {
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
        Instruction::IntSub { dst, src1, src2 } if *src2 == const_dst && const_val == Some(0) => {
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

    // Immediate fusion cases
    if let Some(val) = const_val {
        let imm = val as i16;
        if imm as i64 == val {
            match next_instr {
                Instruction::IntCmp {
                    dst: cmp_dst,
                    src1,
                    src2,
                    op,
                } if *src2 == const_dst => {
                    return apply_fused(
                        instructions,
                        loop_payloads,
                        source_positions,
                        i,
                        const_dst,
                        protected_below,
                        Instruction::IntCmpI {
                            dst: *cmp_dst,
                            src: *src1,
                            imm,
                            op: *op,
                        },
                    );
                }
                Instruction::IntDiv { dst, src1, src2 } if *src2 == const_dst && imm != 0 => {
                    return apply_fused(
                        instructions,
                        loop_payloads,
                        source_positions,
                        i,
                        const_dst,
                        protected_below,
                        Instruction::IntDivI {
                            dst: *dst,
                            src: *src1,
                            imm,
                        },
                    );
                }
                Instruction::IntMod { dst, src1, src2 } if *src2 == const_dst && imm != 0 => {
                    return apply_fused(
                        instructions,
                        loop_payloads,
                        source_positions,
                        i,
                        const_dst,
                        protected_below,
                        Instruction::IntModI {
                            dst: *dst,
                            src: *src1,
                            imm,
                        },
                    );
                }
                _ => {}
            }
        }
    }
    false
}
