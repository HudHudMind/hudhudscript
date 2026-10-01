// P3: Register/constant remapping helpers for the function inliner.
// Extracted from inline_compile.rs (Anayasa: dosya boyutu sınırı — 400 satır).

use hudhudscript_bytecode::Instruction;

/// Check that every register operand in `instr` can be mapped without overflow.
pub(super) fn can_remap_regs<F: Fn(u8) -> Option<u8>>(instr: &Instruction, m: &F) -> bool {
    match *instr {
        Instruction::Move { dst, src } => m(dst).is_some() && m(src).is_some(),
        Instruction::LoadConst { dst, .. }
        | Instruction::LoadIntConst { dst, .. }
        | Instruction::LoadNumConst { dst, .. } => m(dst).is_some(),
        Instruction::IntAdd { dst, src1, src2 }
        | Instruction::IntMul { dst, src1, src2 }
        | Instruction::IntSub { dst, src1, src2 }
        | Instruction::IntDiv { dst, src1, src2 }
        | Instruction::NumAdd { dst, src1, src2 }
        | Instruction::NumSub { dst, src1, src2 }
        | Instruction::NumMul { dst, src1, src2 }
        | Instruction::NumDiv { dst, src1, src2 } => {
            m(dst).is_some() && m(src1).is_some() && m(src2).is_some()
        }
        Instruction::IntCmp {
            dst, src1, src2, ..
        } => m(dst).is_some() && m(src1).is_some() && m(src2).is_some(),
        Instruction::IntCmpI { dst, src, .. }
        | Instruction::NumAddI { dst, src, .. }
        | Instruction::IntAddI { dst, src, .. }
        | Instruction::IntMulI { dst, src, .. }
        | Instruction::IntDivI { dst, src, .. }
        | Instruction::NumDivI { dst, src, .. }
        | Instruction::Neg { dst, src }
        | Instruction::Not { dst, src } => m(dst).is_some() && m(src).is_some(),
        Instruction::JumpIfFalse { src, .. } | Instruction::JumpIfTrue { src, .. } => {
            m(src).is_some()
        }
        Instruction::Return { src } => m(src).is_some(),
        _ => false,
    }
}

/// Remap a single instruction's register operands and constant index.
pub(super) fn remap_single_instr<F: Fn(u8) -> Option<u8>>(
    instr: &Instruction,
    m: &F,
    const_remap: &[u16],
    int_remap: &[u16],
    num_remap: &[u16],
) -> Option<Instruction> {
    let rm = |r: u8| m(r);
    Some(match *instr {
        Instruction::Move { dst, src } => Instruction::Move {
            dst: rm(dst)?,
            src: rm(src)?,
        },
        Instruction::LoadConst { dst, const_idx } => {
            let new_idx = const_remap
                .get(const_idx as usize)
                .copied()
                .unwrap_or(const_idx);
            Instruction::LoadConst {
                dst: rm(dst)?,
                const_idx: new_idx,
            }
        }
        Instruction::LoadIntConst { dst, const_idx } => {
            let new_idx = int_remap
                .get(const_idx as usize)
                .copied()
                .unwrap_or(const_idx);
            Instruction::LoadIntConst {
                dst: rm(dst)?,
                const_idx: new_idx,
            }
        }
        Instruction::LoadNumConst { dst, const_idx } => {
            let new_idx = num_remap
                .get(const_idx as usize)
                .copied()
                .unwrap_or(const_idx);
            Instruction::LoadNumConst {
                dst: rm(dst)?,
                const_idx: new_idx,
            }
        }
        Instruction::IntAdd { dst, src1, src2 } => Instruction::IntAdd {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::IntMul { dst, src1, src2 } => Instruction::IntMul {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::IntSub { dst, src1, src2 } => Instruction::IntSub {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::IntDiv { dst, src1, src2 } => Instruction::IntDiv {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::IntCmp {
            dst,
            src1,
            src2,
            op,
        } => Instruction::IntCmp {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
            op,
        },
        Instruction::IntCmpI { dst, src, op, imm } => Instruction::IntCmpI {
            dst: rm(dst)?,
            src: rm(src)?,
            op,
            imm,
        },
        Instruction::NumAdd { dst, src1, src2 } => Instruction::NumAdd {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::NumSub { dst, src1, src2 } => Instruction::NumSub {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::NumMul { dst, src1, src2 } => Instruction::NumMul {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::NumDiv { dst, src1, src2 } => Instruction::NumDiv {
            dst: rm(dst)?,
            src1: rm(src1)?,
            src2: rm(src2)?,
        },
        Instruction::NumAddI { dst, src, imm } => Instruction::NumAddI {
            dst: rm(dst)?,
            src: rm(src)?,
            imm,
        },
        Instruction::IntAddI { dst, src, imm } => Instruction::IntAddI {
            dst: rm(dst)?,
            src: rm(src)?,
            imm,
        },
        Instruction::IntMulI { dst, src, imm } => Instruction::IntMulI {
            dst: rm(dst)?,
            src: rm(src)?,
            imm,
        },
        Instruction::IntDivI { dst, src, imm } => Instruction::IntDivI {
            dst: rm(dst)?,
            src: rm(src)?,
            imm,
        },
        Instruction::NumDivI { dst, src, imm } => Instruction::NumDivI {
            dst: rm(dst)?,
            src: rm(src)?,
            imm,
        },
        Instruction::Neg { dst, src } => Instruction::Neg {
            dst: rm(dst)?,
            src: rm(src)?,
        },
        Instruction::Not { dst, src } => Instruction::Not {
            dst: rm(dst)?,
            src: rm(src)?,
        },
        Instruction::JumpIfFalse { src, offset } => Instruction::JumpIfFalse {
            src: rm(src)?,
            offset,
        },
        Instruction::JumpIfTrue { src, offset } => Instruction::JumpIfTrue {
            src: rm(src)?,
            offset,
        },
        Instruction::Return { src } => Instruction::Move {
            dst: 255,
            src: rm(src)?,
        },
        _ => return None,
    })
}
