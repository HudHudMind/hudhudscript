//! Slot+immediate fusion pass — re-enabled with jump offset correction (v0.4.468).

use hudhudscript_bytecode::{Instruction, LoopPayload, CallPayload};

/// Jump offset'lerini ve loop payload'larını bir instruction silindikten sonra düzeltir.
/// `removed_at`: silinen instruction'ın index'i.
/// dead_code.rs'deki `dead_code_eliminate_with_positions` pattern'ı referans alınmıştır.
fn adjust_jumps_after_remove(
    instructions: &mut [Instruction],
    loop_payloads: &mut [LoopPayload],
    removed_at: usize,
) {
    // 1. Adjust instruction-embedded jump offsets
    let len = instructions.len();
    for ip in 0..len {
        let adjust = |off: &mut i32, ip: usize| {
            let target = (ip as i64 + *off as i64) as usize;
            let new_ip = if ip > removed_at { ip - 1 } else { ip };
            let new_target = if target > removed_at { target - 1 } else { target };
            *off = (new_target as i64 - new_ip as i64) as i32;
        };
        match &mut instructions[ip] {
            Instruction::Jump(o)
            | Instruction::TryBegin(o)
            | Instruction::FinallyBegin(o)
            | Instruction::FinallyExit(o) => adjust(o, ip),
            Instruction::JumpIfFalse { offset, .. }
            | Instruction::JumpIfTrue { offset, .. } => {
                let mut o32 = *offset as i32;
                adjust(&mut o32, ip);
                *offset = o32 as i16;
            }
            Instruction::ForIn { end_offset, .. }
            | Instruction::IterNext { end_offset, .. } => {
                let mut o32 = *end_offset as i32;
                adjust(&mut o32, ip);
                *end_offset = o32 as i16;
            }
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntLeRIJumpIfFalse { offset, .. }
            | Instruction::IntLtRIJumpIfFalse { offset, .. } => {
                let mut o32 = *offset as i32;
                adjust(&mut o32, ip);
                *offset = o32 as i16;
            }
            _ => {}
        }
    }
    // 2. Adjust loop payload absolute IPs (Break/Continue use these)
    for lp in loop_payloads.iter_mut() {
        if lp.start as usize > removed_at {
            lp.start -= 1;
        }
        if lp.end as usize > removed_at {
            lp.end -= 1;
        }
    }
}

pub fn fuse_slot_immediate(instructions: &mut Vec<Instruction>, loop_payloads: &mut [LoopPayload]) {
    // No-op
}

pub fn fuse_slot_immediate_with_positions(
    instructions: &mut Vec<Instruction>,
    numeric_constants: &[u64],
    int_constants: &[i64],
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        // Fold: LoadIntConst(1) + IntDiv/IntMul → skip
        if let Instruction::LoadIntConst { dst: const_dst, const_idx } = &instructions[i] {
            let const_val = int_constants.get(*const_idx as usize).copied();
            match &instructions[i + 1] {
                Instruction::IntDiv { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(1) => {
                    instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                Instruction::IntMul { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(1) => {
                    instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                Instruction::NumDiv { dst, src1, src2 } if *src2 == *const_dst => {
                    if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                        let val = f64::from_bits(bits);
                        if (val - 1.0).abs() < f64::EPSILON {
                            instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                    }
                }
                Instruction::NumMul { dst, src1, src2 } if *src2 == *const_dst => {
                    if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                        let val = f64::from_bits(bits);
                        if (val - 1.0).abs() < f64::EPSILON {
                            instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                    }
                }
                _ => {}
            }
            // Fold: LoadIntConst(0) + IntMul → constant 0
            match &instructions[i + 1] {
                Instruction::IntMul { dst, src1: _, src2 } if *src2 == *const_dst && const_val == Some(0) => {
                    instructions[i] = Instruction::LoadIntConst { dst: *dst, const_idx: *const_idx };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                Instruction::IntAdd { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(0) => {
                    instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                Instruction::IntSub { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(0) => {
                    instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                // Fold: LoadIntConst(-1) + IntMul → Neg
                Instruction::IntMul { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(-1) => {
                    instructions[i] = Instruction::Neg { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                // Fold: LoadIntConst(-1) + IntDiv → Neg  (x / -1 == -x)
                Instruction::IntDiv { dst, src1, src2 } if *src2 == *const_dst && const_val == Some(-1) => {
                    instructions[i] = Instruction::Neg { dst: *dst, src: *src1 };
                    adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                    continue;
                }
                _ => {}
            }
        }
        // Also fold LoadNumConst patterns
        if let Instruction::LoadNumConst { dst: const_dst, const_idx } = &instructions[i] {
            if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                let val = f64::from_bits(bits);
                // *0.0 → 0.0
                if (val - 0.0).abs() < f64::EPSILON {
                    if let Instruction::NumMul { dst, src1: _, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move { dst: *dst, src: *const_dst };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                    }
                }
                // +0.0 → move
                if (val - 0.0).abs() < f64::EPSILON {
                    if let Instruction::NumAdd { dst, src1, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                    } else if let Instruction::NumSub { dst, src1, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move { dst: *dst, src: *src1 };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                    }
                }
                // Fuse: LoadNumConst + NumAdd/Sub/Mul/Div → Num*I (small int values)
                let int_val = val as i16;
                if (val - int_val as f64).abs() < f64::EPSILON && int_val as f64 == val {
                    match &instructions[i + 1] {
                        Instruction::NumAdd { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumAddI { dst: *dst, src: *src1, imm: int_val };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                        Instruction::NumSub { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumSubI { dst: *dst, src: *src1, imm: int_val };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                        Instruction::NumMul { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumMulI { dst: *dst, src: *src1, imm: int_val };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                        Instruction::NumDiv { dst, src1, src2 } if *src2 == *const_dst && int_val != 0 => {
                            instructions[i] = Instruction::NumDivI { dst: *dst, src: *src1, imm: int_val };
                            adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                            continue;
                        }
                        _ => {}
                    }
                }
            }
        }
        // Fuse: LoadIntConst + IntCmp → IntCmpI
        if i + 1 < instructions.len() {
            if let Instruction::LoadIntConst { dst: const_dst, const_idx } = &instructions[i] {
                let const_val = int_constants.get(*const_idx as usize).copied();
                if let Instruction::IntCmp { dst: cmp_dst, src1, src2, op } = &instructions[i + 1] {
                    if *src2 == *const_dst {
                        if let Some(val) = const_val {
                            let imm = val as i16;
                            if imm as i64 == val {
                                instructions[i] = Instruction::IntCmpI { dst: *cmp_dst, src: *src1, imm, op: *op };
                                adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                                continue;
                            }
                        }
                    }
                }
            }
            // Fuse: LoadIntConst + IntDiv/IntMod → IntDivI/IntModI
            if let Instruction::LoadIntConst { dst: const_dst, const_idx } = &instructions[i] {
                let c = int_constants.get(*const_idx as usize).copied();
                if let Some(val) = c {
                    let imm = val as i16;
                    if imm as i64 == val && imm != 0 {
                        match &instructions[i + 1] {
                            Instruction::IntDiv { dst, src1, src2 } if *src2 == *const_dst => {
                                instructions[i] = Instruction::IntDivI { dst: *dst, src: *src1, imm };
                                adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                                continue;
                            }
                            Instruction::IntMod { dst, src1, src2 } if *src2 == *const_dst => {
                                instructions[i] = Instruction::IntModI { dst: *dst, src: *src1, imm };
                                adjust_jumps_after_remove(instructions, loop_payloads, i + 1);
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() { source_positions.remove(i + 1); }
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Fold: IntAddI/IntSubI with imm=0 → Move (x+0, x-0 no-ops)
        if let Instruction::IntAddI { dst, src, imm: 0 } | Instruction::IntSubI { dst, src, imm: 0 } = &instructions[i] {
            if *dst != *src { instructions[i] = Instruction::Move { dst: *dst, src: *src }; }
        }
        i += 1;
    }
}
