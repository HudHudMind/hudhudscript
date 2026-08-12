use super::fuse_helpers::{instr_reads_reg, reg_dead_after, remove_fused_pair, try_fuse_arith_return};
use super::fuse_super_extra::try_fuse_extra_pattern;
use crate::optimizer::utils::adjust_jumps_after_remove_full;
use hudhudscript_bytecode::{CallPayload, Instruction, LoopPayload, SuperInstrPayload};

pub fn fuse_super_instructions_with_positions(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    _call_payloads: &[CallPayload],
    super_instr_payloads: &mut Vec<SuperInstrPayload>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        if try_fuse_arith_return(instructions, loop_payloads, source_positions, i) {
            continue;
        }
        match (&instructions[i], &instructions[i + 1]) {
            (
                Instruction::IntMul { dst: t, src1, src2 },
                Instruction::IntMod { dst, src1: mod_src1, src2: mod_src2 },
            ) if *t == *mod_src1 => {
                instructions[i] = Instruction::IntMulMod {
                    dst: *dst,
                    src1: *src1,
                    src2: *src2,
                    src3: *mod_src2,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::IntMul { dst: t, src1, src2 },
                Instruction::IntModI { dst, src: mod_src, imm },
            ) if *t == *mod_src => {
                instructions[i] = Instruction::IntMulModI {
                    dst: *dst,
                    src1: *src1,
                    src2: *src2,
                    imm: *imm,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::IntSubI { dst, src, imm },
                Instruction::Call {
                    dst: call_dst,
                    payload_idx,
                    first_arg,
                    arg_count,
                },
            ) if *dst == *first_arg && *arg_count == 1 => {
                let super_idx = super_instr_payloads.len() as u32;
                super_instr_payloads.push(SuperInstrPayload {
                    call_idx: *payload_idx as u32,
                    slot: *src as u32,
                    imm: *imm,
                    offset: 0,
                    call_dst: *call_dst as u32,
                    arg_reg: *first_arg,
                });
                instructions[i] = Instruction::IntSubCall1(super_idx);
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::IntAddI { dst, src, imm },
                Instruction::Call {
                    dst: call_dst,
                    payload_idx,
                    first_arg,
                    arg_count,
                },
            ) if *dst == *first_arg && *arg_count == 1 => {
                let super_idx = super_instr_payloads.len() as u32;
                super_instr_payloads.push(SuperInstrPayload {
                    call_idx: *payload_idx as u32,
                    slot: *src as u32,
                    imm: *imm,
                    offset: 0,
                    call_dst: *call_dst as u32,
                    arg_reg: *first_arg,
                });
                instructions[i] = Instruction::IntAddCall1(super_idx);
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }

            (
                Instruction::NumMul { dst, src1, src2 },
                Instruction::NumAdd {
                    dst: dst2,
                    src1: src1_2,
                    src2: src2_2,
                },
            ) if *dst == *dst2 && *dst == *src1_2 && *src1 == *dst => {
                instructions[i] = Instruction::NumMulAddAssign {
                    dst: *dst,
                    mul: *src2,
                    add: *src2_2,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            // FloatMulAdd: NumMul { R1, R2, R3 } + NumAdd { R1, R1, R4 } → FloatMulAdd { R1, R2, R3, R4 }
            (
                Instruction::NumMul { dst, src1, src2 },
                Instruction::NumAdd {
                    dst: dst2,
                    src1: src1_2,
                    src2: src2_2,
                },
            ) if *dst == *dst2 && *dst == *src1_2 && !(*src1 == *dst) => {
                instructions[i] = Instruction::FloatMulAdd {
                    dst: *dst,
                    mul1: *src1,
                    mul2: *src2,
                    add: *src2_2,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::Index {
                    dst: t,
                    obj: arr,
                    idx,
                },
                Instruction::NumMulAddAssign { dst: acc, mul, add },
            ) if *t == *add => {
                instructions[i] = Instruction::NumMulAddIndexed {
                    acc: *acc,
                    mul: *mul,
                    arr: *arr,
                    idx: *idx,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            // P4b: also match IndexArray (from type propagation)
            (
                Instruction::IndexArray {
                    dst: t,
                    obj: arr,
                    idx,
                },
                Instruction::NumMulAddAssign { dst: acc, mul, add },
            ) if *t == *add => {
                instructions[i] = Instruction::NumMulAddIndexed {
                    acc: *acc,
                    mul: *mul,
                    arr: *arr,
                    idx: *idx,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::Index {
                    dst: a,
                    obj: s1,
                    idx: i1,
                },
                Instruction::Index {
                    dst: b,
                    obj: s2,
                    idx: i2,
                },
            ) if *s1 == *s2 && i + 2 < instructions.len() => {
                let a_r = *a;
                let b_r = *b;
                let s_r = *s1;
                let i1_r = *i1;
                let i2_r = *i2;
                let is_cmp = matches!(&instructions[i + 2],
                    Instruction::IntCmp { dst: _, src1: a2, src2: b2, op: o }
                    if *a2 == a_r && *b2 == b_r && (*o == 4 || *o == 5));
                if is_cmp {
                    let (dst_reg, op) =
                        if let Instruction::IntCmp { dst, op, .. } = instructions[i + 2] {
                            (dst, op)
                        } else {
                            unreachable!()
                        };
                    instructions[i] = Instruction::StrCharEqRR {
                        dst: dst_reg,
                        src_s: s_r,
                        src_i: i1_r,
                        src_j: i2_r,
                    };
                    if op == 5 {
                        instructions[i + 1] = Instruction::Not {
                            dst: dst_reg,
                            src: dst_reg,
                        };
                        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 2);
                        instructions.remove(i + 2);
                    } else {
                        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 2);
                        instructions.remove(i + 2);
                        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 1);
                        instructions.remove(i + 1);
                    }
                    if i + 2 < source_positions.len() {
                        source_positions.remove(i + 2);
                    }
                    if op == 4 && i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
            }
            (
                Instruction::IntCmpI {
                    dst: cmp_dst,
                    src,
                    imm,
                    op,
                },
                Instruction::JumpIfFalse {
                    src: jmp_src,
                    offset,
                },
            ) if *cmp_dst == *jmp_src
                && reg_dead_after(instructions, loop_payloads, i + 2, *cmp_dst) => {
                instructions[i] = Instruction::IntCmpIJumpIfFalse {
                    src: *src,
                    imm: *imm,
                    op: *op,
                    offset: offset.wrapping_add(1),
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::IntCmp {
                    dst: cmp_dst,
                    src1,
                    src2,
                    op,
                },
                Instruction::JumpIfFalse {
                    src: jmp_src,
                    offset,
                },
            ) if *cmp_dst == *jmp_src
                && reg_dead_after(instructions, loop_payloads, i + 2, *cmp_dst) => {
                instructions[i] = Instruction::IntCmpRRJumpIfFalse {
                    src1: *src1,
                    src2: *src2,
                    op: *op,
                    offset: offset.wrapping_add(1),
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            // G4: !x dalını çevir — Not + JumpIfFalse → JumpIfTrue.
            // JumpIfFalse(!x) "!x falsy iken" = "x truthy iken" atlar ≡
            // JumpIfTrue(x). `Not` is_truthy değilini yazar; JumpIfTrue
            // is_truthy test eder — her değer tipinde birebir eşdeğer.
            // Bigram kanıtı: NOT_R→JIF 33.3M aday (plan G4).
            (
                Instruction::Not { dst, src },
                Instruction::JumpIfFalse {
                    src: jmp_src,
                    offset,
                },
            ) if *dst == *jmp_src
                && reg_dead_after(instructions, loop_payloads, i + 2, *dst) => {
                instructions[i] = Instruction::JumpIfTrue {
                    src: *src,
                    offset: offset.wrapping_add(1),
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (Instruction::LoadConst { dst, const_idx }, Instruction::StoreGlobal { src, sym })
                if *dst == *src =>
            {
                instructions[i] = Instruction::StoreGlobalConst {
                    sym: *sym,
                    const_idx: *const_idx,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (Instruction::LoopEnd, Instruction::IntAddI { dst, src, imm })
                if *dst == *src && i + 2 < instructions.len() =>
            {
                // C3: ForCStyle `continue` backpatches payload.start to point to
                // the IntAddI position (right after LoopEnd). Fusing would create
                // LoopEndIntAddIJump, invalidate the continue target, and cause a
                // header double-pop. Skip fusion for such loops.
                let is_continue_target = loop_payloads.iter().any(|p| p.start == (i + 1) as u32);
                if !is_continue_target {
                    if let Instruction::Jump(offset) = &instructions[i + 2] {
                        let new_offset = offset.wrapping_add(2);
                        instructions[i] = Instruction::LoopEndIntAddIJump {
                            reg: *dst,
                            imm: *imm,
                            offset: new_offset as i16,
                        };
                        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 2);
                        instructions.remove(i + 2);
                        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 1);
                        instructions.remove(i + 1);
                        if i + 1 < source_positions.len() {
                            source_positions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                        }
                        continue;
                    }
                }
            }
            (Instruction::IntAddI { dst, src, imm }, Instruction::Jump(offset)) if *dst == *src => {
                // FAZ E: only fuse backward jumps (loop back-edges), not forward
                // jumps (if-else skip-else) — forward fusions break loop counters.
                if *offset >= 0 { i += 1; continue; }
                // Also skip if any OTHER instruction jumps to this Jump (shared target).
                let jump_pos = (i + 1) as i64;
                let has_other_jumper = instructions.iter().enumerate().any(|(ip, instr)| {
                    if ip == i { return false; }
                    match instr {
                        Instruction::Jump(o) => (ip as i64 + *o as i64) == jump_pos,
                        Instruction::IntAddIJump { offset: o, .. }
                        | Instruction::IntSubIJump { offset: o, .. }
                        | Instruction::LoopEndIntAddIJump { offset: o, .. } => {
                            (ip as i64 + *o as i64) == jump_pos
                        }
                        Instruction::JumpIfFalse { offset: o, .. }
                        | Instruction::JumpIfTrue { offset: o, .. } => {
                            (ip as i64 + *o as i64) == jump_pos
                        }
                        _ => false,
                    }
                });
                if has_other_jumper { i += 1; continue; }
                let new_offset = offset.wrapping_add(1);
                instructions[i] = Instruction::IntAddIJump {
                    reg: *dst,
                    imm: *imm,
                    offset: new_offset as i16,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (Instruction::IntSubI { dst, src, imm }, Instruction::Jump(offset)) if *dst == *src => {
                // FAZ E: only fuse backward jumps (loop back-edges), not forward
                // jumps (if-else skip-else) — forward fusions break loop counters.
                if *offset >= 0 { i += 1; continue; }
                // Also skip if any OTHER instruction jumps to this Jump. The Jump
                // is a shared loop back-edge; fusing would cause the OTHER path
                // (e.g., if-branch after its own c=c-1) to double-decrement.
                let jump_pos = (i + 1) as i64;
                let has_other_jumper = instructions.iter().enumerate().any(|(ip, instr)| {
                    if ip == i { return false; }
                    match instr {
                        Instruction::Jump(o) => (ip as i64 + *o as i64) == jump_pos,
                        Instruction::IntAddIJump { offset: o, .. }
                        | Instruction::IntSubIJump { offset: o, .. }
                        | Instruction::LoopEndIntAddIJump { offset: o, .. } => {
                            (ip as i64 + *o as i64) == jump_pos
                        }
                        Instruction::JumpIfFalse { offset: o, .. }
                        | Instruction::JumpIfTrue { offset: o, .. } => {
                            (ip as i64 + *o as i64) == jump_pos
                        }
                        _ => false,
                    }
                });
                if has_other_jumper { i += 1; continue; }
                let new_offset = offset.wrapping_add(1);
                instructions[i] = Instruction::IntSubIJump {
                    reg: *dst,
                    imm: *imm,
                    offset: new_offset as i16,
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            (
                Instruction::IntCmpI {
                    dst: cmp_dst,
                    src,
                    imm,
                    op,
                },
                Instruction::JumpIfTrue {
                    src: jmp_src,
                    offset,
                },
            ) if *cmp_dst == *jmp_src
                && reg_dead_after(instructions, loop_payloads, i + 2, *cmp_dst) => {
                instructions[i] = Instruction::IntCmpIJumpIfTrue {
                    src: *src,
                    imm: *imm,
                    op: *op,
                    offset: offset.wrapping_add(1),
                };
                remove_fused_pair(instructions, loop_payloads, source_positions, i);
                continue;
            }
            _ => {
                if try_fuse_extra_pattern(instructions, loop_payloads, source_positions, i) {
                    continue;
                }
            }
        }
        i += 1;
    }
}
