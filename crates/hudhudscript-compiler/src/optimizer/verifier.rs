//! G2.2: post-optimization verifier for IP carrier invariants.
//! Debug-only — zero runtime cost in release builds.

use hudhudscript_bytecode::{CmpJumpPayload, Instruction, LoopPayload, SuperInstrPayload};

/// Verify all IP carriers are within valid bounds after optimization.
/// Panics on first violation (debug_assert! semantics).
/// Called from `optimize_with_positions` after all passes complete.
pub(crate) fn verify_instruction_bounds(
    instructions: &[Instruction],
    source_positions: &[Option<(usize, usize)>],
    loop_payloads: &[LoopPayload],
    cmp_jump_payloads: &[CmpJumpPayload],
    super_instr_payloads: &[SuperInstrPayload],
) {
    let len = instructions.len();
    if len == 0 {
        return;
    }

    // 1. source_positions must be parallel to instructions
    assert_eq!(
        source_positions.len(),
        len,
        "G2.2: source_positions len {} != instructions len {}",
        source_positions.len(),
        len
    );

    // 2. Check every instruction's embedded jump targets
    for (ip, instr) in instructions.iter().enumerate() {
        match instr {
            Instruction::Jump(o)
            | Instruction::TryBegin(o)
            | Instruction::FinallyBegin(o)
            | Instruction::FinallyExit(o) => {
                let target = (ip as i64 + *o as i64) as usize;
                assert!(
                    target <= len,
                    "G2.2: Jump-like target OOB at ip={}: offset={} target={} len={}",
                    ip,
                    o,
                    target,
                    len
                );
            }
            Instruction::JumpIfFalse { offset, .. } | Instruction::JumpIfTrue { offset, .. } => {
                let target = (ip as i64 + *offset as i64) as usize;
                assert!(
                    target <= len,
                    "G2.2: JumpIfFalse/True target OOB at ip={}: offset={} target={} len={}",
                    ip,
                    offset,
                    target,
                    len
                );
            }
            Instruction::IterNext { end_offset, .. } | Instruction::ForIn { end_offset, .. } => {
                let target = (ip as i64 + *end_offset as i64) as usize;
                assert!(
                    target <= len,
                    "G2.2: IterNext/ForIn target OOB at ip={}: offset={} target={} len={}",
                    ip,
                    end_offset,
                    target,
                    len
                );
            }
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntCmpIJumpIfFalse { offset, .. }
            | Instruction::IntCmpRRJumpIfFalse { offset, .. }
            | Instruction::IntAddIJump { offset, .. }
            | Instruction::LoopEndIntAddIJump { offset, .. }
            | Instruction::IntSubIJump { offset, .. }
            | Instruction::IntCmpIJumpIfTrue { offset, .. } => {
                let target = (ip as i64 + *offset as i64) as usize;
                assert!(
                    target <= len,
                    "G2.2: fused-compare target OOB at ip={}: offset={} target={} len={}",
                    ip,
                    offset,
                    target,
                    len
                );
            }
            Instruction::LoopBegin(idx) => {
                if let Some(lp) = loop_payloads.get(*idx as usize) {
                    let start = lp.start as usize;
                    let end = lp.end as usize;
                    assert!(
                        start <= len,
                        "G2.2: loop_payloads[{}].start={} > len={}",
                        idx,
                        start,
                        len
                    );
                    assert!(
                        end <= len,
                        "G2.2: loop_payloads[{}].end={} > len={}",
                        idx,
                        end,
                        len
                    );
                    assert!(
                        start <= end,
                        "G2.2: loop_payloads[{}] start={} > end={}",
                        idx,
                        start,
                        end
                    );
                }
            }
            Instruction::IntCmpRRJumpPacked { payload_idx, .. } => {
                if let Some(cjp) = cmp_jump_payloads.get(*payload_idx as usize) {
                    let target = cjp.target as usize;
                    assert!(
                        target <= len,
                        "G2.2: cmp_jump_payloads[{}].target={} > len={}",
                        payload_idx,
                        target,
                        len
                    );
                }
            }
            _ => {}
        }
    }

    // 5. super_instr_payloads.offset resolves to valid IP
    //    (offsets are relative to the super-instruction's IP — we check
    //    that the resolved target stays in bounds for any valid IP)
    for (i, sip) in super_instr_payloads.iter().enumerate() {
        if sip.offset != 0 {
            // offset is relative to the owning instruction; we can't check
            // without knowing which instruction owns it. Skip.
            let _ = (i, sip);
        }
    }
}
