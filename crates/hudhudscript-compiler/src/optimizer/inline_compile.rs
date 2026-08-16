// P3: Compiler-side function inlining helper.
// Called during compilation when the compiler encounters a Call to a known
// pure function that's already been compiled. Imports callee constants into
// the caller pool, remaps registers with checked arithmetic, and emits
// inlined instructions or returns false for normal Call fallback.

use crate::compiler::CompileTarget;
use hudhudscript_bytecode::{FunctionChunk, Instruction};

/// Try to inline a call to `callee` using `target` for constant import and
/// instruction emission. Returns `true` if inlining succeeded; `false` means
/// the caller should emit a normal `Call` instruction.
pub(crate) fn try_inline_call(
    target: &mut dyn CompileTarget,
    callee: &FunctionChunk,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
) -> bool {
    // ---- Phase 1: import callee constants into caller pool ---------------
    // B8: all three constant types need remapping — callee and caller may
    // have different pools during function compilation.
    let const_count = callee.constants.len();
    let mut const_remap: Vec<u16> = Vec::with_capacity(const_count);
    if const_count > 0 {
        for val in &callee.constants {
            let new_idx = target.ct_emit_const(*val);
            const_remap.push(new_idx as u16);
        }
    }
    // B8: remap int and num constants using global snapshots
    let ints = target.ct_int_constants().to_vec();
    let nums = target.ct_numeric_constants().to_vec();
    let mut int_remap: Vec<u16> = Vec::with_capacity(ints.len());
    for v in &ints {
        int_remap.push(target.ct_emit_int_const(*v) as u16);
    }
    let mut num_remap: Vec<u16> = Vec::with_capacity(nums.len());
    for bits in &nums {
        num_remap.push(target.ct_emit_num_const(f64::from_bits(*bits)) as u16);
    }

    // ---- Phase 2: plan (all checks, no emission) ------------------------
    if let Some(plan) = try_inline_plan(
        callee,
        first_arg,
        arg_count,
        dst,
        &const_remap,
        &int_remap,
        &num_remap,
    ) {
        // ---- Phase 3: emit atomically -----------------------------------
        for instr in plan {
            target.ct_emit(instr);
        }
        true
    } else {
        false
    }
}

/// Pure planning function: validate eligibility and build remapped
/// instruction list without modifying any target state.  Exposed as
/// `pub(crate)` so integration tests can exercise edge cases directly.
///
/// `const_remap[i]` is the caller-side index for `callee.constants[i]`.
pub fn try_inline_plan(
    callee: &FunctionChunk,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
    const_remap: &[u16],
    int_remap: &[u16],
    num_remap: &[u16],
) -> Option<Vec<Instruction>> {
    let body = &callee.instructions;

    // Size limit: 2..15 instructions
    if body.is_empty() || body.len() > 15 {
        return None;
    }

    // Purity: reject loops
    if body
        .iter()
        .any(|ci| matches!(ci, Instruction::LoopBegin(_)))
    {
        return None;
    }
    // Reject fused returns (inliner only handles plain Return)
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::IntAddReturn { .. }
                | Instruction::IntSubReturn { .. }
                | Instruction::IntMulReturn { .. }
                | Instruction::IntDivReturn { .. }
                | Instruction::IntCmpIReturn { .. }
                | Instruction::ReturnConst { .. }
        )
    }) {
        return None;
    }
    // Reject side effects
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::StoreGlobal { .. }
                | Instruction::DeclGlobal { .. }
                | Instruction::MethodCall { .. }
                | Instruction::SuperCall { .. }
                | Instruction::Call { .. }
                | Instruction::IndexAssign { .. }
                | Instruction::IndexAssignArray { .. }
                | Instruction::SetProperty { .. }
                | Instruction::Yield { .. }
                | Instruction::Await { .. }
                | Instruction::Spawn { .. }
                | Instruction::Throw { .. }
                | Instruction::LoopBegin(_)
                | Instruction::TryBegin(_)
        )
    }) {
        return None;
    }

    // Reject jumps/conditionals inside the body
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::Jump(..)
                | Instruction::JumpIfFalse { .. }
                | Instruction::JumpIfTrue { .. }
                | Instruction::Break
        )
    }) {
        return None;
    }

    // Last instruction must be plain Return
    let ret_src = match body.last() {
        Some(Instruction::Return { src }) => *src,
        _ => return None,
    };

    // ---- Register remap with checked arithmetic --------------------------
    // params 0..arg_count → first_arg..first_arg+arg_count
    // other callee regs → base + (reg - arg_count)  where base=first_arg+arg_count
    // 255 stays 255 (special VM register, used by compile_complex path)
    let base: u8 = first_arg.checked_add(arg_count)?;

    let map_reg = |r: u8| -> Option<u8> {
        if r == 255 {
            Some(255)
        } else if r < arg_count {
            Some(first_arg.checked_add(r)?)
        } else {
            let offset = r.checked_sub(arg_count)?;
            Some(base.checked_add(offset)?)
        }
    };

    // Pre-validate that all register operands can be remapped without wrapping
    for ci in body.iter() {
        if !can_remap_regs(ci, &map_reg) {
            return None;
        }
    }

    // ---- Build remapped instructions ------------------------------------
    let mut out = Vec::with_capacity(body.len());
    for (i, ci) in body.iter().enumerate() {
        if i == body.len() - 1 {
            let mapped_src = map_reg(ret_src)?;
            if dst != mapped_src {
                out.push(Instruction::Move {
                    dst,
                    src: mapped_src,
                });
            }
        } else {
            out.push(remap_single_instr(
                ci,
                &map_reg,
                const_remap,
                int_remap,
                num_remap,
            )?);
        }
    }

    Some(out)
}

/// Check that every register operand in `instr` can be mapped without overflow.
fn can_remap_regs<F: Fn(u8) -> Option<u8>>(instr: &Instruction, m: &F) -> bool {
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
fn remap_single_instr<F: Fn(u8) -> Option<u8>>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use hudhudscript_bytecode::{FunctionChunk, Instruction};

    fn make_chunk(_name: &str, params: Vec<&str>, instructions: Vec<Instruction>) -> FunctionChunk {
        FunctionChunk {
            params: params.iter().map(|s| s.to_string()).collect(),
            instructions,
            constants: vec![],
            captures: vec![],
            capture_sym_ids: vec![],
            capture_slots: vec![],
            is_async: false,
            is_generator: false,
            local_count: 2,
            local_names: params.iter().map(|s| s.to_string()).collect(),
            capture_cells: vec![],
            max_register: 2,
            sym_to_slot: std::sync::OnceLock::new(),
            source_positions: vec![],
            param_slots: (0..params.len() as u16)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            is_plain_function: true,
        }
    }

    #[test]
    fn add1_is_inlinable() {
        let chunk = make_chunk(
            "add1",
            vec!["x"],
            vec![
                Instruction::IntAddI {
                    dst: 1,
                    src: 0,
                    imm: 1,
                },
                Instruction::Return { src: 1 },
            ],
        );
        let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
        assert!(result.is_some(), "add1(x)=x+1 should be inlinable");
    }

    #[test]
    fn recursive_not_inlinable() {
        let chunk = make_chunk(
            "recurse",
            vec!["x"],
            vec![
                Instruction::Call {
                    dst: 1,
                    payload_idx: 0,
                    first_arg: 0,
                    arg_count: 1,
                },
                Instruction::Return { src: 1 },
            ],
        );
        let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
        assert!(result.is_none(), "recursive function must NOT be inlinable");
    }

    #[test]
    fn side_effect_not_inlinable() {
        let chunk = make_chunk(
            "s",
            vec!["x"],
            vec![
                Instruction::StoreGlobal { src: 0, sym: 0 },
                Instruction::Return { src: 0 },
            ],
        );
        let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
        assert!(result.is_none(), "side-effect must NOT be inlinable");
    }
}
