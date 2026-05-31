use super::*;

/// Dead-code elimination: remove instructions after unconditional jumps
/// or returns that can never be reached, up to the next jump target.
///
/// Audit v3 F4.2: rewritten for i32 relative jump offsets.  Targets are
/// now resolved as `ip + offset`, and when a range `[i+1, j)` is removed
/// offsets that straddle the deleted zone need to be shrunk (if both
/// jump and target are on the same side of the zone, the offset stays
/// valid).
pub fn dead_code_eliminate(instructions: &mut Vec<Instruction>) {
    // Collect absolute jump targets so we know which IPs are reachable
    // entry points (cannot be dead-code-removed).
    let mut jump_targets = std::collections::HashSet::new();
    for (ip, instr) in instructions.iter().enumerate() {
        match instr {
            Instruction::Jump(o)
            | Instruction::TryBegin(o)
            | Instruction::FinallyBegin(o)
            | Instruction::FinallyExit(o) => {
                jump_targets.insert(abs_target(ip, *o));
            }
            Instruction::JumpIfFalse { offset, .. }
            | Instruction::JumpIfTrue { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            Instruction::IterNext { end_offset, .. } => {
                jump_targets.insert(abs_target(ip, *end_offset as i32));
            }
            // Fused compare+branch instructions also encode jump targets.
            // These must be included so DCE does not remove instructions
            // reachable only via the fused branch (e.g., the outer LoopEnd
            // after a `while (left <= right) { ... break; }` body).
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntLeRIJumpIfFalse { offset, .. }
            | Instruction::IntLtRIJumpIfFalse { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            _ => {}
        }
    }

    let mut i = 0;
    while i < instructions.len() {
        let is_terminal = matches!(
            instructions[i],
            Instruction::Jump(_) | Instruction::Return { .. } | Instruction::Throw { .. }
        );
        if !is_terminal {
            i += 1;
            continue;
        }

        // Remove subsequent unreachable instructions until we hit a jump target.
        let mut j = i + 1;
        while j < instructions.len() && !jump_targets.contains(&j) {
            j += 1;
        }

        if j > i + 1 {
            let removed_count = j - (i + 1);
            // Before draining, rewrite offsets whose jump-site and target
            // are on opposite sides of the deleted range.
            let rc = removed_count as i32;
            // Walk with original indices so we know each jump's true ip.
            let len_before = instructions.len();
            for ip in 0..len_before {
                if ip >= i + 1 && ip < j {
                    // Will be removed — don't touch.
                    continue;
                }
                let adjust = |off: &mut i32, ip: usize| {
                    let target = abs_target(ip, *off);
                    // new position of jump after drain
                    let new_ip = if ip < i + 1 { ip } else { ip - removed_count };
                    // new position of target after drain
                    let new_target = if target < i + 1 {
                        target
                    } else if target >= j {
                        target - removed_count
                    } else {
                        // target was in deleted range — was jumping into
                        // dead code; still adjust conservatively.
                        i + 1
                    };
                    *off = (new_target as i64 - new_ip as i64) as i32;
                    let _ = rc; // silence unused in case of trivial case
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
            instructions.drain((i + 1)..j);
        }
        i += 1;
    }
}

/// Position-aware variant of [`dead_code_eliminate`] — drains
/// source_positions in parallel with the instruction drain so
/// ip-indexed lookups line up after optimization.
pub fn dead_code_eliminate_with_positions(
    instructions: &mut Vec<Instruction>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    // Audit v3 F4.2: relative-offset DCE.  See dead_code_eliminate for logic.
    let mut jump_targets = std::collections::HashSet::new();
    for (ip, instr) in instructions.iter().enumerate() {
        match instr {
            Instruction::Jump(o)
            | Instruction::TryBegin(o)
            | Instruction::FinallyBegin(o)
            | Instruction::FinallyExit(o) => {
                jump_targets.insert(abs_target(ip, *o));
            }
            Instruction::JumpIfFalse { offset, .. }
            | Instruction::JumpIfTrue { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            Instruction::IterNext { end_offset, .. } => {
                jump_targets.insert(abs_target(ip, *end_offset as i32));
            }
            // Fused compare+branch instructions also encode jump targets.
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntLeRIJumpIfFalse { offset, .. }
            | Instruction::IntLtRIJumpIfFalse { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            _ => {}
        }
    }

    let mut i = 0;
    while i < instructions.len() {
        let is_terminal = matches!(
            instructions[i],
            Instruction::Jump(_) | Instruction::Return { .. } | Instruction::Throw { .. }
        );
        if !is_terminal {
            i += 1;
            continue;
        }

        let mut j = i + 1;
        while j < instructions.len() && !jump_targets.contains(&j) {
            j += 1;
        }

        if j > i + 1 {
            let removed_count = j - (i + 1);
            let len_before = instructions.len();
            for ip in 0..len_before {
                if ip >= i + 1 && ip < j {
                    continue;
                }
                let adjust = |off: &mut i32, ip: usize| {
                    let target = abs_target(ip, *off);
                    let new_ip = if ip < i + 1 { ip } else { ip - removed_count };
                    let new_target = if target < i + 1 {
                        target
                    } else if target >= j {
                        target - removed_count
                    } else {
                        i + 1
                    };
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
            instructions.drain((i + 1)..j);
            let end = j.min(source_positions.len());
            let start = (i + 1).min(end);
            source_positions.drain(start..end);
        }
        i += 1;
    }
}
