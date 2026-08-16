//! Slot+immediate fusion pass — re-enabled with jump offset correction (v0.4.468).

use hudhudscript_bytecode::{CallPayload, Instruction, LoopPayload};

/// Jump offset'lerini ve loop payload'larını bir instruction silindikten sonra düzeltir.
/// Moved to utils.rs — single source (Kural 7).
use crate::optimizer::utils::adjust_jumps_after_remove_full;

/// Fuse IntModI + IntCmpI → IntModCmpI in a separate pass.
/// This must run AFTER `fuse_slot_immediate_with_positions` which creates
/// IntModI and IntCmpI from LoadIntConst+IntMod/IntCmp patterns.
pub fn fuse_intmodcmpi_chain(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    let mut i = 0;
    while i + 1 < instructions.len() {
        if let Instruction::IntModI {
            dst: mod_dst,
            src: mod_src,
            imm: mod_imm,
        } = &instructions[i]
        {
            if let Instruction::IntCmpI {
                dst: cmp_dst,
                src: cmp_src,
                imm: cmp_imm,
                op,
            } = &instructions[i + 1]
            {
                if *cmp_src == *mod_dst {
                    // Single-use: only block if mod_dst is READ (not written) after i+2.
                    // If it's written first, the old IntModI value is dead.
                    let mut reused = false;
                    for instr_after in &instructions[i + 2..] {
                        if writes_reg(instr_after, *mod_dst) {
                            break; // write kills the old value
                        }
                        if instruction_reads_reg(instr_after, *mod_dst) {
                            reused = true;
                            break;
                        }
                    }
                    if !reused {
                        instructions[i] = Instruction::IntModCmpI {
                            dst: *cmp_dst,
                            src: *mod_src,
                            mod_imm: *mod_imm,
                            cmp_imm: *cmp_imm,
                            op: *op,
                        };
                        adjust_jumps_after_remove_full(
                            instructions,
                            loop_payloads,
                            &mut [],
                            &mut [],
                            i + 1,
                        );
                        instructions.remove(i + 1);
                        if i + 1 < source_positions.len() {
                            source_positions.remove(i + 1);
                        }
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

/// G2.2b: check if register `reg` is modified in a loop body after `site`.
/// Scans forward until a backward jump is found that targets at or before `site`.
fn reg_modified_in_enclosing_loop(
    instructions: &[Instruction],
    loop_payloads: &[LoopPayload],
    site: usize,
    reg: u8,
) -> bool {
    // First try loop payloads (precise)
    let enclosing = loop_payloads
        .iter()
        .filter(|lp| (lp.start as usize) <= site && site < (lp.end as usize))
        .min_by_key(|lp| lp.end - lp.start);
    let end = if let Some(lp) = enclosing {
        (lp.end as usize).min(instructions.len())
    } else {
        // Fallback: scan forward for backward jumps that loop back to near `site`.
        // A jump targeting far before site is a different loop, not this one.
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
                // Target must be near site (same loop header), not a distant unrelated loop
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

pub fn fuse_slot_immediate(instructions: &mut Vec<Instruction>, loop_payloads: &mut [LoopPayload]) {
    // No-op
}

#[inline(always)]
fn fuse_intmod_cmpi(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: usize,
    mod_dst: u8,
    mod_src: u8,
    mod_imm: i16,
    cmp_dst: u8,
    cmp_imm: i16,
    op: u8,
) {
    // Single-use check: mod_dst must not appear as operand in any later instruction
    let mut reused = false;
    for instr_after in &instructions[i + 2..] {
        if instruction_reads_reg(instr_after, mod_dst) {
            reused = true;
            break;
        }
    }
    if !reused {
        instructions[i] = Instruction::IntModCmpI {
            dst: cmp_dst,
            src: mod_src,
            mod_imm,
            cmp_imm,
            op,
        };
        adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 1);
        instructions.remove(i + 1);
        if i + 1 < source_positions.len() {
            source_positions.remove(i + 1);
        }
    }
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
        if let Instruction::LoadIntConst {
            dst: const_dst,
            const_idx,
        } = &instructions[i]
        {
            let const_val = int_constants.get(*const_idx as usize).copied();
            match &instructions[i + 1] {
                Instruction::IntDiv { dst, src1, src2 }
                    if *src2 == *const_dst && const_val == Some(1) =>
                {
                    instructions[i] = Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                Instruction::IntMul { dst, src1, src2 }
                    if *src2 == *const_dst && const_val == Some(1) =>
                {
                    instructions[i] = Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                Instruction::NumDiv { dst, src1, src2 } if *src2 == *const_dst => {
                    if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                        let val = f64::from_bits(bits);
                        if (val - 1.0).abs() < f64::EPSILON {
                            instructions[i] = Instruction::Move {
                                dst: *dst,
                                src: *src1,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                    }
                }
                Instruction::NumMul { dst, src1, src2 } if *src2 == *const_dst => {
                    if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                        let val = f64::from_bits(bits);
                        if (val - 1.0).abs() < f64::EPSILON {
                            instructions[i] = Instruction::Move {
                                dst: *dst,
                                src: *src1,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                    }
                }
                _ => {}
            }
            // Fold: LoadIntConst(0) + IntMul → constant 0
            // G2.2b: unsafe if const_dst is modified in a loop (IntAddIJump writes to it)
            match &instructions[i + 1] {
                Instruction::IntMul { dst, src1: _, src2 }
                    if *src2 == *const_dst
                        && const_val == Some(0)
                        && !reg_modified_in_enclosing_loop(
                            instructions,
                            loop_payloads,
                            i,
                            *const_dst,
                        ) =>
                {
                    instructions[i] = Instruction::LoadIntConst {
                        dst: *dst,
                        const_idx: *const_idx,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                Instruction::IntAdd { dst, src1, src2 }
                    if *src2 == *const_dst
                        && const_val == Some(0)
                        && !reg_modified_in_enclosing_loop(
                            instructions,
                            loop_payloads,
                            i,
                            *const_dst,
                        ) =>
                {
                    instructions[i] = Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                Instruction::IntSub { dst, src1, src2 }
                    if *src2 == *const_dst
                        && const_val == Some(0)
                        && !reg_modified_in_enclosing_loop(
                            instructions,
                            loop_payloads,
                            i,
                            *const_dst,
                        ) =>
                {
                    instructions[i] = Instruction::Move {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                // Fold: LoadIntConst(-1) + IntMul → Neg
                Instruction::IntMul { dst, src1, src2 }
                    if *src2 == *const_dst
                        && const_val == Some(-1)
                        && !reg_modified_in_enclosing_loop(
                            instructions,
                            loop_payloads,
                            i,
                            *const_dst,
                        ) =>
                {
                    instructions[i] = Instruction::Neg {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                // Fold: LoadIntConst(-1) + IntDiv → Neg  (x / -1 == -x)
                Instruction::IntDiv { dst, src1, src2 }
                    if *src2 == *const_dst
                        && const_val == Some(-1)
                        && !reg_modified_in_enclosing_loop(
                            instructions,
                            loop_payloads,
                            i,
                            *const_dst,
                        ) =>
                {
                    instructions[i] = Instruction::Neg {
                        dst: *dst,
                        src: *src1,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
                _ => {}
            }
        }
        // Also fold LoadNumConst patterns
        if let Instruction::LoadNumConst {
            dst: const_dst,
            const_idx,
        } = &instructions[i]
        {
            if let Some(&bits) = numeric_constants.get(*const_idx as usize) {
                let val = f64::from_bits(bits);
                // *0.0 → 0.0
                if (val - 0.0).abs() < f64::EPSILON {
                    if let Instruction::NumMul { dst, src1: _, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move {
                                dst: *dst,
                                src: *const_dst,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                    }
                }
                // +0.0 → move
                if (val - 0.0).abs() < f64::EPSILON {
                    if let Instruction::NumAdd { dst, src1, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move {
                                dst: *dst,
                                src: *src1,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                    } else if let Instruction::NumSub { dst, src1, src2 } = &instructions[i + 1] {
                        if *src2 == *const_dst {
                            instructions[i] = Instruction::Move {
                                dst: *dst,
                                src: *src1,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                    }
                }
                // Fuse: LoadNumConst + NumAdd/Sub/Mul/Div → Num*I (small int values)
                let int_val = val as i16;
                if (val - int_val as f64).abs() < f64::EPSILON && int_val as f64 == val {
                    match &instructions[i + 1] {
                        Instruction::NumAdd { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumAddI {
                                dst: *dst,
                                src: *src1,
                                imm: int_val,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                        Instruction::NumSub { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumSubI {
                                dst: *dst,
                                src: *src1,
                                imm: int_val,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                        Instruction::NumMul { dst, src1, src2 } if *src2 == *const_dst => {
                            instructions[i] = Instruction::NumMulI {
                                dst: *dst,
                                src: *src1,
                                imm: int_val,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                        Instruction::NumDiv { dst, src1, src2 }
                            if *src2 == *const_dst && int_val != 0 =>
                        {
                            instructions[i] = Instruction::NumDivI {
                                dst: *dst,
                                src: *src1,
                                imm: int_val,
                            };
                            adjust_jumps_after_remove_full(
                                instructions,
                                loop_payloads,
                                &mut [],
                                &mut [],
                                i + 1,
                            );
                            instructions.remove(i + 1);
                            if i + 1 < source_positions.len() {
                                source_positions.remove(i + 1);
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
            }
        }
        // Fuse: LoadIntConst + IntCmp → IntCmpI
        if i + 1 < instructions.len() {
            if let Instruction::LoadIntConst {
                dst: const_dst,
                const_idx,
            } = &instructions[i]
            {
                let const_val = int_constants.get(*const_idx as usize).copied();
                if let Instruction::IntCmp {
                    dst: cmp_dst,
                    src1,
                    src2,
                    op,
                } = &instructions[i + 1]
                {
                    if *src2 == *const_dst {
                        if let Some(val) = const_val {
                            let imm = val as i16;
                            if imm as i64 == val {
                                instructions[i] = Instruction::IntCmpI {
                                    dst: *cmp_dst,
                                    src: *src1,
                                    imm,
                                    op: *op,
                                };
                                adjust_jumps_after_remove_full(
                                    instructions,
                                    loop_payloads,
                                    &mut [],
                                    &mut [],
                                    i + 1,
                                );
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() {
                                    source_positions.remove(i + 1);
                                }
                                continue;
                            }
                        }
                    }
                }
            }
            // Fuse: LoadIntConst + IntDiv/IntMod → IntDivI/IntModI
            if let Instruction::LoadIntConst {
                dst: const_dst,
                const_idx,
            } = &instructions[i]
            {
                let c = int_constants.get(*const_idx as usize).copied();
                if let Some(val) = c {
                    let imm = val as i16;
                    if imm as i64 == val && imm != 0 {
                        match &instructions[i + 1] {
                            Instruction::IntDiv { dst, src1, src2 } if *src2 == *const_dst => {
                                instructions[i] = Instruction::IntDivI {
                                    dst: *dst,
                                    src: *src1,
                                    imm,
                                };
                                adjust_jumps_after_remove_full(
                                    instructions,
                                    loop_payloads,
                                    &mut [],
                                    &mut [],
                                    i + 1,
                                );
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() {
                                    source_positions.remove(i + 1);
                                }
                                continue;
                            }
                            Instruction::IntMod { dst, src1, src2 } if *src2 == *const_dst => {
                                instructions[i] = Instruction::IntModI {
                                    dst: *dst,
                                    src: *src1,
                                    imm,
                                };
                                adjust_jumps_after_remove_full(
                                    instructions,
                                    loop_payloads,
                                    &mut [],
                                    &mut [],
                                    i + 1,
                                );
                                instructions.remove(i + 1);
                                if i + 1 < source_positions.len() {
                                    source_positions.remove(i + 1);
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Fold: IntAddI/IntSubI with imm=0 → Move (x+0, x-0 no-ops)
        if let Instruction::IntAddI { dst, src, imm: 0 }
        | Instruction::IntSubI { dst, src, imm: 0 } = &instructions[i]
        {
            if *dst != *src {
                instructions[i] = Instruction::Move {
                    dst: *dst,
                    src: *src,
                };
            }
        }
        // B4: Re-fuse IntSubI { dst: A, src: B, imm } + Move { dst: B, src: A }
        //     → IntSubI { dst: B, src: B, imm } (self-update, no temp)
        if i + 1 < instructions.len() {
            if let (
                Instruction::IntSubI { dst, src, imm },
                Instruction::Move {
                    dst: m_dst,
                    src: m_src,
                },
            ) = (&instructions[i], &instructions[i + 1])
            {
                if *imm != 0 && *dst == *m_src && *src == *m_dst && *dst != *src {
                    instructions[i] = Instruction::IntSubI {
                        dst: *src,
                        src: *src,
                        imm: *imm,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
            }
        }
        // P2-B1: Re-fuse IntAddI { dst: A, src: B, imm } + Move { dst: B, src: A }
        //         → IntAddI { dst: B, src: B, imm } (self-update, no temp)
        if i + 1 < instructions.len() {
            if let (
                Instruction::IntAddI { dst, src, imm },
                Instruction::Move {
                    dst: m_dst,
                    src: m_src,
                },
            ) = (&instructions[i], &instructions[i + 1])
            {
                if *imm != 0 && *dst == *m_src && *src == *m_dst && *dst != *src {
                    instructions[i] = Instruction::IntAddI {
                        dst: *src,
                        src: *src,
                        imm: *imm,
                    };
                    adjust_jumps_after_remove_full(
                        instructions,
                        loop_payloads,
                        &mut [],
                        &mut [],
                        i + 1,
                    );
                    instructions.remove(i + 1);
                    if i + 1 < source_positions.len() {
                        source_positions.remove(i + 1);
                    }
                    continue;
                }
            }
        }
        i += 1;
    }
}
