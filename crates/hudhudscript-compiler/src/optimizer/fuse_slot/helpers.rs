use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Check if register `reg` is modified in a loop body enclosing or near `site`.
pub(crate) fn reg_modified_in_enclosing_loop(
    instructions: &[Instruction],
    loop_payloads: &[LoopPayload],
    site: usize,
    reg: u8,
) -> bool {
    let enclosing = loop_payloads
        .iter()
        .filter(|lp| (lp.start as usize) <= site && site < (lp.end as usize))
        .min_by_key(|lp| lp.end - lp.start);
    let end = if let Some(lp) = enclosing {
        (lp.end as usize).min(instructions.len())
    } else {
        let mut loop_end = instructions.len();
        for (j, instr) in instructions.iter().enumerate().skip(site + 1) {
            let backward = match instr {
                Instruction::Jump(o) if (*o as i32) < 0 => Some((j as i32 + *o as i32) as usize),
                Instruction::IntAddIJump { offset, .. } if *offset < 0 => {
                    Some((j as i32 + *offset as i32) as usize)
                }
                Instruction::IntSubIJump { offset, .. } if *offset < 0 => {
                    Some((j as i32 + *offset as i32) as usize)
                }
                _ => None,
            };
            if let Some(target) = backward {
                if target <= site && site - target <= 5 {
                    loop_end = j + 1;
                    break;
                }
            }
        }
        loop_end
    };
    for instr in &instructions[site + 1..end] {
        match instr {
            Instruction::IntAddIJump { reg: r, .. } if *r == reg => return true,
            Instruction::IntAddI { dst, .. } if *dst == reg => return true,
            Instruction::IntSubI { dst, .. } if *dst == reg => return true,
            Instruction::Move { dst, .. } if *dst == reg => return true,
            _ => {}
        }
    }
    false
}

/// Returns true if `instr` writes to register `reg`.
pub(crate) fn writes_reg(instr: &Instruction, reg: u8) -> bool {
    use Instruction::*;
    match instr {
        Move { dst, .. } => *dst == reg,
        LoadGlobal { dst, .. }
        | LoadIntConst { dst, .. }
        | LoadConst { dst, .. }
        | LoadNumConst { dst, .. } => *dst == reg,
        IntAdd { dst, .. }
        | IntSub { dst, .. }
        | IntMul { dst, .. }
        | IntDiv { dst, .. }
        | IntMod { dst, .. } => *dst == reg,
        IntCmp { dst, .. } | IntCmpI { dst, .. } | IntModI { dst, .. } | IntModCmpI { dst, .. } => {
            *dst == reg
        }
        IntAddI { dst, .. } | IntSubI { dst, .. } | IntMulI { dst, .. } | IntDivI { dst, .. } => {
            *dst == reg
        }
        NumAdd { dst, .. } | NumSub { dst, .. } | NumMul { dst, .. } | NumDiv { dst, .. } => {
            *dst == reg
        }
        NumAddI { dst, .. } | NumSubI { dst, .. } | NumMulI { dst, .. } | NumDivI { dst, .. } => {
            *dst == reg
        }
        StrCat { dst, .. } => *dst == reg,
        Index { dst, .. } => *dst == reg,
        MethodCall { dst, .. } => *dst == reg,
        Call { dst, .. } => *dst == reg,
        _ => false,
    }
}

/// Returns true if `instr` reads register `reg` (source operand).
pub(crate) fn instruction_reads_reg(instr: &Instruction, reg: u8) -> bool {
    use Instruction::*;
    match instr {
        Move { src, .. } => *src == reg,
        IntAdd { src1, src2, .. } | IntSub { src1, src2, .. } | IntMul { src1, src2, .. } => {
            *src1 == reg || *src2 == reg
        }
        IntDiv { src1, src2, .. } | IntMod { src1, src2, .. } => *src1 == reg || *src2 == reg,
        IntCmp { src1, src2, .. } => *src1 == reg || *src2 == reg,
        IntAddI { src, .. } | IntSubI { src, .. } | IntMulI { src, .. } => *src == reg,
        IntDivI { src, .. } | IntModI { src, .. } => *src == reg,
        IntCmpI { src, .. } | IntModCmpI { src, .. } => *src == reg,
        NumAdd { src1, src2, .. }
        | NumSub { src1, src2, .. }
        | NumMul { src1, src2, .. }
        | NumDiv { src1, src2, .. } => *src1 == reg || *src2 == reg,
        NumAddI { src, .. } | NumSubI { src, .. } | NumMulI { src, .. } | NumDivI { src, .. } => {
            *src == reg
        }
        StoreGlobal { src, .. } => *src == reg,
        DeclGlobal { src, .. } => *src == reg,
        MethodCall {
            obj,
            first_arg,
            arg_count,
            ..
        } => *obj == reg || (*first_arg..*first_arg + *arg_count).any(|i| i == reg),
        Call {
            first_arg,
            arg_count,
            ..
        } => (*first_arg..*first_arg + *arg_count).any(|i| i == reg),
        StrCat { src1, src2, .. } => *src1 == reg || *src2 == reg,
        _ => false,
    }
}

/// Determines whether a constant load instruction can be completely eliminated.
///
/// Registers `< protected_below` are named local variables (e.g. `let y = 0;`), whose
/// constant initialization must be preserved when referenced in subsequent statements.
/// Registers `>= protected_below` are compiler-generated temporary scratch registers
/// allocated solely for a single binary expression, which can always be safely fused and eliminated.
#[inline(always)]
pub(crate) fn can_eliminate_const_load(const_dst: u8, protected_below: u8) -> bool {
    const_dst >= protected_below
}
