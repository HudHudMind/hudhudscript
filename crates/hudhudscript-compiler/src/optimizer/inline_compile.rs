// P3: Compiler-side function inlining helper.
// Called during compilation when the compiler encounters a Call to a known
// pure function that's already been compiled. Returns inlined instructions
// if the function qualifies, None otherwise.

use hudhudscript_bytecode::{FunctionChunk, Instruction};

/// Try to inline a call to `callee`. Returns remapped instructions if successful.
pub(crate) fn try_inline_call(
    callee: &FunctionChunk,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
) -> Option<Vec<Instruction>> {
    let body = &callee.instructions;

    // Size limit: must be between 2 and 15 instructions
    if body.is_empty() || body.len() > 15 {
        return None;
    }

    // Purity checks
    let has_loop = body.iter().any(|ci| matches!(ci, Instruction::LoopBegin(_)));
    let has_fused_return = body.iter().any(|ci| matches!(ci,
        Instruction::IntAddReturn { .. } | Instruction::IntSubReturn { .. }
        | Instruction::IntMulReturn { .. } | Instruction::IntDivReturn { .. }
        | Instruction::IntCmpIReturn { .. }
    ));
    if has_fused_return { return None; }
    let has_side_effect = body.iter().any(|ci| matches!(ci,
        Instruction::StoreGlobal { .. } | Instruction::DeclGlobal { .. }
        | Instruction::MethodCall { .. } | Instruction::SuperCall { .. } | Instruction::Call { .. }
        | Instruction::IndexAssign { .. } | Instruction::IndexAssignArray { .. }
        | Instruction::SetProperty { .. } | Instruction::Yield { .. }
        | Instruction::Await { .. } | Instruction::Spawn { .. }
        | Instruction::Throw { .. } | Instruction::LoopBegin(_)
        | Instruction::TryBegin(_)
    ));
    if has_loop || has_side_effect {
        return None;
    }

    // Must have exactly one Return at the end (no Jump/conditional)
    let has_jump = body.iter().any(|ci| matches!(ci,
        Instruction::Jump(..) | Instruction::JumpIfFalse { .. }
        | Instruction::JumpIfTrue { .. } | Instruction::Break
    ));
    if has_jump {
        return None;
    }

    // Find return instruction (must be last)
    let ret_src = match body.last() {
        Some(Instruction::Return { src }) => *src,
        _ => return None,
    };

    // Remap registers: params 0..arg_count → first_arg..first_arg+arg_count
    // Other callee registers → remapped above arg window
    // Return src → dst
    let base = first_arg.wrapping_add(arg_count);
    let map = |r: u8| -> u8 {
        if r < arg_count {
            first_arg.wrapping_add(r)
        } else if r == 255 {
            255
        } else {
            base.wrapping_add(r.wrapping_sub(arg_count))
        }
    };

    let mut out = Vec::with_capacity(body.len());
    for (i, ci) in body.iter().enumerate() {
        if i == body.len() - 1 {
            // Return → Move to dst
            let mapped_src = if ret_src < arg_count {
                first_arg.wrapping_add(ret_src)
            } else {
                base.wrapping_add(ret_src.wrapping_sub(arg_count))
            };
            if dst != mapped_src {
                out.push(Instruction::Move { dst, src: mapped_src });
            }
        } else {
            let remapped = remap_single_instr(ci, &map);
            if remapped.is_none() {
                return None; // unsupported instruction
            }
            out.push(remapped.unwrap());
        }
    }

    Some(out)
}

fn remap_single_instr<F: Fn(u8) -> u8>(instr: &Instruction, map: &F) -> Option<Instruction> {
    let m = map;
    Some(match *instr {
        Instruction::Move { dst, src } => Instruction::Move { dst: m(dst), src: m(src) },
        Instruction::LoadIntConst { dst, const_idx } => Instruction::LoadIntConst { dst: m(dst), const_idx },
        Instruction::LoadConst { dst, const_idx } => Instruction::LoadConst { dst: m(dst), const_idx },
        Instruction::LoadNumConst { dst, const_idx } => Instruction::LoadNumConst { dst: m(dst), const_idx },
        Instruction::IntAdd { dst, src1, src2 } => Instruction::IntAdd { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::IntMul { dst, src1, src2 } => Instruction::IntMul { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::IntSub { dst, src1, src2 } => Instruction::IntSub { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::IntDiv { dst, src1, src2 } => Instruction::IntDiv { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::IntCmp { dst, src1, src2, op } => Instruction::IntCmp { dst: m(dst), src1: m(src1), src2: m(src2), op },
        Instruction::IntCmpI { dst, src, op, imm } => Instruction::IntCmpI { dst: m(dst), src: m(src), op, imm },
        Instruction::NumAdd { dst, src1, src2 } => Instruction::NumAdd { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::NumSub { dst, src1, src2 } => Instruction::NumSub { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::NumMul { dst, src1, src2 } => Instruction::NumMul { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::NumDiv { dst, src1, src2 } => Instruction::NumDiv { dst: m(dst), src1: m(src1), src2: m(src2) },
        Instruction::NumAddI { dst, src, imm } => Instruction::NumAddI { dst: m(dst), src: m(src), imm },
        Instruction::IntAddI { dst, src, imm } => Instruction::IntAddI { dst: m(dst), src: m(src), imm },
        Instruction::IntMulI { dst, src, imm } => Instruction::IntMulI { dst: m(dst), src: m(src), imm },
        Instruction::IntDivI { dst, src, imm } => Instruction::IntDivI { dst: m(dst), src: m(src), imm },
        Instruction::NumDivI { dst, src, imm } => Instruction::NumDivI { dst: m(dst), src: m(src), imm },
        Instruction::Neg { dst, src } => Instruction::Neg { dst: m(dst), src: m(src) },
        Instruction::Not { dst, src } => Instruction::Not { dst: m(dst), src: m(src) },
        Instruction::JumpIfFalse { src, offset } => Instruction::JumpIfFalse { src: m(src), offset },
        Instruction::JumpIfTrue { src, offset } => Instruction::JumpIfTrue { src: m(src), offset },
        Instruction::Return { src } => Instruction::Move { dst: m(255), src: m(src) },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hudhudscript_bytecode::{FunctionChunk, Instruction};

    fn make_chunk(name: &str, params: Vec<&str>, instructions: Vec<Instruction>) -> FunctionChunk {
        FunctionChunk {
            params: params.iter().map(|s| s.to_string()).collect(),
            instructions,
            constants: vec![],
            captures: vec![],
            is_async: false,
            is_generator: false,
            local_count: 2,
            local_names: params.iter().map(|s| s.to_string()).collect(),
            capture_cells: vec![],
            max_register: 2,
            sym_to_slot: std::sync::OnceLock::new(),
            source_positions: vec![],
            param_slots: (0..params.len() as u16).collect::<Vec<_>>().into_boxed_slice(),
            is_plain_function: true,
        }
    }

    #[test]
    fn add1_is_inlinable() {
        let chunk = make_chunk("add1", vec!["x"], vec![
            Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
            Instruction::Return { src: 1 },
        ]);
        let result = try_inline_call(&chunk, 10, 1, 255);
        assert!(result.is_some(), "add1(x)=x+1 should be inlinable");
    }

    #[test]
    fn recursive_not_inlinable() {
        let chunk = make_chunk("recurse", vec!["x"], vec![
            Instruction::Call { dst: 1, payload_idx: 0, first_arg: 0, arg_count: 1 },
            Instruction::Return { src: 1 },
        ]);
        let result = try_inline_call(&chunk, 10, 1, 255);
        assert!(result.is_none(), "recursive function must NOT be inlinable");
    }

    #[test]
    fn side_effect_not_inlinable() {
        let chunk = make_chunk("s", vec!["x"], vec![
            Instruction::StoreGlobal { src: 0, sym: 0 },
            Instruction::Return { src: 0 },
        ]);
        let result = try_inline_call(&chunk, 10, 1, 255);
        assert!(result.is_none(), "side-effect must NOT be inlinable");
    }
}
