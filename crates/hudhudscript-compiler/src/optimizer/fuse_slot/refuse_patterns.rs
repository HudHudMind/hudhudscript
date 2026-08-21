use crate::optimizer::utils::adjust_jumps_after_remove_full;
use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Try applying post-fusion simplifications and self-update refuse patterns.
pub(super) fn try_refuse_patterns(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: &mut usize,
) -> bool {
    let idx = *i;

    // Fold: IntAddI/IntSubI with imm=0 → Move (x+0, x-0 no-ops)
    if let Instruction::IntAddI { dst, src, imm: 0 }
    | Instruction::IntSubI { dst, src, imm: 0 } = &instructions[idx]
    {
        if *dst != *src {
            instructions[idx] = Instruction::Move {
                dst: *dst,
                src: *src,
            };
        }
    }

    if idx + 1 < instructions.len() {
        // B4: Re-fuse IntSubI { dst: A, src: B, imm } + Move { dst: B, src: A }
        //     → IntSubI { dst: B, src: B, imm } (self-update, no temp)
        if let (
            Instruction::IntSubI { dst, src, imm },
            Instruction::Move {
                dst: m_dst,
                src: m_src,
            },
        ) = (&instructions[idx], &instructions[idx + 1])
        {
            if *imm != 0 && *dst == *m_src && *src == *m_dst && *dst != *src {
                instructions[idx] = Instruction::IntSubI {
                    dst: *src,
                    src: *src,
                    imm: *imm,
                };
                adjust_jumps_after_remove_full(
                    instructions,
                    loop_payloads,
                    &mut [],
                    &mut [],
                    idx + 1,
                );
                instructions.remove(idx + 1);
                if idx + 1 < source_positions.len() {
                    source_positions.remove(idx + 1);
                }
                return true;
            }
        }

        // P2-B1: Re-fuse IntAddI { dst: A, src: B, imm } + Move { dst: B, src: A }
        //         → IntAddI { dst: B, src: B, imm } (self-update, no temp)
        if let (
            Instruction::IntAddI { dst, src, imm },
            Instruction::Move {
                dst: m_dst,
                src: m_src,
            },
        ) = (&instructions[idx], &instructions[idx + 1])
        {
            if *imm != 0 && *dst == *m_src && *src == *m_dst && *dst != *src {
                instructions[idx] = Instruction::IntAddI {
                    dst: *src,
                    src: *src,
                    imm: *imm,
                };
                adjust_jumps_after_remove_full(
                    instructions,
                    loop_payloads,
                    &mut [],
                    &mut [],
                    idx + 1,
                );
                instructions.remove(idx + 1);
                if idx + 1 < source_positions.len() {
                    source_positions.remove(idx + 1);
                }
                return true;
            }
        }
    }

    false
}
