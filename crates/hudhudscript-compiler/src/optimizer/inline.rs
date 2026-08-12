use super::*;

/// ⚠️ EXPERIMENTAL — DO NOT ACTIVATE ⚠️
/// Function inlining (P7).
///
/// Status: **DEFERRED / NOT PRODUCTION-READY**
///
/// Known issues:
/// - Jump offset remapping not implemented
/// - source_positions/loop_payloads not adjusted after splice
/// - Register remap incomplete (clone fallback for many instructions)
/// - Return→Move remap checks pre-remap register, not post-remap
/// - Function-body inlining impossible (Arc<FunctionChunk> immutability)
/// - Index update after splice is incorrect
///
/// Activation requires: mutable IR, position remap, jump remap,
/// register allocation verification, and dedicated inline tests.
///
/// ## Original design intent:
///
/// Inlines small pure functions (< 8 instructions, no loops/recursion/side-effects)
/// by replacing Call instructions with the callee body.
pub fn inline_small_functions(
    instructions: &mut Vec<Instruction>,
    bytecode_functions: &HashMap<String, Arc<FunctionChunk>>,
    call_payloads: &[CallPayload],
) {
    let mut i = 0;
    while i < instructions.len() {
        let (payload_idx, dst, first_arg, arg_count) = match &instructions[i] {
            Instruction::Call { payload_idx, dst, first_arg, arg_count } => {
                (*payload_idx, *dst, *first_arg, *arg_count)
            }
            _ => { i += 1; continue; }
        };

        let Some(payload) = call_payloads.get(payload_idx as usize) else {
            i += 1; continue;
        };
        let name = hudhudscript_bytecode::interner::resolve(
            hudhudscript_bytecode::interner::SymbolId(payload.sym.0),
        );
        let Some(chunk) = bytecode_functions.get(&name) else {
            i += 1; continue;
        };

        let body = &chunk.instructions;
        if body.len() < 2 || body.len() > 8 {
            i += 1; continue;
        }

        // Purity checks
        let has_loop = body.iter().any(|ci| matches!(ci, Instruction::LoopBegin(_)));
        let has_try = body.iter().any(|ci| matches!(ci, Instruction::TryBegin(_)));
        let has_recursion = body.iter().any(|ci| match ci {
            Instruction::Call { payload_idx: idx, .. } => {
                call_payloads.get(*idx as usize)
                    .map(|p| p.sym == payload.sym)
                    .unwrap_or(false)
            }
            _ => false,
        });
        // Side effects: global writes, method calls, array writes in callee
        let has_side_effect = body.iter().any(|ci| matches!(ci,
            Instruction::StoreGlobal { .. } | Instruction::DeclGlobal { .. }
            | Instruction::MethodCall { .. } | Instruction::SuperCall { .. }
            | Instruction::IndexAssign { .. } | Instruction::IndexAssignArray { .. }
            | Instruction::SetProperty { .. } | Instruction::Yield { .. }
            | Instruction::Await { .. } | Instruction::Spawn { .. }
            | Instruction::Throw { .. }
        ));

        if has_loop || has_try || has_recursion || has_side_effect {
            i += 1; continue;
        }

        // Find Return instruction and determine the return register
        let mut ret_src: Option<u8> = None;
        for ci in body {
            if let Instruction::Return { src } = ci {
                ret_src = Some(*src);
                break;
            }
            // Handle fused returns
            if let Instruction::IntAddReturn { .. } | Instruction::IntSubReturn { .. }
                | Instruction::IntMulReturn { .. } | Instruction::IntCmpIReturn { .. } = ci
            {
                // For fused returns, the result is implicitly in a fixed register.
                // Skip these because they require special handling.
                i += 1;
                ret_src = None; // skip this candidate
                break;
            }
        }
        let Some(ret_src) = ret_src else { i += 1; continue; };

        // Compute register remap:
        // 0..arg_count (params) → first_arg..first_arg+arg_count
        // ret_src → dst
        // other registers → first_arg + arg_count + offset
        let base = first_arg as usize + arg_count as usize;
        let mut remapped: Vec<Instruction> = Vec::with_capacity(body.len());
        for ci in body {
            let new_instr = remap_instr(ci, first_arg, arg_count, dst, ret_src, base as u8);
            remapped.push(new_instr);
        }

        // Replace Call with remapped body
        // The Return becomes a Move to dst (or Nop if dst == ret_src after remap)
        let return_idx = remapped.iter().position(|ci| matches!(ci, Instruction::Return { .. }));
        if let Some(ri) = return_idx {
            if dst != ret_src {
                let remapped_ret = remap_reg(ret_src, first_arg, arg_count, dst, base as u8);
                remapped[ri] = Instruction::Move { dst, src: remapped_ret };
            } else {
                remapped.remove(ri);
            }
        }

        // Splice
        let old_len = instructions.len();
        instructions.splice(i..=i, remapped);
        // Adjust i: skip past the newly inserted instructions
        i += instructions.len() - old_len + 1; // +1 because we removed 1 (Call) and added body.len()
        // Actually i + (new_len - 1) - 1 + 1... let's just set i to after the inserted block
        // Correct: i was pointing at the Call. After splice, body.len() instructions are at i.
        // We want to continue after them: i + body.len()
        i = i + body.len() - (instructions.len() - old_len); 
        // Simpler: reset i to previous i + (current instructions at that position count)
    }
}

/// Remap a single instruction's registers for inlining.
fn remap_instr(
    instr: &Instruction,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
    _ret_src: u8,
    base: u8,
) -> Instruction {
    let map = |r: u8| -> u8 {
        remap_reg(r, first_arg, arg_count, dst, base)
    };

    match *instr {
        Instruction::Move { dst: d, src: s } => Instruction::Move { dst: map(d), src: map(s) },
        Instruction::IntAdd { dst: d, src1: s1, src2: s2 } => Instruction::IntAdd { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::IntSub { dst: d, src1: s1, src2: s2 } => Instruction::IntSub { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::IntMul { dst: d, src1: s1, src2: s2 } => Instruction::IntMul { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::IntDiv { dst: d, src1: s1, src2: s2 } => Instruction::IntDiv { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::IntMod { dst: d, src1: s1, src2: s2 } => Instruction::IntMod { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::NumAdd { dst: d, src1: s1, src2: s2 } => Instruction::NumAdd { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::NumSub { dst: d, src1: s1, src2: s2 } => Instruction::NumSub { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::NumMul { dst: d, src1: s1, src2: s2 } => Instruction::NumMul { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::NumDiv { dst: d, src1: s1, src2: s2 } => Instruction::NumDiv { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::NumMod { dst: d, src1: s1, src2: s2 } => Instruction::NumMod { dst: map(d), src1: map(s1), src2: map(s2) },
        Instruction::IntCmp { dst: d, src1: s1, src2: s2, op } => Instruction::IntCmp { dst: map(d), src1: map(s1), src2: map(s2), op },
        Instruction::IntAddI { dst: d, src: s, imm } => Instruction::IntAddI { dst: map(d), src: map(s), imm },
        Instruction::IntSubI { dst: d, src: s, imm } => Instruction::IntSubI { dst: map(d), src: map(s), imm },
        Instruction::IntMulI { dst: d, src: s, imm } => Instruction::IntMulI { dst: map(d), src: map(s), imm },
        Instruction::IntDivI { dst: d, src: s, imm } => Instruction::IntDivI { dst: map(d), src: map(s), imm },
        Instruction::NumAddI { dst: d, src: s, imm } => Instruction::NumAddI { dst: map(d), src: map(s), imm },
        Instruction::NumSubI { dst: d, src: s, imm } => Instruction::NumSubI { dst: map(d), src: map(s), imm },
        Instruction::NumMulI { dst: d, src: s, imm } => Instruction::NumMulI { dst: map(d), src: map(s), imm },
        Instruction::NumDivI { dst: d, src: s, imm } => Instruction::NumDivI { dst: map(d), src: map(s), imm },
        Instruction::Neg { dst: d, src: s } => Instruction::Neg { dst: map(d), src: map(s) },
        Instruction::Not { dst: d, src: s } => Instruction::Not { dst: map(d), src: map(s) },
        Instruction::LoadIntConst { dst: d, const_idx } => Instruction::LoadIntConst { dst: map(d), const_idx },
        Instruction::LoadConst { dst: d, const_idx } => Instruction::LoadConst { dst: map(d), const_idx },
        Instruction::LoadNumConst { dst: d, const_idx } => Instruction::LoadNumConst { dst: map(d), const_idx },
        Instruction::LoadGlobal { dst: d, sym } => Instruction::LoadGlobal { dst: map(d), sym },
        Instruction::Index { dst: d, obj: o, idx } => Instruction::Index { dst: map(d), obj: map(o), idx: map(idx) },
        Instruction::IndexArray { dst: d, obj: o, idx } => Instruction::IndexArray { dst: map(d), obj: map(o), idx: map(idx) },
        Instruction::IndexStringAscii { dst: d, obj: o, idx } => Instruction::IndexStringAscii { dst: map(d), obj: map(o), idx: map(idx) },
        Instruction::IntCmpI { dst: d, src: s, op, imm } => Instruction::IntCmpI { dst: map(d), src: map(s), op, imm },
        Instruction::JumpIfFalse { src, offset } => Instruction::JumpIfFalse { src: map(src), offset },
        Instruction::JumpIfTrue { src, offset } => Instruction::JumpIfTrue { src: map(src), offset },
        Instruction::Return { src } => Instruction::Return { src: map(src) },
        _ => instr.clone(),
    }
}

fn remap_reg(r: u8, first_arg: u8, arg_count: u8, _dst: u8, base: u8) -> u8 {
    if r < arg_count {
        // Parameter → caller's first_arg + r
        first_arg + r
    } else if r == 255 {
        255 // reg255 stays
    } else {
        // Local → base + (r - arg_count)
        base + (r - arg_count)
    }
}
