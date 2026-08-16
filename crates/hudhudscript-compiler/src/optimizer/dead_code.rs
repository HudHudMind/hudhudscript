use super::*;
use hudhudscript_bytecode::{CmpJumpPayload, LoopPayload, SuperInstrPayload};

/// Dead-code elimination: remove instructions after unconditional jumps
/// or returns that can never be reached, up to the next jump target.
///
/// Audit v3 F4.2: rewritten for i32 relative jump offsets.  Targets are
/// now resolved as `ip + offset`, and when a range `[i+1, j)` is removed
/// offsets that straddle the deleted zone need to be shrunk (if both
/// jump and target are on the same side of the zone, the offset stays
/// valid).

// G2.3: old dead_code_eliminate removed — only _with_positions remains
pub fn dead_code_eliminate_with_positions(
    instructions: &mut Vec<Instruction>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
    _loop_payloads: &mut [LoopPayload],
    _cmp_jump_payloads: &mut [CmpJumpPayload],
    _super_instr_payloads: &mut [SuperInstrPayload],
) {
    let mut dead_ips: Vec<usize> = Vec::new();
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
            Instruction::JumpIfFalse { offset, .. } | Instruction::JumpIfTrue { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            Instruction::IterNext { end_offset, .. } => {
                jump_targets.insert(abs_target(ip, *end_offset as i32));
            }
            // Fused compare+branch instructions also encode jump targets.
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntCmpIJumpIfFalse { offset, .. }
            | Instruction::IntCmpRRJumpIfFalse { offset, .. } => {
                jump_targets.insert(abs_target(ip, *offset as i32));
            }
            Instruction::CharDispatch { .. } => {
                for t in (ip + 1)..instructions.len() {
                    jump_targets.insert(t);
                }
            }
            _ => {}
        }
    }

    let mut i = 0;
    while i < instructions.len() {
        let is_terminal = matches!(
            instructions[i],
            Instruction::Jump(_) | Instruction::Return { .. }
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
                    | Instruction::IntCmpRRJumpIfFalse { offset, .. } => {
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

            let mut new_jump_targets = std::collections::HashSet::new();
            for &t in &jump_targets {
                if t < i + 1 {
                    new_jump_targets.insert(t);
                } else if t >= j {
                    new_jump_targets.insert(t - removed_count);
                } else {
                    new_jump_targets.insert(i + 1);
                }
            }
            jump_targets = new_jump_targets;

            // G2.3.2: collect dead IPs for payload remap at end
            for dead_ip in (i + 1)..j {
                dead_ips.push(dead_ip);
            }
        }
        i += 1;
    }

    // G2.3.2: remap payloads using dead_ip set.
    if !dead_ips.is_empty() {
        dead_ips.sort_unstable();
        dead_ips.dedup();
        let remap = |old_ip: usize| -> usize { old_ip - dead_ips.partition_point(|&d| d < old_ip) };
        for lp in _loop_payloads.iter_mut() {
            lp.start = remap(lp.start as usize) as u32;
            lp.end = remap(lp.end as usize) as u32;
        }
        for cjp in _cmp_jump_payloads.iter_mut() {
            cjp.target = remap(cjp.target as usize) as u32;
        }
    }
}
